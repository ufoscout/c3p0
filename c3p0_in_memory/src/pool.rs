use async_trait::async_trait;
use c3p0_common::*;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Mutex;

type Db = HashMap<String, BTreeMap<IdType, Model<serde_json::Value>>>;

#[derive(Clone, Default)]
pub struct InMemoryC3p0Pool {
    db: Arc<Mutex<Db>>,
}

impl InMemoryC3p0Pool {
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
impl C3p0Pool for InMemoryC3p0Pool {
    type Conn = InMemoryConnection;

    async fn transaction<
        'a,
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(&'a mut Self::Conn) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &'a self,
        tx: F,
    ) -> Result<T, E> {
        let mut guard = self.db.lock().await;
        // .map_err(|err| C3p0Error::InternalError {
        //         cause: format!("{:?}", err),
        //     })?;
        let db = guard.deref();
        let mut db_clone = db.clone();

        // ToDo: To avoid this unsafe we need GAT
        let mut transaction = InMemoryConnection {
            db: unsafe { ::std::mem::transmute(&mut db_clone) },
        };
        let ref_transaction = unsafe { ::std::mem::transmute(&mut transaction) };

        let result = (tx)(ref_transaction).await?;
        *guard = db_clone;

        Ok(result)
    }
}

pub struct InMemoryConnection {
    db: &'static mut Db,
}

impl Deref for InMemoryConnection {
    type Target = Db;

    fn deref(&self) -> &Self::Target {
        self.db
    }
}

impl DerefMut for InMemoryConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn should_write_data_in_connection() -> Result<(), C3p0Error> {
        let pool = InMemoryC3p0Pool::new();

        {
            let result: Result<(), C3p0Error> = pool
                .transaction(|conn| async {
                    conn.insert("one".to_string(), Default::default());
                    Ok(())
                })
                .await;
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool
                .transaction(|db| async {
                    assert!(db.contains_key("one"));
                    db.insert("two".to_string(), Default::default());
                    db.remove("one");
                    Ok(())
                })
                .await;
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool
                .transaction(|db| async {
                    assert!(!db.contains_key("one"));
                    assert!(db.contains_key("two"));
                    Ok(())
                })
                .await;
            assert!(result.is_ok())
        }

        Ok(())
    }

    #[tokio::test]
    async fn should_commit_transaction() -> Result<(), C3p0Error> {
        let pool = InMemoryC3p0Pool::new();

        {
            let result: Result<(), C3p0Error> = pool
                .transaction(|db| async {
                    db.insert("one".to_string(), Default::default());
                    Ok(())
                })
                .await;
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool
                .transaction(|db| async {
                    assert!(db.contains_key("one"));
                    db.insert("two".to_string(), Default::default());
                    db.remove("one");
                    Ok(())
                })
                .await;
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool
                .transaction(|tx| async {
                    assert!(!tx.contains_key("one"));
                    assert!(tx.contains_key("two"));
                    Ok(())
                })
                .await;
            assert!(result.is_ok())
        }

        Ok(())
    }

    #[tokio::test]
    async fn should_rollback_transaction() -> Result<(), C3p0Error> {
        let pool = InMemoryC3p0Pool::new();

        {
            let result: Result<(), C3p0Error> = pool
                .transaction(|tx| async {
                    tx.insert("one".to_string(), Default::default());
                    Ok(())
                })
                .await;
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool
                .transaction(|tx| async {
                    assert!(tx.contains_key("one"));
                    tx.insert("two".to_string(), Default::default());
                    tx.remove("one");
                    Err(C3p0Error::InternalError {
                        cause: "test error on purpose".to_string(),
                    })
                })
                .await;
            match result {
                Err(C3p0Error::InternalError { cause }) => {
                    assert_eq!("test error on purpose", cause)
                }
                _ => assert!(false),
            }
        }

        {
            let result: Result<(), C3p0Error> = pool
                .transaction(|tx| async move {
                    assert!(tx.contains_key("one"));
                    assert!(!tx.contains_key("two"));
                    Ok(())
                })
                .await;
            assert!(result.is_ok())
        }

        Ok(())
    }
}
