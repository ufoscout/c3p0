use c3p0_common::json::model::IdType;
use c3p0_common::*;
use guardian::ArcMutexGuardian;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct C3p0PoolInMemory {
    db: Arc<Mutex<HashMap<String, HashMap<IdType, serde_json::Value>>>>,
}

impl C3p0PoolInMemory {
    pub fn new() -> Self {
        Default::default()
    }
}

impl C3p0Pool for C3p0PoolInMemory {
    type CONN = InMemoryConnection;

    fn connection(&self) -> Result<InMemoryConnection, C3p0Error> {
        Ok(InMemoryConnection::Conn(self.db.clone()))
    }

    fn transaction<T, E: From<C3p0Error>, F: FnOnce(&InMemoryConnection) -> Result<T, E>>(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let db_clone = {
            let guard = self.db.lock().map_err(|err| C3p0Error::InternalError {
                cause: format!("{}", err),
            })?;
            guard.deref().clone()
        };

        let locked_hashmap =
            RefCell::new(ArcMutexGuardian::take(self.db.clone()).map_err(|err| {
                C3p0Error::InternalError {
                    cause: format!("{}", err),
                }
            })?);

        let conn = InMemoryConnection::Tx(locked_hashmap);

        (tx)(&conn).map_err(|err| {
            match conn {
                InMemoryConnection::Tx(locked) => {
                    let mut borrowed_lock = locked.borrow_mut();
                    **borrowed_lock.deref_mut() = db_clone;
                }
                _ => panic!("InMemoryTransaction must be Tx"),
            };
            err
        })
    }
}

pub enum InMemoryConnection {
    Conn(Arc<Mutex<HashMap<String, HashMap<IdType, serde_json::Value>>>>),
    Tx(RefCell<ArcMutexGuardian<HashMap<String, HashMap<IdType, serde_json::Value>>>>),
}

impl InMemoryConnection {
    pub fn read_db<
        T,
        E: From<C3p0Error>,
        F: FnOnce(&HashMap<String, HashMap<IdType, serde_json::Value>>) -> Result<T, E>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        match self {
            InMemoryConnection::Conn(db) => {
                let guard = db.lock().map_err(|err| C3p0Error::InternalError {
                    cause: format!("{}", err),
                })?;
                (tx)(&guard)
            }
            InMemoryConnection::Tx(locked) => {
                let borrowed_lock = locked.borrow();
                (tx)(borrowed_lock.deref())
            }
        }
    }

    pub fn write_db<
        T,
        E: From<C3p0Error>,
        F: FnOnce(&mut HashMap<String, HashMap<IdType, serde_json::Value>>) -> Result<T, E>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        match self {
            InMemoryConnection::Conn(db) => {
                let mut guard = db.lock().map_err(|err| C3p0Error::InternalError {
                    cause: format!("{}", err),
                })?;
                (tx)(&mut guard)
            }
            InMemoryConnection::Tx(locked) => {
                let mut borrowed_lock = locked.borrow_mut();
                (tx)(borrowed_lock.deref_mut())
            }
        }
    }
}

impl Connection for InMemoryConnection {
    fn batch_execute(&self, _sql: &str) -> Result<(), C3p0Error> {
        Err(C3p0Error::InternalError {
            cause: "batch_execute is not implemented for InMemoryConnection".to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_write_data_in_connection() -> Result<(), C3p0Error> {
        let pool = C3p0PoolInMemory::new();

        {
            let conn = pool.connection()?;
            let result: Result<(), C3p0Error> = conn.write_db(|db| {
                db.insert("one".to_string(), Default::default());
                Ok(())
            });
            assert!(result.is_ok())
        }

        {
            let conn = pool.connection()?;
            let result: Result<(), C3p0Error> = conn.write_db(|db| {
                assert!(db.contains_key("one"));
                db.insert("two".to_string(), Default::default());
                db.remove("one");
                Ok(())
            });
            assert!(result.is_ok())
        }

        {
            let conn = pool.connection()?;
            let result: Result<(), C3p0Error> = conn.read_db(|db| {
                assert!(!db.contains_key("one"));
                assert!(db.contains_key("two"));
                Ok(())
            });
            assert!(result.is_ok())
        }

        Ok(())
    }

    #[test]
    fn should_commit_transaction() -> Result<(), C3p0Error> {
        let pool = C3p0PoolInMemory::new();

        {
            let result: Result<(), C3p0Error> = pool.transaction(|tx| {
                tx.write_db(|db| {
                    db.insert("one".to_string(), Default::default());
                    Ok(())
                })
            });
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool.transaction(|tx| {
                tx.write_db(|db| {
                    assert!(db.contains_key("one"));
                    db.insert("two".to_string(), Default::default());
                    db.remove("one");
                    Ok(())
                })
            });
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool.transaction(|tx| {
                tx.read_db(|db| {
                    assert!(!db.contains_key("one"));
                    assert!(db.contains_key("two"));
                    Ok(())
                })
            });
            assert!(result.is_ok())
        }

        Ok(())
    }

    #[test]
    fn should_rollback_transaction() -> Result<(), C3p0Error> {
        let pool = C3p0PoolInMemory::new();

        {
            let result: Result<(), C3p0Error> = pool.transaction(|tx| {
                tx.write_db(|db| {
                    db.insert("one".to_string(), Default::default());
                    Ok(())
                })
            });
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool.transaction(|tx| {
                tx.write_db(|db| {
                    assert!(db.contains_key("one"));
                    db.insert("two".to_string(), Default::default());
                    db.remove("one");
                    Err(C3p0Error::InternalError {
                        cause: "test error on purpose".to_string(),
                    })
                })
            });
            match result {
                Err(C3p0Error::InternalError { cause }) => {
                    assert_eq!("test error on purpose", cause)
                }
                _ => assert!(false),
            }
        }

        {
            let result: Result<(), C3p0Error> = pool.transaction(|tx| {
                tx.read_db(|db| {
                    assert!(db.contains_key("one"));
                    assert!(!db.contains_key("two"));
                    Ok(())
                })
            });
            assert!(result.is_ok())
        }

        Ok(())
    }
}
