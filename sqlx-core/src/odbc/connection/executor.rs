use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::odbc::{
    Odbc, OdbcArgumentValue, OdbcConnection, OdbcQueryResult, OdbcRow, OdbcStatement, OdbcTypeInfo,
};
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
        mut _query: E,
    ) -> BoxStream<'e, Result<Either<OdbcQueryResult, OdbcRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database> + 'q,
    {
        let sql = _query.sql().to_string();
        let mut args = _query.take_arguments();
        Box::pin(try_stream! {
            let rx = if let Some(a) = args.take() {
                let new_sql = interpolate_sql_with_odbc_args(&sql, &a.values);
                self.worker.execute_stream(&new_sql).await?
            } else {
                self.worker.execute_stream(&sql).await?
            };
            while let Ok(item) = rx.recv_async().await {
                r#yield!(item?);
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

fn interpolate_sql_with_odbc_args(sql: &str, args: &[OdbcArgumentValue<'_>]) -> String {
    let mut result = String::with_capacity(sql.len() + args.len() * 8);
    let mut arg_iter = args.iter();
    for ch in sql.chars() {
        if ch == '?' {
            if let Some(arg) = arg_iter.next() {
                match arg {
                    OdbcArgumentValue::Int(i) => result.push_str(&i.to_string()),
                    OdbcArgumentValue::Float(f) => result.push_str(&format!("{}", f)),
                    OdbcArgumentValue::Text(s) => {
                        result.push('\'');
                        for c in s.chars() {
                            if c == '\'' {
                                result.push('\'');
                            }
                            result.push(c);
                        }
                        result.push('\'');
                    }
                    OdbcArgumentValue::Bytes(b) => {
                        result.push_str("X'");
                        for byte in b {
                            result.push_str(&format!("{:02X}", byte));
                        }
                        result.push('\'');
                    }
                    OdbcArgumentValue::Null | OdbcArgumentValue::Phantom(_) => {
                        result.push_str("NULL")
                    }
                }
            } else {
                result.push('?');
            }
        } else {
            result.push(ch);
        }
    }
    result
}
