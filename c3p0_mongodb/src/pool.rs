use crate::*;

use async_trait::async_trait;
use c3p0_common::*;
use ::mongodb::{Client, Database, options::SessionOptions, ClientSession};
use std::future::Future;

#[derive(Clone)]
pub struct MongodbC3p0Pool {
    pool: Client,
    database: String,
}

impl MongodbC3p0Pool {
    pub fn new(pool: Client, database: String) -> Self {
        MongodbC3p0Pool { pool, database }
    }
}

impl From<(Client, String)> for MongodbC3p0Pool {
    fn from((pool, database): (Client, String)) -> Self {
        MongodbC3p0Pool::new(pool, database)
    }
}

#[async_trait]
impl C3p0Pool for MongodbC3p0Pool {
    type Tx = MongodbTx;

    async fn transaction<
        'a,
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(&'a mut Self::Tx) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &'a self,
        tx: F,
    ) -> Result<T, E> {

        let session_options = SessionOptions::builder()
            // .causal_consistency(true)
            .build();
        let mut session = self.pool.start_session(session_options).await.map_err(into_c3p0_error)?;
        session.start_transaction(None).await.map_err(into_c3p0_error)?;

        let client = session.client();
        let database = client.database(&self.database);

        // ToDo: To avoid this unsafe we need GAT
        let mut transaction = MongodbTx {
            inner: database,
            session,
        };
        let ref_transaction = unsafe { ::std::mem::transmute(&mut transaction) };
        let result = { (tx)(ref_transaction).await };

        match result {
            Ok(result) => {
                transaction.session.commit_transaction().await.map_err(into_c3p0_error)?;
                Ok(result)
            }
            Err(err) => {
                transaction.session.abort_transaction().await.map_err(into_c3p0_error)?;
                Err(err)
            }
        }

    }
}

pub struct MongodbTx {
    inner: Database,
    session: ClientSession,
}

impl MongodbTx {
    pub fn db(&mut self) -> (&mut Database, &mut ClientSession) {
        (&mut self.inner, &mut self.session)
    }
}
