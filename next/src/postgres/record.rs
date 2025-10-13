use std::sync::OnceLock;

use sqlx::PgConnection;
use sqlx::Row;
use crate::codec::Codec;
use crate::error::into_c3p0_error;
use crate::time::get_current_epoch_millis;
use crate::{error::C3p0Error, record::{Data, NewRecord, Record, TxRead, TxWrite}};


impl <DATA: Data> TxRead<PgConnection, DATA> for Record<DATA> {

    async fn fetch_one_by_id(id: u64, tx: &mut PgConnection) -> Result<Record<DATA>, C3p0Error> {
        todo!()
    }

}

impl <DATA: Data> TxWrite<PgConnection, DATA> for NewRecord<DATA> {

    async fn save(self, tx: &mut PgConnection) -> Result<Record<DATA>, C3p0Error> {
        static QUERY: OnceLock::<String> = OnceLock::new();
        let query = QUERY.get_or_init(|| format!(
            "INSERT INTO {} (version, create_epoch_millis, update_epoch_millis, data) VALUES ($1, $2, $2, $3) RETURNING id",
            DATA::TABLE_NAME,
        ));

        let data_encoded = DATA::CODEC::encode(self.data);
        let json_data = serde_json::to_value(&data_encoded)?;
        let data = DATA::CODEC::decode(data_encoded);

        let create_epoch_millis = get_current_epoch_millis();

        let id = sqlx::query(&query)
                .bind(0)
                .bind(create_epoch_millis)
                .bind(json_data)
                .fetch_one(tx)
                .await
                .map_err(into_c3p0_error)
                .and_then(|row| {
                    row.try_get(&0)
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("Row contains no values for id index. Err: {err:?}"),
                    })
                    .map(|id: i64| id as u64)
                })?;

        Ok(Record {
            id,
            version: 0,
            data,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
        })
    }

}