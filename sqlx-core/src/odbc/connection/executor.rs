use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::odbc::{Odbc, OdbcConnection, OdbcQueryResult, OdbcRow, OdbcStatement, OdbcTypeInfo};
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use std::borrow::Cow;

// run method removed; fetch_many implements streaming directly

impl<'c> Executor<'c> for &'c mut OdbcConnection {
    type Database = Odbc;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        _query: E,
    ) -> BoxStream<'e, Result<Either<OdbcQueryResult, OdbcRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database> + 'q,
    {
        let empty: Vec<Result<Either<OdbcQueryResult, OdbcRow>, Error>> = Vec::new();
        Box::pin(futures_util::stream::iter(empty))
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
