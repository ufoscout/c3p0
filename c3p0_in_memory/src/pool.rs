use c3p0_common::*;
use c3p0_common::json::model::IdType;
use std::sync::{Arc, Mutex};
use std::ops::{Deref};
use chashmap::CHashMap;
use guardian::ArcMutexGuardian;

#[derive(Clone, Default)]
pub struct C3p0PoolInMemory {
    db: Arc<Mutex<CHashMap<String, CHashMap<IdType, serde_json::Value>>>>
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
            let guard = self.db.lock().map_err(|err| C3p0Error::InternalError {cause: format!("{}", err)})?;
            guard.deref().clone()
        };

        let locked_hashmap = ArcMutexGuardian::take(self.db.clone()).map_err(|err| C3p0Error::InternalError {cause: format!("{}", err)})?;

        let conn = InMemoryConnection::Tx(locked_hashmap);

        (tx)(&conn).map_err(|err| {
            match conn {
                InMemoryConnection::Tx(mut locked) => {
                    *locked = db_clone;
                },
                _ => panic!("InMemoryTransaction must be Tx")
            };
            err
        })
    }
}

pub enum  InMemoryConnection {
    Conn(Arc<Mutex<CHashMap<String, CHashMap<IdType, serde_json::Value>>>>),
    Tx(ArcMutexGuardian<CHashMap<String, CHashMap<IdType, serde_json::Value>>>),
}

impl InMemoryConnection {

    pub fn get_db<T, E: From<C3p0Error>, F: FnOnce(&CHashMap<String, CHashMap<IdType, serde_json::Value>>) -> Result<T, E>>
        (&self, tx: F) -> Result<T, E> {

        match self {
            InMemoryConnection::Conn(db) => {
                let guard = db.lock().map_err(|err| C3p0Error::InternalError {cause: format!("{}", err)})?;
                (tx)(&guard)
            },
            InMemoryConnection::Tx(locked) => {
                (tx)(&locked)
            }
        }

    }

}

impl Connection for InMemoryConnection {
    fn batch_execute(&self, _sql: &str) -> Result<(), C3p0Error> {
        Err(C3p0Error::InternalError{cause: "batch_execute is not implemented for InMemoryConnection".to_string()})
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
            let result: Result<(), C3p0Error> = conn.get_db(|db| {
                db.insert("one".to_string(), Default::default());
                Ok(())
            });
            assert!(result.is_ok())
        }

        {
            let conn = pool.connection()?;
            let result: Result<(), C3p0Error> = conn.get_db(|db| {
                assert!(db.contains_key("one"));
                db.insert("two".to_string(), Default::default());
                db.remove("one");
                Ok(())
            });
            assert!(result.is_ok())
        }

        {
            let conn = pool.connection()?;
            let result: Result<(), C3p0Error> = conn.get_db(|db| {
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
                tx.get_db(|db| {
                    db.insert("one".to_string(), Default::default());
                    Ok(())
                })
            });
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool.transaction(|tx| {
                tx.get_db(|db| {
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
                tx.get_db(|db| {
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
                tx.get_db(|db| {
                    db.insert("one".to_string(), Default::default());
                    Ok(())
                })
            });
            assert!(result.is_ok())
        }

        {
            let result: Result<(), C3p0Error> = pool.transaction(|tx| {
                tx.get_db(|db| {
                    assert!(db.contains_key("one"));
                    db.insert("two".to_string(), Default::default());
                    db.remove("one");
                    Err(C3p0Error::InternalError {cause: "test error on purpose".to_string() })
                })

            });
            match result {
                Err(C3p0Error::InternalError {cause}) => assert_eq!("test error on purpose", cause),
                _ => assert!(false)
            }
        }

        {
            let result: Result<(), C3p0Error> = pool.transaction(|tx| {
                tx.get_db(|db| {
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