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

impl<DATA, CODEC: JsonCodec<DATA>> MongodbC3p0Json<DATA, CODEC>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{


    // #[inline]
    // pub fn to_model(&self, row: &Row) -> Result<Model<DATA>, Box<dyn std::error::Error>> {
    //     to_model(&self.codec, row, 0, 1, 2, 3, 4)
    // }

    // /// Allows the execution of a custom sql query and returns the first entry in the result set.
    // /// For this to work, the sql query:
    // /// - must be a SELECT
    // /// - must declare the ID, VERSION and DATA fields in this exact order
    // pub async fn fetch_one_optional_with_sql(
    //     &self,
    //     tx: &mut MongodbTx,
    //     sql: &str,
    //     params: &[&(dyn ToSql + Sync)],
    // ) -> Result<Option<Model<DATA>>, C3p0Error> {
    //     tx.fetch_one_optional(sql, params, |row| self.to_model(row))
    //         .await
    // }

    // /// Allows the execution of a custom sql query and returns the first entry in the result set.
    // /// For this to work, the sql query:
    // /// - must be a SELECT
    // /// - must declare the ID, VERSION and DATA fields in this exact order
    // pub async fn fetch_one_with_sql(
    //     &self,
    //     tx: &mut MongodbTx,
    //     sql: &str,
    //     params: &[&(dyn ToSql + Sync)],
    // ) -> Result<Model<DATA>, C3p0Error> {
    //     tx.fetch_one(sql, params, |row| self.to_model(row)).await
    // }

    // /// Allows the execution of a custom sql query and returns all the entries in the result set.
    // /// For this to work, the sql query:
    // /// - must be a SELECT
    // /// - must declare the ID, VERSION and DATA fields in this exact order
    // pub async fn fetch_all_with_sql(
    //     &self,
    //     tx: &mut MongodbTx,
    //     sql: &str,
    //     params: &[&(dyn ToSql + Sync)],
    // ) -> Result<Vec<Model<DATA>>, C3p0Error> {
    //     tx.fetch_all(sql, params, |row| self.to_model(row)).await
    // }
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
        let TO_DO = 0;
        Ok(())
    }

    async fn drop_table_if_exists(&self, tx: &mut MongodbTx, cascade: bool) -> Result<(), C3p0Error> {
        let TO_DO = 0;
        Ok(())
    }

    async fn count_all(&self, tx: &mut MongodbTx) -> Result<u64, C3p0Error> {
        tx.db().collection::<Model<DATA>>(&self.table_name).count_documents(None, None).await.map_err(into_c3p0_error)
    }

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        let filter = doc! { "_id": id.into() };
        let options = CountOptions::builder().limit(1).build();
        tx.db().collection::<Model<DATA>>(&self.table_name).count_documents(filter, options).await.map_err(into_c3p0_error).map(|count| count > 0)
    }

    async fn fetch_all(&self, tx: &mut MongodbTx) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let mut cursor = tx.db().collection::<Model<DATA>>(&self.table_name).find(None, None).await.map_err(into_c3p0_error)?;
        let mut result = vec![];
        while cursor.advance().await.map_err(into_c3p0_error)? {
            result.push(cursor.deserialize_current().map_err(into_c3p0_error)?);
        }
        Ok(result)
    }

    async fn fetch_all_for_update(
        &self,
        tx: &mut MongodbTx,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let THIS_METHOD_COULD_BE_REMOVED = 0;
        self.fetch_all(tx).await
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        let filter = doc! {
            "_id": id.into()
        };
        tx.db().collection::<Model<DATA>>(&self.table_name).find_one(filter, None).await.map_err(into_c3p0_error)
    }

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        let THIS_METHOD_COULD_BE_REMOVED = 0;
        self.fetch_one_optional_by_id(tx, id)            .await
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

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error> {
        self.fetch_one_optional_by_id_for_update(tx, id, for_update)
            .await
            .and_then(|result| result.ok_or(C3p0Error::ResultNotFoundError))
    }

    async fn delete(&self, tx: &mut MongodbTx, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {

        let filter = doc! {
            "_id": obj.id,
            "version": obj.version
        };

        let result = tx.db().collection::<Model<DATA>>(&self.table_name).delete_one(filter, None).await.map_err(into_c3p0_error)?;

        if result.deleted_count == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot delete data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.table_name, &obj.id, &obj.version
            )});
        }

        Ok(obj)
    }

    async fn delete_all(&self, tx: &mut MongodbTx) -> Result<u64, C3p0Error> {
        tx.db().collection::<Model<DATA>>(&self.table_name).delete_many(doc! {}, None).await.map_err(into_c3p0_error).map(|result| result.deleted_count)
    }

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        tx: &mut MongodbTx,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        tx.db().collection::<Model<DATA>>(&self.table_name).delete_one(doc! { "_id": id.into() }, None).await.map_err(into_c3p0_error).map(|result| result.deleted_count)
    }

    async fn save(&self, tx: &mut MongodbTx, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().data_to_value(&obj.data)?;
        let create_epoch_millis = get_current_epoch_millis();

        // TODO REMOVE_ME
        let id = rand::random::<u64>();
        let REMOVE_ME = 0;

        let new_model = Model {
            id: id as i64,
            version: obj.version,
            data: obj.data,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
        };

        tx.db().collection::<Model<DATA>>(&self.table_name).insert_one(&new_model, None).await.map_err(into_c3p0_error)?;

        Ok(new_model)
    }

    async fn update(&self, tx: &mut MongodbTx, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let json_data = self.codec().data_to_value(&obj.data)?;
        let previous_version = obj.version;
        let updated_model = obj.into_new_version(get_current_epoch_millis());

        let result = tx.db().collection::<Model<DATA>>(&self.table_name)
            .replace_one(doc! { "_id": updated_model.id, "version": previous_version }, 
            &updated_model, 
            None)
            .await.map_err(into_c3p0_error)?;

        // let result = tx
        //     .execute(
        //         &self.queries.update_sql_query,
        //         &[
        //             &updated_model.version,
        //             &updated_model.update_epoch_millis,
        //             &json_data,
        //             &updated_model.id,
        //             &previous_version,
        //         ],
        //     )
        //     .await?;

        if result.modified_count == 0 {
            return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
                                                                        &self.table_name, &updated_model.id, &previous_version
            )});
        }

        Ok(updated_model)
    }
}
