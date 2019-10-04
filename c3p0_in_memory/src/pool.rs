use c3p0_common::*;
use c3p0_common::json::model::IdType;
use std::sync::{Arc, Mutex, MutexGuard, LockResult};
use std::ops::{Deref, DerefMut};
use std::any::Any;
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

        let mut locked_hashmap = ArcMutexGuardian::take(self.db.clone()).map_err(|err| C3p0Error::InternalError {cause: format!("{}", err)})?;

        let mut conn = InMemoryConnection::Tx(locked_hashmap);

        let result = (tx)(&conn);
        if result.is_err() {
            match conn {
                InMemoryConnection::Tx(mut locked) => {
                    *locked = db_clone;
                    result
                },
                _ => Err(C3p0Error::InternalError {cause: "InMemoryTransaction must be Tx".to_owned()})?
            }
        } else {
            result
        }
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
                let data = guard.deref();
                (tx)(data)
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
