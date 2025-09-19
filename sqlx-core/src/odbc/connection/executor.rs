use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::logger::QueryLogger;
use crate::odbc::{
    Odbc, OdbcColumn, OdbcConnection, OdbcQueryResult, OdbcRow, OdbcStatement, OdbcTypeInfo,
};
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use odbc_api::Cursor;
use std::borrow::Cow;
use std::pin::Pin;

// run method removed; fetch_many implements streaming directly

impl<'c> Executor<'c> for &'c mut OdbcConnection {
    type Database = Odbc;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<OdbcQueryResult, OdbcRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database> + 'q,
    {
        let sql = query.sql().to_string();
        let shared = self.worker.shared.clone();
        let settings = self.log_settings.clone();
        Box::pin(try_stream! {
            let mut logger = QueryLogger::new(&sql, settings.clone());
            let guard = shared.conn.lock().await;
            match guard.execute(&sql, (), None) {
                Ok(Some(mut cursor)) => {
                    use odbc_api::ResultSetMetadata;
                    let mut columns = Vec::new();
                    if let Ok(count) = cursor.num_result_cols() {
                        for i in 1..=count { // ODBC columns are 1-based
                            let mut cd = odbc_api::ColumnDescription::default();
                            let _ = cursor.describe_col(i as u16, &mut cd);
                            let name = String::from_utf8(cd.name).unwrap_or_else(|_| format!("col{}", i-1));
                            columns.push(OdbcColumn { name, type_info: OdbcTypeInfo { name: format!("{:?}", cd.data_type), is_null: false }, ordinal: (i-1) as usize });
                        }
                    }
                    while let Some(mut row) = cursor.next_row().map_err(|e| Error::from(e))? {
                        let mut values = Vec::with_capacity(columns.len());
                        for i in 1..=columns.len() {
                            let mut buf = Vec::new();
                            let not_null = row.get_text(i as u16, &mut buf).map_err(|e| Error::from(e))?;
                            if not_null {
                                let ti = OdbcTypeInfo { name: "TEXT".into(), is_null: false };
                                values.push((ti, Some(buf)));
                            } else {
                                let ti = OdbcTypeInfo { name: "TEXT".into(), is_null: true };
                                values.push((ti, None));
                            }
                        }
                        logger.increment_rows_returned();
                        r#yield!(Either::Right(OdbcRow { columns: columns.clone(), values }));
                    }
                    r#yield!(Either::Left(OdbcQueryResult { rows_affected: 0 }));
                }
                Ok(None) => {
                    r#yield!(Either::Left(OdbcQueryResult { rows_affected: 0 }));
                }
                Err(e) => return Err(Error::from(e)),
            }
            Ok(())
        })
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<OdbcRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database> + 'q,
    {
        let mut s = self.fetch_many(query);
        Box::pin(async move {
            while let Some(v) = s.try_next().await? {
                if let Either::Right(r) = v {
                    return Ok(Some(r));
                }
            }
            Ok(None)
        })
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        _parameters: &'e [OdbcTypeInfo],
    ) -> BoxFuture<'e, Result<OdbcStatement<'q>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            // Basic statement metadata: no parameter/column info without executing
            Ok(OdbcStatement {
                sql: Cow::Borrowed(sql),
                columns: Vec::new(),
                parameters: 0,
            })
        })
    }

    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(self, _sql: &'q str) -> BoxFuture<'e, Result<Describe<Odbc>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move { Err(Error::Protocol("ODBC describe not implemented".into())) })
    }
}
