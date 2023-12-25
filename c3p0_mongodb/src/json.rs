use std::sync::Arc;

use crate::*;
use ::mongodb::bson::doc;
use ::mongodb::bson::oid::ObjectId;
use ::mongodb::options::CountOptions;
use async_trait::async_trait;
use c3p0_common::time::utils::get_current_epoch_millis;
use c3p0_common::*;
use serde_json::Value;

pub trait MongodbIdType: IdType + Into<mongodb::bson::Bson> {}
impl<T: IdType + Into<mongodb::bson::Bson>> MongodbIdType for T {}

/// A trait that allows the creation of an Id
pub trait IdGenerator<Id>: Send + Sync {
    fn generate_id(&self) -> Option<Id>;
}

/// An IdGenerator that uses the auto-increment feature of the database
pub struct AutogeneratedIdGenerator {}

impl IdGenerator<ObjectId> for AutogeneratedIdGenerator {
    fn generate_id(&self) -> Option<ObjectId> {
        None
    }
}

/// An IdGenerator that uses the uuid crate to generate a random uuid
pub struct UuidIdGenerator {}

impl IdGenerator<uuid::Uuid> for UuidIdGenerator {
    fn generate_id(&self) -> Option<uuid::Uuid> {
        Some(uuid::Uuid::new_v4())
    }
}

#[derive(Clone)]
pub struct MongodbC3p0JsonBuilder<Id> {
    pub id_generator: Arc<dyn IdGenerator<Id>>,
    pub table_name: String,
}

impl MongodbC3p0JsonBuilder<ObjectId> {
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        MongodbC3p0JsonBuilder {
            id_generator: Arc::new(AutogeneratedIdGenerator {}),
            table_name,
        }
    }
}

impl<Id> MongodbC3p0JsonBuilder<Id>
where
    Id: MongodbIdType,
{
    pub fn with_id_generator<NewId, T: 'static + IdGenerator<NewId> + Send + Sync>(
        self,
        id_generator: T,
    ) -> MongodbC3p0JsonBuilder<NewId> {
        MongodbC3p0JsonBuilder {
            id_generator: Arc::new(id_generator),
            table_name: self.table_name,
        }
    }

    pub fn build<Data: DataType>(self) -> MongodbC3p0Json<Id, Data, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    pub fn build_with_codec<Data: DataType, CODEC: JsonCodec<Data>>(
        self,
        codec: CODEC,
    ) -> MongodbC3p0Json<Id, Data, CODEC> {
        MongodbC3p0Json {
            phantom_data: std::marker::PhantomData,
            id_generator: self.id_generator.clone(),
            codec,
            table_name: self.table_name,
        }
    }
}

#[derive(Clone)]
pub struct MongodbC3p0Json<Id, Data: DataType, CODEC: JsonCodec<Data>>
where
    Id: MongodbIdType,
{
    phantom_data: std::marker::PhantomData<Data>,
    id_generator: Arc<dyn IdGenerator<Id>>,
    codec: CODEC,
    table_name: String,
}

#[async_trait]
impl<Id, Data: DataType, CODEC: JsonCodec<Data>> C3p0Json<Id, Data, CODEC>
    for MongodbC3p0Json<Id, Data, CODEC>
where
    Id: MongodbIdType,
{
    type Tx = MongodbTx;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(&self, tx: &mut MongodbTx) -> Result<(), C3p0Error> {
        // perform a simple query will create the table if it does not exist
        self.count_all(tx).await.map(|_| ())
    }

    async fn drop_table_if_exists(
        &self,
        _tx: &mut MongodbTx,
        _cascade: bool,
    ) -> Result<(), C3p0Error> {
        // Cannot drop collection with session because it is not supported by mongodb
        Err(C3p0Error::OperationNotSupported {
            cause: "Cannot drop collection with session because it is not supported by mongodb"
                .into(),
        })
    }

    async fn count_all(&self, tx: &mut MongodbTx) -> Result<u64, C3p0Error> {
        let (db, session) = tx.db();
        db.collection::<ModelWithId<Id>>(&self.table_name)
            .count_documents_with_session(None, None, session)
            .await
            .map_err(into_c3p0_error)
    }

    async fn exists_by_id<'a, ID: Into<&'a Id> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        let filter = doc! { "_id": id.into() };
        let options = CountOptions::builder().limit(1).build();
        let (db, session) = tx.db();
        db.collection::<ModelWithId<Id>>(&self.table_name)
            .count_documents_with_session(filter, options, session)
            .await
            .map_err(into_c3p0_error)
            .map(|count| count > 0)
    }

    async fn fetch_all(&self, tx: &mut MongodbTx) -> Result<Vec<Model<Id, Data>>, C3p0Error> {
        let (db, session) = tx.db();
        let mut cursor = db
            .collection::<ModelWithId<Id>>(&self.table_name)
            .find_with_session(None, None, session)
            .await
            .map_err(into_c3p0_error)?;
        let mut result = vec![];
        let codec = &self.codec;
        while cursor.advance(session).await.map_err(into_c3p0_error)? {
            result.push(
                cursor
                    .deserialize_current()
                    .map_err(into_c3p0_error)
                    .and_then(|model| model.to_model(codec))?,
            );
        }
        Ok(result)
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a Id> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<Option<Model<Id, Data>>, C3p0Error> {
        let filter = doc! {
            "_id": id.into()
        };
        let (db, session) = tx.db();
        let model = db
            .collection::<ModelWithId<Id>>(&self.table_name)
            .find_one_with_session(filter, None, session)
            .await
            .map_err(into_c3p0_error)?;
        if let Some(model) = model {
            Ok(Some(model.to_model(&self.codec)?))
        } else {
            Ok(None)
        }
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a Id> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        self.fetch_one_optional_by_id(tx, id)
            .await
            .and_then(|result| result.ok_or(C3p0Error::ResultNotFoundError))
    }

    async fn delete(
        &self,
        tx: &mut MongodbTx,
        obj: Model<Id, Data>,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        let filter = doc! {
            "_id": obj.id.clone().into(),
            "version": obj.version
        };
        let (db, session) = tx.db();
        let result = db
            .collection::<ModelWithId<Id>>(&self.table_name)
            .delete_one_with_session(filter, None, session)
            .await
            .map_err(into_c3p0_error)?;

        if result.deleted_count == 0 {
            return Err(C3p0Error::OptimisticLockError{ cause: format!("Cannot delete data in table [{}] with id [{:?}], version [{}]: data was changed!",
                                                                        &self.table_name, &obj.id, &obj.version
            )});
        }

        Ok(obj)
    }

    async fn delete_all(&self, tx: &mut MongodbTx) -> Result<u64, C3p0Error> {
        let (db, session) = tx.db();
        db.collection::<ModelWithId<Id>>(&self.table_name)
            .delete_many_with_session(doc! {}, None, session)
            .await
            .map_err(into_c3p0_error)
            .map(|result| result.deleted_count)
    }

    async fn delete_by_id<'a, ID: Into<&'a Id> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        let (db, session) = tx.db();
        db.collection::<ModelWithId<Id>>(&self.table_name)
            .delete_one_with_session(doc! { "_id": id.into() }, None, session)
            .await
            .map_err(into_c3p0_error)
            .map(|result| result.deleted_count)
    }

    async fn save(
        &self,
        tx: &mut MongodbTx,
        obj: NewModel<Data>,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        let json_data = self.codec().data_to_value(&obj.data)?;
        let create_epoch_millis = get_current_epoch_millis();

        let (db, session) = tx.db();

        let new_model = if let Some(id) = self.id_generator.generate_id() {
            let new_model = ModelWithId {
                id,
                version: obj.version,
                data: json_data,
                create_epoch_millis,
                update_epoch_millis: create_epoch_millis,
            };
            db.collection::<ModelWithId<Id>>(&self.table_name)
                .insert_one_with_session(&new_model, None, session)
                .await
                .map_err(into_c3p0_error)?;
            new_model
        } else {
            let new_model = ModelWithoutId {
                version: 0,
                data: json_data,
                create_epoch_millis,
                update_epoch_millis: create_epoch_millis,
            };
            let result = db
                .collection::<ModelWithoutId>(&self.table_name)
                .insert_one_with_session(&new_model, None, session)
                .await
                .map_err(into_c3p0_error)?;
            let id: Id = serde_json::from_value(result.inserted_id.into_relaxed_extjson())?;
            ModelWithId {
                id,
                version: new_model.version,
                data: new_model.data,
                create_epoch_millis: new_model.create_epoch_millis,
                update_epoch_millis: new_model.update_epoch_millis,
            }
        };

        Ok(new_model.to_model(&self.codec)?)
    }

    async fn update(
        &self,
        tx: &mut MongodbTx,
        obj: Model<Id, Data>,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        let previous_version = obj.version;
        let updated_model = obj.into_new_version(get_current_epoch_millis());
        let updated_model = ModelWithId::from_model(updated_model, &self.codec)?;

        let (db, session) = tx.db();
        let result = db
            .collection::<ModelWithId<Id>>(&self.table_name)
            .replace_one_with_session(
                doc! { "_id": updated_model.id.clone().into(), "version": previous_version },
                &updated_model,
                None,
                session,
            )
            .await
            .map_err(into_c3p0_error)?;

        if result.modified_count == 0 {
            return Err(C3p0Error::OptimisticLockError{ cause: format!("Cannot update data in table [{}] with id [{:?}], version [{}]: data was changed!",
                                                                        &self.table_name, &updated_model.id, &previous_version
            )});
        }

        Ok(updated_model.to_model(&self.codec)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ModelWithoutId {
    pub version: VersionType,
    pub create_epoch_millis: EpochMillisType,
    pub update_epoch_millis: EpochMillisType,
    pub data: Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ModelWithId<Id> {
    #[serde(rename = "_id")]
    pub id: Id,
    pub version: VersionType,
    pub create_epoch_millis: EpochMillisType,
    pub update_epoch_millis: EpochMillisType,
    pub data: Value,
}

impl<Id: MongodbIdType> ModelWithId<Id> {
    fn to_model<Data: DataType, Codec: JsonCodec<Data>>(
        self,
        codec: &Codec,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        Ok(Model {
            id: self.id,
            version: self.version,
            create_epoch_millis: self.create_epoch_millis,
            update_epoch_millis: self.update_epoch_millis,
            data: codec.data_from_value(self.data)?,
        })
    }

    fn from_model<Data: DataType, Codec: JsonCodec<Data>>(
        model: Model<Id, Data>,
        codec: &Codec,
    ) -> Result<Self, C3p0Error> {
        Ok(ModelWithId {
            id: model.id,
            version: model.version,
            create_epoch_millis: model.create_epoch_millis,
            update_epoch_millis: model.update_epoch_millis,
            data: codec.data_to_value(&model.data)?,
        })
    }
}
