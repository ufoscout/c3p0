use c3p0_common::json::model::IdType;
use c3p0_common::*;
use guardian::ArcMutexGuardian;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

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

impl C3p0Pool for InMemoryC3p0Pool {
    type CONN = InMemoryConnection;

    fn connection(&self) -> Result<InMemoryConnection, C3p0Error> {
        Ok(InMemoryConnection::Conn(self.db.clone()))
    }

    fn transaction<T, E: From<C3p0Error>, F: FnOnce(&mut InMemoryConnection) -> Result<T, E>>(
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
            ArcMutexGuardian::take(self.db.clone()).map_err(|err| C3p0Error::InternalError {
                cause: format!("{}", err),
            })?;

        let mut conn = InMemoryConnection::Tx(locked_hashmap);

        (tx)(&mut conn).map_err(|err| {
            match conn {
                InMemoryConnection::Tx(mut locked) => {
                    *locked.deref_mut() = db_clone;
                }
                _ => panic!("InMemoryTransaction must be Tx"),
            };
            err
        })
    }
}

pub enum InMemoryConnection {
    Conn(Arc<Mutex<Db>>),
    Tx(ArcMutexGuardian<Db>),
}

impl InMemoryConnection {
    pub fn read_db<T, E: From<C3p0Error>, F: FnOnce(&Db) -> Result<T, E>>(
        &mut self,
        tx: F,
    ) -> Result<T, E> {
        match self {
            InMemoryConnection::Conn(db) => {
                let guard = db.lock().map_err(|err| C3p0Error::InternalError {
                    cause: format!("{}", err),
                })?;
                (tx)(&guard)
            }
            InMemoryConnection::Tx(locked) => (tx)(locked.deref()),
        }
    }

    pub fn write_db<T, E: From<C3p0Error>, F: FnOnce(&mut Db) -> Result<T, E>>(
        &mut self,
        tx: F,
    ) -> Result<T, E> {
        match self {
            InMemoryConnection::Conn(db) => {
                let mut guard = db.lock().map_err(|err| C3p0Error::InternalError {
                    cause: format!("{}", err),
                })?;
                (tx)(&mut guard)
            }
            InMemoryConnection::Tx(locked) => (tx)(locked.deref_mut()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_write_data_in_connection() -> Result<(), C3p0Error> {
        let pool = InMemoryC3p0Pool::new();

        {
            let conn = &mut pool.connection()?;
            let result: Result<(), C3p0Error> = conn.write_db(|db| {
                db.insert("one".to_string(), Default::default());
                Ok(())
            });
            assert!(result.is_ok())
        }

        {
            let conn = &mut pool.connection()?;
            let result: Result<(), C3p0Error> = conn.write_db(|db| {
                assert!(db.contains_key("one"));
                db.insert("two".to_string(), Default::default());
                db.remove("one");
                Ok(())
            });
            assert!(result.is_ok())
        }

        {
            let conn = &mut pool.connection()?;
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
        let pool = InMemoryC3p0Pool::new();

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
        let pool = InMemoryC3p0Pool::new();

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
