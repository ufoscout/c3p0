use c3p0::{Model, Jpo};
use postgres::{Connection};
use postgres::rows::Row;

pub struct JpoPg {
    id_field_name: String,
    version_field_name: String,
    data_field_name: String,
    table_name: String,
    conn: Connection
}

impl JpoPg {
    pub fn build<DATA>(conn: Connection, table_name: &str) -> impl Jpo<DATA>
    where DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned
    {
        JpoPg{
            conn,
            table_name: table_name.to_owned(),
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
        }
    }

    pub fn build_custom<S: Into<String>, DATA>(conn: Connection, table_name: S, id_field_name: S,
    version_field_name: S,  data_field_name: S) -> impl Jpo<DATA>
        where DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned
    {
        JpoPg{
            conn,
            table_name: table_name.into(),
            id_field_name: id_field_name.into(),
            version_field_name: version_field_name.into(),
            data_field_name: data_field_name.into(),
        }
    }

    pub fn to_model<DATA>(&self, row: Row) -> Model<DATA>
        where DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned
    {
        Model{
            //id: Some(row.get(self.id_field_name.as_str())),
            //version: row.get(self.version_field_name.as_str()),
            //data: serde_json::from_value::<DATA>(row.get(self.data_field_name.as_str())).unwrap()
            id: Some(row.get(0)),
            version: row.get(1),
            data: serde_json::from_value::<DATA>(row.get(2)).unwrap()
        }
    }
}

impl <DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned> Jpo<DATA> for JpoPg {

    fn find_by_id(&self, id: i64) -> Option<Model<DATA>> {
        let query = format!("SELECT {}, {}, {} FROM {} WHERE {} = $1",
                            self.id_field_name,
                            self.version_field_name,
                            self.data_field_name,
                            self.table_name,
                            self.id_field_name,
        );
        let stmt = self.conn.prepare(&query).unwrap();
        stmt.query(&[&id]).unwrap().iter().next().map(|row| self.to_model(row))
    }

    fn save(&self, obj: &Model<DATA>) -> Model<DATA> {
        let query = format!("INSERT INTO {} ({}, {}) VALUES ($1, $2) RETURNING {}",
            self.table_name,
            self.version_field_name,
            self.data_field_name,
            self.id_field_name
        );
        let stmt = self.conn.prepare(&query).unwrap();
        let json_data = serde_json::to_value(&obj.data).unwrap();
        let id: i64 = stmt.query(&[&obj.version, &json_data]).unwrap().iter().next().unwrap().get(0);

        let mut clone = obj.clone();
        clone.id = Some(id);
        clone
    }
}

