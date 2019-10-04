use c3p0_common::*;
use c3p0_common::json::model::IdType;
use std::sync::{Arc, Mutex, MutexGuard, LockResult};
use std::ops::{Deref, DerefMut};
use std::any::Any;
use chashmap::CHashMap;

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

        let locked_hashmap = new_locked(self.db.clone())?;
        //locked_hashmap.rent(|data| *data = db_clone);

        let conn = InMemoryConnection::Tx(locked_hashmap);

        let result = (tx)(&conn);
        match &result {
            Ok(smt) => {
            },
            _ => {}
        };
        result
    }
}

fn new_locked(
    repo: Arc<Mutex<CHashMap<String, CHashMap<IdType, serde_json::Value>>>>,
) -> Result<rentals::LockedHashMap, C3p0Error> {
    rentals::LockedHashMap::try_new_or_drop(
        repo,
        |c| {
            Ok(c.lock().map_err(|err| C3p0Error::InternalError {cause: format!("{}", err)})?)
        })
}

rental! {
    pub mod rentals {
        use super::*;

        #[rental]
        pub struct LockedHashMap {
            lock: Arc<Mutex<CHashMap<String, CHashMap<IdType, serde_json::Value>>>>,
            guard: std::sync::MutexGuard<'lock, CHashMap<String, CHashMap<IdType, serde_json::Value>>>,
        }
    }
}

pub enum  InMemoryConnection {
    Conn(Arc<Mutex<CHashMap<String, CHashMap<IdType, serde_json::Value>>>>),
    Tx(rentals::LockedHashMap),
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
                locked.rent(|data| (tx)(data))
            }
        }

    }

}

impl Connection for InMemoryConnection {
    fn batch_execute(&self, _sql: &str) -> Result<(), C3p0Error> {
        Err(C3p0Error::InternalError{cause: "batch_execute is not implemented for InMemoryConnection".to_string()})
    }
}
