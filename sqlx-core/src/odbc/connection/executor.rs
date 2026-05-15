use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::odbc::{Odbc, OdbcConnection, OdbcQueryResult, OdbcRow, OdbcStatement, OdbcTypeInfo};
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;

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
        let args = query.take_arguments();
        Box::pin(self.execute_stream(query.sql(), args).into_stream())
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<OdbcRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database> + 'q,
    {
        let mut stream = self.fetch_many(query);

        Box::pin(async move {
            while let Some(step) = stream.try_next().await? {
                if let Either::Right(row) = step {
                    return Ok(Some(row));
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
        Box::pin(async move { self.prepare(sql).await })
    }

    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(self, sql: &'q str) -> BoxFuture<'e, Result<Describe<Odbc>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            let statement = self.describe_statement(sql).await?;
            let nullable = vec![None; statement.metadata.columns.len()];

            Ok(Describe {
                columns: statement.metadata.columns,
                parameters: Some(Either::Right(statement.metadata.parameters)),
                nullable,
            })
        })
    }
}
