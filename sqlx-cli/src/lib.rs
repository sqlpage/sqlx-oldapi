use anyhow::Result;
use sqlx::{AnyConnection, Connection};
use std::future::Future;
use std::io;
use std::time::Duration;
use tokio::time::{sleep, Instant};

use crate::opt::{Command, ConnectOpts, DatabaseCommand, MigrateCommand};

mod database;
mod metadata;
// mod migration;
// mod migrator;
mod migrate;
mod opt;
mod prepare;

pub use crate::opt::Opt;

pub async fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Migrate(migrate) => match migrate.command {
            MigrateCommand::Add {
                source,
                description,
                reversible,
            } => migrate::add(source.resolve(&migrate.source), &description, reversible).await?,
            MigrateCommand::Run {
                source,
                dry_run,
                ignore_missing,
                connect_opts,
            } => {
                migrate::run(
                    source.resolve(&migrate.source),
                    &connect_opts,
                    dry_run,
                    *ignore_missing,
                )
                .await?
            }
            MigrateCommand::Revert {
                source,
                dry_run,
                ignore_missing,
                connect_opts,
            } => {
                migrate::revert(
                    source.resolve(&migrate.source),
                    &connect_opts,
                    dry_run,
                    *ignore_missing,
                )
                .await?
            }
            MigrateCommand::Info {
                source,
                connect_opts,
            } => migrate::info(source.resolve(&migrate.source), &connect_opts).await?,
            MigrateCommand::BuildScript { source, force } => {
                migrate::build_script(source.resolve(&migrate.source), force)?
            }
        },

        Command::Database(database) => match database.command {
            DatabaseCommand::Create { connect_opts } => database::create(&connect_opts).await?,
            DatabaseCommand::Drop {
                confirmation,
                connect_opts,
            } => database::drop(&connect_opts, !confirmation.yes).await?,
            DatabaseCommand::Reset {
                confirmation,
                source,
                connect_opts,
            } => database::reset(&source, &connect_opts, !confirmation.yes).await?,
            DatabaseCommand::Setup {
                source,
                connect_opts,
            } => database::setup(&source, &connect_opts).await?,
        },

        Command::Prepare {
            check: false,
            merged,
            args,
            connect_opts,
        } => prepare::run(&connect_opts, merged, args).await?,

        Command::Prepare {
            check: true,
            merged,
            args,
            connect_opts,
        } => prepare::check(&connect_opts, merged, args).await?,
    };

    Ok(())
}

/// Attempt to connect to the database server, retrying up to `ops.connect_timeout`.
async fn connect(opts: &ConnectOpts) -> sqlx::Result<AnyConnection> {
    retry_connect_errors(opts, AnyConnection::connect).await
}

/// Attempt an operation that may return errors like `ConnectionRefused`,
/// retrying up until `ops.connect_timeout`.
///
/// The closure is passed `&ops.database_url` for easy composition.
async fn retry_connect_errors<'a, F, Fut, T>(
    opts: &'a ConnectOpts,
    mut connect: F,
) -> sqlx::Result<T>
where
    F: FnMut(&'a str) -> Fut,
    Fut: Future<Output = sqlx::Result<T>> + 'a,
{
    let deadline = Instant::now() + Duration::from_secs(opts.connect_timeout);
    let mut delay = Duration::from_millis(50);

    loop {
        match connect(&opts.database_url).await {
            Ok(value) => return Ok(value),
            Err(error) if is_retriable_connect_error(&error) && Instant::now() < deadline => {
                let sleep_for = delay.min(deadline.saturating_duration_since(Instant::now()));

                if sleep_for.is_zero() {
                    return Err(error);
                }

                sleep(sleep_for).await;
                delay = (delay + delay).min(Duration::from_secs(1));
            }
            Err(error) => return Err(error),
        }
    }
}

fn is_retriable_connect_error(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Io(ioe)
            if matches!(
                ioe.kind(),
                io::ErrorKind::ConnectionRefused
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::ConnectionAborted
            )
    )
}
