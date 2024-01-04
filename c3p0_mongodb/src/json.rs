use std::borrow::Cow;
use std::sync::Arc;

use crate::*;
use ::mongodb::bson::{doc, Bson};
use ::mongodb::bson::oid::ObjectId;
use ::mongodb::options::CountOptions;
use async_trait::async_trait;
use c3p0_common::time::utils::get_current_epoch_millis;
use c3p0_common::*;
use serde_json::Value;

/// A trait that allows the creation of an Id
pub trait IdGenerator<Id: IdType>: Send + Sync {
    fn generate_id(&self) -> Option<Id>;
    fn id_to_db_id<'a>(&self, id: Cow<'a, Id>) -> Result<Bson, C3p0Error>;
    fn db_id_to_id(&self, id: &Bson) -> Result<Id, C3p0Error>;
}

/// An IdGenerator that uses the auto-increment feature of the database
pub struct AutogeneratedIdGenerator {}

impl IdGenerator<ObjectId> for AutogeneratedIdGenerator {
    fn generate_id(&self) -> Option<ObjectId> {
        None
    }

    fn id_to_db_id<'a>(&self, id: Cow<'a, ObjectId>) -> Result<Bson, C3p0Error> {
        Ok(id.as_ref().into())
    }

    // fn db_id_to_id<'a>(&self, id: Bson) -> Result<Cow<'a, ObjectId>, C3p0Error> {
    //     Ok(id)
    // }

    fn db_id_to_id(
        &self,
        id: &mongodb::bson::Bson,
    ) -> Result<ObjectId, C3p0Error> {
        if let mongodb::bson::Bson::ObjectId(id) = id {
            Ok(id.clone())
        } else {
            Err(C3p0Error::RowMapperError {
                cause: "Cannot convert inserted id to ObjectId".into(),
            })
        }
    }
}

/// An IdGenerator that uses the uuid crate to generate a random uuid
pub struct UuidIdGenerator {}

impl IdGenerator<uuid::Uuid> for UuidIdGenerator {
    fn generate_id(&self) -> Option<uuid::Uuid> {
        Some(uuid::Uuid::new_v4())
    }

    fn id_to_db_id<'a>(
        &self,
        id: Cow<'a, uuid::Uuid>,
    ) -> Result<Bson, C3p0Error> {
        let id = id.into_owned();
        Ok(mongodb::bson::Uuid::from(id).into())
    }

    fn db_id_to_id<'a>(
        &self,
        id: &'a Bson,
    ) -> Result<uuid::Uuid, C3p0Error> {
        match id {
            mongodb::bson::Bson::Binary(binary) => {
                if let mongodb::bson::spec::BinarySubtype::Uuid = binary.subtype {
                    return binary.clone()
                        .bytes
                        .try_into()
                        .map_err(|err| C3p0Error::RowMapperError {
                            cause: format!("Cannot convert inserted id to Uuid: {:?}", err),
                        });
                }
            }
            mongodb::bson::Bson::String(string) => {
                return uuid::Uuid::parse_str(&string).map_err(|err| C3p0Error::RowMapperError {
                    cause: format!("Cannot convert inserted id to Uuid: {:?}", err),
                });
            }
            _ => {}
        };
        Err(C3p0Error::RowMapperError {
            cause: "Cannot convert inserted id to Uuid: Unexpected type".into(),
        })
    }

}

#[derive(Clone)]
pub struct MongodbC3p0JsonBuilder<Id: IdType> {
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

impl<Id: IdType> MongodbC3p0JsonBuilder<Id> {
    pub fn with_id_generator<
        NewId: IdType,
        T: 'static + IdGenerator<NewId> + Send + Sync,
    >(
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
pub struct MongodbC3p0Json<Id: IdType, Data: DataType, CODEC: JsonCodec<Data>>
{
    phantom_data: std::marker::PhantomData<Data>,
    id_generator: Arc<dyn IdGenerator<Id>>,
    codec: CODEC,
    table_name: String,
}

#[async_trait]
impl<Id: IdType, Data: DataType, CODEC: JsonCodec<Data>>
    C3p0Json<Id, Data, CODEC> for MongodbC3p0Json<Id, Data, CODEC>
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
        db.collection::<ModelWithId>(&self.table_name)
            .count_documents_with_session(None, None, session)
            .await
            .map_err(into_c3p0_error)
    }

    async fn exists_by_id<'a, ID: Into<&'a Id> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        let filter =
            doc! { "_id": self.id_generator.id_to_db_id(Cow::Borrowed(id.into()))? };
        let options = CountOptions::builder().limit(1).build();
        let (db, session) = tx.db();
        db.collection::<ModelWithId>(&self.table_name)
            .count_documents_with_session(filter, options, session)
            .await
            .map_err(into_c3p0_error)
            .map(|count| count > 0)
    }

    async fn fetch_all(&self, tx: &mut MongodbTx) -> Result<Vec<Model<Id, Data>>, C3p0Error> {
        let (db, session) = tx.db();
        let mut cursor = db
            .collection::<ModelWithId>(&self.table_name)
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
                    .and_then(|model| model.into_model(self.id_generator.as_ref(), codec))?,
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
            "_id": self.id_generator.id_to_db_id(Cow::Borrowed(id.into()))?
        };
        let (db, session) = tx.db();
        let model = db
            .collection::<ModelWithId>(&self.table_name)
            .find_one_with_session(filter, None, session)
            .await
            .map_err(into_c3p0_error)?;
        if let Some(model) = model {
            Ok(Some(
                model.into_model(self.id_generator.as_ref(), &self.codec)?,
            ))
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
            "_id": self.id_generator.id_to_db_id(Cow::Borrowed(&obj.id))?,
            "version": obj.version
        };
        let (db, session) = tx.db();
        let result = db
            .collection::<ModelWithId>(&self.table_name)
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
        db.collection::<ModelWithId>(&self.table_name)
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
        db.collection::<ModelWithId>(&self.table_name)
            .delete_one_with_session(
                doc! {
                    "_id":  self.id_generator.id_to_db_id(Cow::Borrowed(id.into()))?
                },
                None,
                session,
            )
            .await
            .map_err(into_c3p0_error)
            .map(|result| result.deleted_count)
    }

    async fn save(
        &self,
        tx: &mut MongodbTx,
        obj: NewModel<Data>,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        let json_data = self.codec.data_to_value(&obj.data)?;
        let create_epoch_millis = get_current_epoch_millis();

        let (db, session) = tx.db();

        let new_model = if let Some(id) = self.id_generator.generate_id() {
            let param_id = self.id_generator.id_to_db_id(Cow::Owned(id.clone()))?;
            let new_model = ModelWithId {
                id: param_id,
                version: obj.version,
                data: json_data,
                create_epoch_millis,
                update_epoch_millis: create_epoch_millis,
            };
            db.collection::<ModelWithId>(&self.table_name)
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

            ModelWithId {
                id: result.inserted_id,
                version: new_model.version,
                data: new_model.data,
                create_epoch_millis: new_model.create_epoch_millis,
                update_epoch_millis: new_model.update_epoch_millis,
            }
        };

        Ok(new_model.into_model(self.id_generator.as_ref(), &self.codec)?)
    }

    async fn update(
        &self,
        tx: &mut MongodbTx,
        obj: Model<Id, Data>,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        let previous_version = obj.version;
        let obj_id = obj.id.clone();
        let updated_model = obj.into_new_version(get_current_epoch_millis());
        let updated_model =
            ModelWithId::from_model(updated_model, self.id_generator.as_ref(), &self.codec)?;

        let (db, session) = tx.db();
        let result = db
            .collection::<ModelWithId>(&self.table_name)
            .replace_one_with_session(
                doc! { "_id": &updated_model.id, "version": previous_version },
                &updated_model,
                None,
                session,
            )
            .await
            .map_err(into_c3p0_error)?;

        if result.modified_count == 0 {
            return Err(C3p0Error::OptimisticLockError{ cause: format!("Cannot update data in table [{}] with id [{:?}], version [{}]: data was changed!",
                                                                        &self.table_name, &obj_id, &previous_version
            )});
        }

        Ok(updated_model.into_model(self.id_generator.as_ref(), &self.codec)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ModelWithoutId {
    pub version: VersionType,
    pub create_epoch_millis: EpochMillisType,
    pub update_epoch_millis: EpochMillisType,
    pub data: Value,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ModelWithId {
    #[serde(rename = "_id")]
    pub id: Bson,
    pub version: VersionType,
    pub create_epoch_millis: EpochMillisType,
    pub update_epoch_millis: EpochMillisType,
    pub data: Value,
}

impl ModelWithId {
    fn into_model<Id: IdType, Data: DataType, Codec: JsonCodec<Data>>(
        self,
        id_generator: &(dyn IdGenerator<Id>),
        codec: &Codec,
    ) -> Result<Model<Id, Data>, C3p0Error> {
        Ok(Model {
            id: id_generator.db_id_to_id(&self.id)?,
            version: self.version,
            create_epoch_millis: self.create_epoch_millis,
            update_epoch_millis: self.update_epoch_millis,
            data: codec.data_from_value(self.data)?,
        })
    }

    fn from_model<Id: IdType, Data: DataType, Codec: JsonCodec<Data>>(
        model: Model<Id, Data>,
        id_generator: &(dyn IdGenerator<Id>),
        codec: &Codec,
    ) -> Result<Self, C3p0Error> {
        Ok(ModelWithId {
            id: id_generator.id_to_db_id(Cow::Owned(model.id))?,
            version: model.version,
            create_epoch_millis: model.create_epoch_millis,
            update_epoch_millis: model.update_epoch_millis,
            data: codec.data_to_value(&model.data)?,
        })
    }
}
