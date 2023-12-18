use deadpool::managed;

use async_trait::async_trait;
use c3p0_common::*;
use surrealdb::{Surreal, Connection, Error as SurrealError, sql::statements::{BeginStatement, CommitStatement, CancelStatement}};
use std::{future::Future, sync::Arc, pin::Pin};

use crate::{deadpool_into_c3p0_error, into_c3p0_error};

pub struct SurrealdbManager<C: Connection> {
    builder: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<Surreal<C>, SurrealError>> + Send + Sync + 'static>> + Send + Sync>
}

impl <C: Connection> SurrealdbManager<C> {

    pub fn new(builder: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<Surreal<C>, SurrealError>> + Send + Sync + 'static>> + Send + Sync>) -> Self {
        SurrealdbManager {
            builder
        }
    }
    
}

impl <C: Connection> Clone for SurrealdbManager<C> {
    fn clone(&self) -> Self {
        Self { builder: self.builder.clone() }
    }
}

#[async_trait]
impl <C: Connection> managed::Manager for SurrealdbManager<C> {
    type Type = Surreal<C>;
    type Error =  SurrealError;
    
    async fn create(&self) -> Result<Surreal<C>, Self::Error> {
        (self.builder)().await
    }
    
    async fn recycle(&self, _: &mut Surreal<C>, _: &managed::Metrics) -> managed::RecycleResult<Self::Error> {
        Ok(())
    }
}

pub type SurrealdbPool<C> = managed::Pool<SurrealdbManager<C>>;

pub struct SurrealdbC3p0Pool<C: Connection> {
    pool: SurrealdbPool<C>,
}

impl <C: Connection> Clone for SurrealdbC3p0Pool<C> {
    fn clone(&self) -> Self {
        Self { pool: self.pool.clone() }
    }
}

impl <C: Connection> SurrealdbC3p0Pool<C> {
    pub fn new(pool: SurrealdbPool<C>) -> Self {
        SurrealdbC3p0Pool { pool }
    }
}

impl <C: Connection> From<SurrealdbPool<C>> for SurrealdbC3p0Pool<C> {
    fn from(pool: SurrealdbPool<C>) -> Self {
        SurrealdbC3p0Pool::new(pool)
    }
}

#[async_trait]
impl <C: Connection> C3p0Pool for SurrealdbC3p0Pool<C> {
    type Tx = SurrealdbTx<C>;

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
        let conn = self.pool.get().await.map_err(deadpool_into_c3p0_error)?;

        println!("transaction: begin");
        println!("transaction: begin");
        println!("transaction: begin");
        println!("transaction: begin");
        println!("transaction: begin");
        let res = conn.query(BeginStatement).await.map_err(into_c3p0_error)?;

        println!("transaction: end");
        println!("transaction: end");
        println!("transaction: end");
        println!("transaction: end");
        println!("transaction: end");

        // ToDo: To avoid this unsafe we need GAT
        let mut transaction: SurrealdbTx<C> = SurrealdbTx {
            inner: conn,
        };
        let ref_transaction = unsafe { ::std::mem::transmute(&mut transaction) };

        println!("transaction: start");
        println!("transaction: start");
        println!("transaction: start");
        println!("transaction: start");
        println!("transaction: start");
        match { (tx)(ref_transaction).await } {
            Ok(result) => {
                println!("transaction: executed -> success");
                println!("transaction: executed -> success");
                println!("transaction: executed -> success");
                println!("transaction: executed -> success");
                println!("transaction: executed -> success");
                let _res = transaction.inner.query(CommitStatement).await.map_err(into_c3p0_error)?;
                Ok(result)
            },
            Err(err) => {
                println!("transaction: executed -> failure");
                println!("transaction: executed -> failure");
                println!("transaction: executed -> failure");
                println!("transaction: executed -> failure");
                let _res = transaction.inner.query(CancelStatement).await.map_err(into_c3p0_error)?;
                Err(err)
            }
        }
    }
}

pub struct SurrealdbTx<C: Connection> {
    inner: managed::Object<SurrealdbManager<C>>,
}

#[async_trait]
impl <C: Connection> SqlTx for SurrealdbTx<C> {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        // todo
        let todo = 0;
        //self.inner.batch_execute(sql).await.map_err(into_c3p0_error)
        todo!()
    }
}


#[cfg(test)]
mod test {

    use deadpool::managed::Pool;
    use serde::{Deserialize, Serialize};
    use surrealdb::{engine::local::Mem, sql::Thing};

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        name: String,
    }

    #[derive(Debug, Deserialize)]
    struct Record {
        id: Thing,
    }

    #[tokio::test]
    async fn test_create_deadpool() {
        let manager = SurrealdbManager::new(Arc::new(|| {
            Box::pin(async move {
                let db = Surreal::new::<Mem>(()).await?;
                db.use_ns("test").use_db("test").await?;
                Ok(db)
            })
        }));
        let deadpool = Pool::builder(manager).max_size(50).build().unwrap();
        let pool = SurrealdbC3p0Pool::new(deadpool);

        let result = pool.transaction(|tx| async {
            // Create a new person with a random id
            let created: Result<Vec<Record>, _> = tx.inner
                .create("person")
                .content(Person {
                    name: "Tobie".to_owned(),
                })
                .await.map_err(into_c3p0_error);
            created
        }).await;
        assert!(result.is_ok());
    }

}