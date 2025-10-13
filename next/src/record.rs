use std::sync::OnceLock;

use serde::{de::DeserializeOwned, Serialize};

pub trait Data: Sized {
    const TABLE_NAME: &'static str;
    type ID;
    type CODEC: Codec<Self>;
}

pub struct Record<DATA: Data> {
    pub id: DATA::ID,
    pub data: DATA
}

pub trait TxRead<Tx, DATA> {
    fn select(&self, tx: &Tx) -> Option<DATA>;
}

pub trait TxWrite<Tx, DATA> {
    fn save(&self, tx: &Tx) -> Option<DATA>;
}

impl <DATA: Data> Record<DATA> {
    pub fn new(id: DATA::ID, data: DATA) -> Self {
        Self { id, data }
    }

    pub fn select(&self) -> &'static str {
        static QUERY: OnceLock::<String> = OnceLock::new();
        QUERY.get_or_init(|| format!("select * from {} where id = $1", DATA::TABLE_NAME))
    }

}

pub struct NewRecord<DATA> {
    pub data: DATA
}

impl <DATA: Data> NewRecord<DATA> {

    pub fn insert(&self) -> &'static str {
        static QUERY: OnceLock::<String> = OnceLock::new();
        QUERY.get_or_init(|| format!("insert into {}", DATA::TABLE_NAME))
    }
}

pub trait Codec<DATA>: Serialize + DeserializeOwned {

    fn encode(data: DATA) -> Self;

    fn decode(data: Self) -> DATA;

}

impl <T: Serialize + DeserializeOwned> Codec<T> for T {
    
    #[inline(always)]
    fn encode(data: T) -> Self { data }

    #[inline(always)]
    fn decode(data: Self) -> T { data }
}
