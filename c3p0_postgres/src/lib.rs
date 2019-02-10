use c3p0::{Model, Jpo};
use postgres::{Connection};

pub struct JpoPg {
    id_field_name: String,
    version_field_name: String,
    data_field_name: String,
    table_name: String,
    conn: Connection
}

impl JpoPg {
    pub fn new<D>(conn: Connection, table_name: &str) -> impl Jpo<D>
    where D: Clone + serde::ser::Serialize + serde::de::DeserializeOwned
    {
        JpoPg{
            conn,
            table_name: table_name.to_owned(),
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
        }
    }

    pub fn new_custom<S: Into<String>, D>(conn: Connection, table_name: S, id_field_name: S,
    version_field_name: S,  data_field_name: S) -> impl Jpo<D>
        where D: Clone + serde::ser::Serialize + serde::de::DeserializeOwned
    {
        JpoPg{
            conn,
            table_name: table_name.into(),
            id_field_name: id_field_name.into(),
            version_field_name: version_field_name.into(),
            data_field_name: data_field_name.into(),
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
        let result = stmt.query(&[&id]).unwrap().iter().next().map(|row| {
            Model{
                id: Some(row.get(0)),
                version: row.get(1),
                data: serde_json::from_value::<DATA>(row.get(2)).unwrap()
            }
        });
        result
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

