use crate::acquire::Acquire;
use crate::migrate::{AppliedMigration, Migrate, MigrateError, Migration, MigrationSource};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::slice;

#[derive(Debug)]
pub struct Migrator {
    pub migrations: Cow<'static, [Migration]>,
    pub ignore_missing: bool,
    pub locking: bool,
}

fn validate_applied_migrations(
    applied_migrations: &[AppliedMigration],
    migrator: &Migrator,
) -> Result<(), MigrateError> {
    if migrator.ignore_missing {
        return Ok(());
    }

    let migrations: HashSet<_> = migrator.iter().map(|m| m.version).collect();

    for applied_migration in applied_migrations {
        if !migrations.contains(&applied_migration.version) {
            return Err(MigrateError::VersionMissing(applied_migration.version));
        }
    }

    Ok(())
}

impl Migrator {
    /// Creates a new instance with the given source.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use sqlx_core_oldapi::migrate::MigrateError;
    /// # fn main() -> Result<(), MigrateError> {
    /// # sqlx_rt::block_on(async move {
    /// # use sqlx_core_oldapi::migrate::Migrator;
    /// use std::path::Path;
    ///
    /// // Read migrations from a local folder: ./migrations
    /// let m = Migrator::new(Path::new("./migrations")).await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// See [MigrationSource] for details on structure of the `./migrations` directory.
    pub async fn new<'s, S>(source: S) -> Result<Self, MigrateError>
    where
        S: MigrationSource<'s>,
    {
        Ok(Self {
            migrations: Cow::Owned(source.resolve().await.map_err(MigrateError::Source)?),
            ignore_missing: false,
            locking: true,
        })
    }

    /// Specify whether applied migrations that are missing from the resolved migrations should be ignored.
    pub fn set_ignore_missing(&mut self, ignore_missing: bool) -> &Self {
        self.ignore_missing = ignore_missing;
        self
    }

    /// Specify whether or not to lock database during migration. Defaults to `true`.
    ///
    /// ### Warning
    /// Disabling locking can lead to errors or data loss if multiple clients attempt to apply migrations simultaneously
    /// without some sort of mutual exclusion.
    ///
    /// This should only be used if the database does not support locking, e.g. CockroachDB which talks the Postgres
    /// protocol but does not support advisory locks used by SQLx's migrations support for Postgres.
    pub fn set_locking(&mut self, locking: bool) -> &Self {
        self.locking = locking;
        self
    }

    /// Get an iterator over all known migrations.
    pub fn iter(&self) -> slice::Iter<'_, Migration> {
        self.migrations.iter()
    }

    /// Run any pending migrations against the database; and, validate previously applied migrations
    /// against the current migration source to detect accidental changes in previously-applied migrations.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use sqlx_core_oldapi::migrate::MigrateError;
    /// # #[cfg(feature = "sqlite")]
    /// # fn main() -> Result<(), MigrateError> {
    /// #     sqlx_rt::block_on(async move {
    /// # use sqlx_core_oldapi::migrate::Migrator;
    /// let m = Migrator::new(std::path::Path::new("./migrations")).await?;
    /// let pool = sqlx_core_oldapi::sqlite::SqlitePoolOptions::new().connect("sqlite::memory:").await?;
    /// m.run(&pool).await
    /// #     })
    /// # }
    /// ```
    pub async fn run<'a, A>(&self, migrator: A) -> Result<(), MigrateError>
    where
        A: Acquire<'a>,
        <A::Connection as Deref>::Target: Migrate,
    {
        let mut conn = migrator
            .acquire()
            .await
            .map_err(MigrateError::AcquireConnection)?;
        self.run_direct(&mut *conn).await
    }

    // Getting around the annoying "implementation of `Acquire` is not general enough" error
    #[doc(hidden)]
    pub async fn run_direct<C>(&self, conn: &mut C) -> Result<(), MigrateError>
    where
        C: Migrate,
    {
        // lock the database for exclusive access by the migrator
        if self.locking {
            conn.lock().await.map_err(MigrateError::AcquireConnection)?;
        }

        // creates [_migrations] table only if needed
        // eventually this will likely migrate previous versions of the table
        conn.ensure_migrations_table()
            .await
            .map_err(MigrateError::AccessMigrationMetadata)?;

        let version = conn
            .dirty_version()
            .await
            .map_err(MigrateError::AccessMigrationMetadata)?;
        if let Some(version) = version {
            return Err(MigrateError::Dirty(version));
        }

        let applied_migrations = conn
            .list_applied_migrations()
            .await
            .map_err(MigrateError::AccessMigrationMetadata)?;
        validate_applied_migrations(&applied_migrations, self)?;

        let applied_migrations: HashMap<_, _> = applied_migrations
            .into_iter()
            .map(|m| (m.version, m))
            .collect();

        for migration in self.iter() {
            if migration.migration_type.is_down_migration() {
                continue;
            }

            match applied_migrations.get(&migration.version) {
                Some(applied_migration) => {
                    if migration.checksum != applied_migration.checksum {
                        return Err(MigrateError::VersionMismatch(migration.version));
                    }
                }
                None => {
                    conn.apply(migration)
                        .await
                        .map_err(|e| MigrateError::Execute(migration.version, e))?;
                }
            }
        }

        // unlock the migrator to allow other migrators to run
        // but do nothing as we already migrated
        if self.locking {
            conn.unlock()
                .await
                .map_err(MigrateError::AcquireConnection)?;
        }

        Ok(())
    }

    /// Run down migrations against the database until a specific version.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use sqlx_core_oldapi::migrate::MigrateError;
    /// # #[cfg(feature = "sqlite")]
    /// # fn main() -> Result<(), MigrateError> {
    /// #     sqlx_rt::block_on(async move {
    /// # use sqlx_core_oldapi::migrate::Migrator;
    /// let m = Migrator::new(std::path::Path::new("./migrations")).await?;
    /// let pool = sqlx_core_oldapi::sqlite::SqlitePoolOptions::new().connect("sqlite::memory:").await?;
    /// m.undo(&pool, 4).await
    /// #     })
    /// # }
    /// ```
    pub async fn undo<'a, A>(&self, migrator: A, target: i64) -> Result<(), MigrateError>
    where
        A: Acquire<'a>,
        <A::Connection as Deref>::Target: Migrate,
    {
        let mut conn = migrator
            .acquire()
            .await
            .map_err(MigrateError::AcquireConnection)?;

        // lock the database for exclusive access by the migrator
        if self.locking {
            conn.lock().await.map_err(MigrateError::AcquireConnection)?;
        }

        // creates [_migrations] table only if needed
        // eventually this will likely migrate previous versions of the table
        conn.ensure_migrations_table()
            .await
            .map_err(MigrateError::AccessMigrationMetadata)?;

        let version = conn
            .dirty_version()
            .await
            .map_err(MigrateError::AccessMigrationMetadata)?;
        if let Some(version) = version {
            return Err(MigrateError::Dirty(version));
        }

        let applied_migrations = conn
            .list_applied_migrations()
            .await
            .map_err(MigrateError::AccessMigrationMetadata)?;
        validate_applied_migrations(&applied_migrations, self)?;

        let applied_migrations: HashMap<_, _> = applied_migrations
            .into_iter()
            .map(|m| (m.version, m))
            .collect();

        for migration in self
            .iter()
            .rev()
            .filter(|m| m.migration_type.is_down_migration())
            .filter(|m| applied_migrations.contains_key(&m.version))
            .filter(|m| m.version > target)
        {
            conn.revert(migration)
                .await
                .map_err(|e| MigrateError::Execute(migration.version, e))?;
        }

        // unlock the migrator to allow other migrators to run
        // but do nothing as we already migrated
        if self.locking {
            conn.unlock()
                .await
                .map_err(MigrateError::AcquireConnection)?;
        }

        Ok(())
    }
}
