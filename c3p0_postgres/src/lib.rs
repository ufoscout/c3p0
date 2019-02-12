use c3p0::{C3p0Model, Jpo};
use postgres::{Connection};
use postgres::rows::Row;
use std::marker::PhantomData;

pub struct JpoPg<DATA, M: C3p0Model<DATA>, F>
    where DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
    F: Fn(Option<i64>, i32, DATA) -> M
{
    id_field_name: String,
    version_field_name: String,
    data_field_name: String,
    table_name: String,
    conn: Connection,
    model_factory: F,
    _phantom_data: PhantomData<DATA>,
    _phantom_m: PhantomData<M>,
}

impl <DATA, M: C3p0Model<DATA>, F> JpoPg<DATA, M, F>
    where DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
    F: Fn(Option<i64>, i32, DATA) -> M  {

    pub fn build<T>(conn: Connection, table_name: T, factory: F) -> impl Jpo<DATA, M>
        where T: Into<String>
    {
        JpoPg::build_custom(conn, table_name, "id", "version", "data", factory)
    }

    pub fn build_custom<T, I, V, D>(conn: Connection, table_name: T, id_field_name: I,
    version_field_name: V,  data_field_name: D, factory: F) -> impl Jpo<DATA, M>
        where T: Into<String>, I: Into<String>, V: Into<String>, D: Into<String>
    {
        JpoPg{
            conn,
            table_name: table_name.into(),
            id_field_name: id_field_name.into(),
            version_field_name: version_field_name.into(),
            data_field_name: data_field_name.into(),
            model_factory: factory,
            _phantom_data: PhantomData,
            _phantom_m: PhantomData,
        }
    }

    pub fn to_model(&self, row: Row) -> M
    {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: serde_json::from_value::<DATA>(row.get(self.data_field_name.as_str())).unwrap()
        let id = Some(row.get(0));
        let version = row.get(1);
        let data = serde_json::from_value::<DATA>(row.get(2)).unwrap();
        (self.model_factory)(id, version, data)

    }
}

impl <DATA, M: C3p0Model<DATA>, F> Jpo<DATA, M> for JpoPg<DATA, M, F>
    where DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
          F: Fn(Option<i64>, i32, DATA) -> M
{
    fn find_by_id(&self, id: i64) -> Option<M> {
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

    fn save(&self, obj: M) -> M {
        let query = format!("INSERT INTO {} ({}, {}) VALUES ($1, $2) RETURNING {}",
            self.table_name,
            self.version_field_name,
            self.data_field_name,
            self.id_field_name
        );
        let stmt = self.conn.prepare(&query).unwrap();
        let json_data = serde_json::to_value(obj.c3p0_get_data()).unwrap();
        let id: i64 = stmt.query(&[obj.c3p0_get_version(), &json_data]).unwrap().iter().next().unwrap().get(0);

        obj.c3p0_clone_with_id(id)
    }
}

