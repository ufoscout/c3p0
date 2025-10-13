use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::{codec::Codec, error::C3p0Error};

pub trait Data: Sized + Send + Sync {
    const TABLE_NAME: &'static str;
    type CODEC: Codec<Self>;
}

/// A model for a database table.
/// This is used to retrieve and update an entry in a database table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Record<DATA: Data> {
    /// The unique identifier of the model.
    pub id: u64,
    /// The version of the model used for optimistic locking.
    pub version: i64,
    /// The epoch millis when the model was created.
    #[serde(default)]
    pub create_epoch_millis: i64,
    /// The epoch millis when the model was last updated.
    #[serde(default)]
    pub update_epoch_millis: i64,
    /// The data of the model.
    pub data: DATA,
}


/// A new model for a database table.
/// This is used to create a new entry in a database table.
pub struct NewRecord<DATA> {
    pub data: DATA
}

impl<DATA: Data> NewRecord<DATA> {
    /// Creates a new `NewRecord` instance from a `Data` value.
    /// Sets the version to 0.
    pub fn new(data: DATA) -> Self {
        NewRecord { data }
    }

}

impl<DATA: Data + Default> Default for NewRecord<DATA> {
    fn default() -> Self {
        NewRecord::new(DATA::default())
    }
}

impl<DATA> From<DATA> for NewRecord<DATA>
where
    DATA: Data,
{
    fn from(data: DATA) -> Self {
        NewRecord::new(data)
    }
}

impl<DATA: Data> Record<DATA> {
    /// Converts the current `Record` instance into a `NewRecord` instance,
    /// resetting the version to the initial state while retaining the data.
    pub(crate) fn into_new(self) -> NewRecord<DATA> {
        NewRecord::new(self.data)
    }

    /// Creates a new `Record` instance from a `NewRecord` instance.
    ///
    /// - `id`: The unique identifier of the model.
    /// - `create_epoch_millis`: The epoch millis when the model was created.
    /// - `model`: The `NewRecord` instance to create the `Record` instance from.
    ///
    /// Returns a `Record` instance with the version set to the initial state,
    /// the create and update epoch millis set to the given `create_epoch_millis`,
    /// and the data set to the data of the `model` parameter.
    pub(crate) fn from_new(
        id: u64,
        create_epoch_millis: i64,
        model: NewRecord<DATA>,
    ) -> Record<DATA> {
        Record {
            id,
            version: 0,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
            data: model.data,
        }
    }

    /// Creates a new `Record` instance from the current `Record` instance,
    /// incrementing the version by one and updating the update epoch millis
    /// to the given `update_epoch_millis`.
    ///
    /// - `update_epoch_millis`: The epoch millis when the model was last updated.
    ///
    /// Returns a `Record` instance with the version incremented by one,
    /// the create epoch millis unchanged, the update epoch millis set to
    /// the given `update_epoch_millis`, and the data unchanged.
    pub(crate) fn into_new_version(self, update_epoch_millis: i64) -> Record<DATA> {
        Record {
            id: self.id,
            version: self.version + 1,
            create_epoch_millis: self.create_epoch_millis,
            update_epoch_millis,
            data: self.data,
        }
    }
}

impl <DATA: Data> Record<DATA> {
    
    pub fn select(&self) -> &'static str {
        static QUERY: OnceLock::<String> = OnceLock::new();
        QUERY.get_or_init(|| format!("select * from {} where id = $1", DATA::TABLE_NAME))
    }
    
}

pub trait TxRead<Tx, DATA: Data> {
    fn fetch_one_by_id(id: u64, tx: &mut Tx) -> impl Future<Output = Result<Record<DATA>, C3p0Error>> + Send;
}

pub trait TxWrite<Tx, DATA: Data> {
    fn save(self, tx: &mut Tx) -> impl Future<Output = Result<Record<DATA>, C3p0Error>> + Send;
}