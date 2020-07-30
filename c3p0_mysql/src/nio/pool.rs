use crate::common::to_value_mapper;
use crate::nio::error::into_c3p0_error;
use async_trait::async_trait;
use c3p0_common::*;
use futures::Future;
use mysql_async::prelude::{FromValue, Queryable, ToValue};
use mysql_async::{Pool, Row, Transaction, TxOpts};

pub enum MysqlC3p0ConnectionManager {
    DeadPool,
}

impl MysqlC3p0ConnectionManager {}

#[derive(Clone)]
pub struct MysqlC3p0PoolAsync {
    pool: Pool,
}

impl MysqlC3p0PoolAsync {
    pub fn new(pool: Pool) -> Self {
        MysqlC3p0PoolAsync { pool }
    }
}

impl Into<MysqlC3p0PoolAsync> for Pool {
    fn into(self) -> MysqlC3p0PoolAsync {
        MysqlC3p0PoolAsync::new(self)
    }
}

#[async_trait]
impl C3p0PoolAsync for MysqlC3p0PoolAsync {
    type Conn = MysqlConnectionAsync;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(Self::Conn) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut conn = self.pool.get_conn().await.map_err(into_c3p0_error)?;

        let mut native_transaction = conn
            .start_transaction(TxOpts::default())
            .await
            .map_err(into_c3p0_error)?;

        // ToDo: To avoid this unsafe we need GAT
        let transaction =
            MysqlConnectionAsync::Tx(unsafe { ::std::mem::transmute(&mut native_transaction) });

        let result = { (tx)(transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub enum MysqlConnectionAsync {
    Tx(&'static mut Transaction<'static>),
}

#[async_trait]
impl SqlConnectionAsync for MysqlConnectionAsync {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            MysqlConnectionAsync::Tx(tx) => tx.query_drop(sql).await.map_err(into_c3p0_error),
        }
    }
}

impl MysqlConnectionAsync {
    pub async fn execute(&mut self, sql: &str, params: &[&dyn ToValue]) -> Result<u64, C3p0Error> {
        match self {
            MysqlConnectionAsync::Tx(tx) => tx
                .exec_iter(sql, params)
                .await
                .map(|row| row.affected_rows())
                .map_err(into_c3p0_error),
        }
    }

    pub async fn fetch_one_value<T: FromValue>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
    ) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper).await
    }

    pub async fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        self.fetch_one_optional(sql, params, mapper)
            .await
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub async fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            MysqlConnectionAsync::Tx(tx) => {
                let mut result = tx.exec_iter(sql, params).await.map_err(into_c3p0_error)?;

                if let Some(row) = result.next().await.map_err(into_c3p0_error)? {
                    Ok(Some(mapper(&row).map_err(|err| {
                        C3p0Error::RowMapperError {
                            cause: format!("{}", err),
                        }
                    })?))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            MysqlConnectionAsync::Tx(tx) => {
                let mut result = tx.exec_iter(sql, params).await.map_err(into_c3p0_error)?;

                let mut rows = vec![];
                while let Some(row) = result.next().await.map_err(into_c3p0_error)? {
                    rows.push(mapper(&row).map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })?);
                }
                Ok(rows)
            }
        }
    }

    pub async fn fetch_all_values<T: FromValue>(
        &mut self,
        sql: &str,
        params: &[&dyn ToValue],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper).await
    }
}
