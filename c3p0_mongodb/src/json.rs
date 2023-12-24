use crate::*;
use async_trait::async_trait;
use bson::doc;
use c3p0_common::time::utils::get_current_epoch_millis;
use c3p0_common::*;
use ::mongodb::options::CountOptions;

pub trait MongodbC3p0JsonBuilder {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> MongodbC3p0Json<DATA, DefaultJsonCodec>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> MongodbC3p0Json<DATA, CODEC>;
}

impl MongodbC3p0JsonBuilder for C3p0JsonBuilder<MongodbC3p0Pool> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> MongodbC3p0Json<DATA, DefaultJsonCodec> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> MongodbC3p0Json<DATA, CODEC> {
        MongodbC3p0Json {
            phantom_data: std::marker::PhantomData,
            codec,
            table_name: self.table_name,
        }
    }
}

#[derive(Clone)]
pub struct MongodbC3p0Json<DATA, CODEC: JsonCodec<DATA>>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    phantom_data: std::marker::PhantomData<DATA>,

    codec: CODEC,
    table_name: String,
}

#[async_trait]
impl<DATA, CODEC: JsonCodec<DATA>> C3p0Json<DATA, CODEC> for MongodbC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin,
{
    type Tx = MongodbTx;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(&self, tx: &mut MongodbTx) -> Result<(), C3p0Error> {
        // perform a simple query will create the table if it does not exist
        self.count_all(tx).await.map(|_| ())
    }

    async fn drop_table_if_exists(&self, _tx: &mut MongodbTx, _cascade: bool) -> Result<(), C3p0Error> {
        // Cannot drop collection with session because it is not supported by mongodb
        Err(C3p0Error::OperationNotSupported { cause: "Cannot drop collection with session because it is not supported by mongodb".into() })
    }

    async fn count_all(&self, tx: &mut MongodbTx) -> Result<u64, C3p0Error> {
        let (db, session) = tx.db();
        db.collection::<Model<DATA>>(&self.table_name).count_documents_with_session(None, None, session).await.map_err(into_c3p0_error)
    }

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        let filter = doc! { "_id": id.into() };
        let options = CountOptions::builder().limit(1).build();
        let (db, session) = tx.db();
        db.collection::<Model<DATA>>(&self.table_name).count_documents_with_session(filter, options, session).await.map_err(into_c3p0_error).map(|count| count > 0)
    }

    async fn fetch_all(&self, tx: &mut MongodbTx) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let (db, session) = tx.db();
        let mut cursor = db.collection::<Model<DATA>>(&self.table_name).find_with_session(None, None, session).await.map_err(into_c3p0_error)?;
        let mut result = vec![];
        while cursor.advance(session).await.map_err(into_c3p0_error)? {
            result.push(cursor.deserialize_current().map_err(into_c3p0_error)?);
        }
        Ok(result)
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        let filter = doc! {
            "_id": id.into()
        };
        let (db, session) = tx.db();
        db.collection::<Model<DATA>>(&self.table_name).find_one_with_session(filter, None, session).await.map_err(into_c3p0_error)
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_optional_by_id(tx, id)
            .await
            .and_then(|result| result.ok_or(C3p0Error::ResultNotFoundError))
    }

    async fn delete(&self, tx: &mut MongodbTx, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {

        let filter = doc! {
            "_id": obj.id,
            "version": obj.version
        };
        let (db, session) = tx.db();
        let result = db.collection::<Model<DATA>>(&self.table_name).delete_one_with_session(filter, None, session).await.map_err(into_c3p0_error)?;

        if result.deleted_count == 0 {
            return Err(C3p0Error::OptimisticLockError{ cause: format!("Cannot delete data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.table_name, &obj.id, &obj.version
            )});
        }

        Ok(obj)
    }

    async fn delete_all(&self, tx: &mut MongodbTx) -> Result<u64, C3p0Error> {
        let (db, session) = tx.db();
        db.collection::<Model<DATA>>(&self.table_name).delete_many_with_session(doc! {}, None, session).await.map_err(into_c3p0_error).map(|result| result.deleted_count)
    }

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        let (db, session) = tx.db();
        db.collection::<Model<DATA>>(&self.table_name).delete_one_with_session(doc! { "_id": id.into() }, None, session).await.map_err(into_c3p0_error).map(|result| result.deleted_count)
    }

    async fn save(&self, tx: &mut MongodbTx, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().data_to_value(&obj.data)?;
        let create_epoch_millis = get_current_epoch_millis();

        // TODO REMOVE_ME
        let id = rand::random::<u32>();
        let REMOVE_ME = 0;

        let new_model = Model {
            id: id as i64,
            version: obj.version,
            data: obj.data,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
        };

        let (db, session) = tx.db();
        db.collection::<Model<DATA>>(&self.table_name).insert_one_with_session(&new_model, None, session).await.map_err(into_c3p0_error)?;

        Ok(new_model)
    }

    async fn update(&self, tx: &mut MongodbTx, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().data_to_value(&obj.data)?;
        let previous_version = obj.version;
        let updated_model = obj.into_new_version(get_current_epoch_millis());

        let (db, session) = tx.db();
        let result = db.collection::<Model<DATA>>(&self.table_name)
            .replace_one_with_session(doc! { "_id": updated_model.id, "version": previous_version }, 
            &updated_model, 
            None, session)
            .await.map_err(into_c3p0_error)?;

        if result.modified_count == 0 {
            return Err(C3p0Error::OptimisticLockError{ cause: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.table_name, &updated_model.id, &previous_version
            )});
        }

        Ok(updated_model)
    }
}
