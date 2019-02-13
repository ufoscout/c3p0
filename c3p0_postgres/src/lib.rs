use c3p0::{C3p0Model};
use postgres::{Connection};
use postgres::rows::Row;

pub struct Conf<DATA, M: C3p0Model<DATA>>
    where DATA: serde::ser::Serialize + serde::de::DeserializeOwned
{
    pub id_field_name: String,
    pub version_field_name: String,
    pub data_field_name: String,
    pub table_name: String,
    pub conn: Connection,
    pub model_factory: fn(Option<i64>, i32, DATA) -> M
}

impl <DATA, M: C3p0Model<DATA>> Conf<DATA, M>
    where DATA: serde::ser::Serialize + serde::de::DeserializeOwned
{
    pub fn build<T>(conn: Connection, table_name: T, factory: fn(Option<i64>, i32, DATA) -> M) -> Self
        where T: Into<String>
    {
        Conf::build_custom(conn, table_name, "id", "version", "data", factory)
    }

    pub fn build_custom<T, I, V, D>(conn: Connection, table_name: T, id_field_name: I,
                                    version_field_name: V, data_field_name: D, factory: fn(Option<i64>, i32, DATA) -> M) -> Self
        where T: Into<String>, I: Into<String>, V: Into<String>, D: Into<String>
    {
        Conf {
            conn,
            table_name: table_name.into(),
            id_field_name: id_field_name.into(),
            version_field_name: version_field_name.into(),
            data_field_name: data_field_name.into(),
            model_factory: factory,
        }
    }
}

pub trait JpoPg<DATA, M: C3p0Model<DATA>>
    where DATA: serde::ser::Serialize + serde::de::DeserializeOwned
{
    fn conf(&self) -> &Conf<DATA, M>;

    fn to_model(&self, row: Row) -> M
    {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: serde_json::from_value::<DATA>(row.get(self.data_field_name.as_str())).unwrap()
        let id = Some(row.get(0));
        let version = row.get(1);
        let data = serde_json::from_value::<DATA>(row.get(2)).unwrap();
        (self.conf().model_factory)(id, version, data)

    }

    fn find_by_id(&self, id: i64) -> Option<M> {
        let conf = self.conf();
        let query = format!("SELECT {}, {}, {} FROM {} WHERE {} = $1",
                            conf.id_field_name,
                            conf.version_field_name,
                            conf.data_field_name,
                            conf.table_name,
                            conf.id_field_name,
        );
        let stmt = conf.conn.prepare(&query).unwrap();
        stmt.query(&[&id]).unwrap().iter().next().map(|row| self.to_model(row))
    }

    fn save(&self, obj: M) -> M {
        let conf = self.conf();
        let query = format!("INSERT INTO {} ({}, {}) VALUES ($1, $2) RETURNING {}",
                            conf.table_name,
                            conf.version_field_name,
                            conf.data_field_name,
                            conf.id_field_name
        );
        let stmt = conf.conn.prepare(&query).unwrap();
        let json_data = serde_json::to_value(obj.c3p0_get_data()).unwrap();
        let id: i64 = stmt.query(&[obj.c3p0_get_version(), &json_data]).unwrap().iter().next().unwrap().get(0);

        obj.c3p0_clone_with_id(id)
    }
}

pub struct SimpleRepository<DATA, M: C3p0Model<DATA>>
    where DATA: serde::ser::Serialize + serde::de::DeserializeOwned {
    conf: Conf<DATA, M>,
}

impl <DATA, M: C3p0Model<DATA>> SimpleRepository<DATA, M>
    where DATA: serde::ser::Serialize + serde::de::DeserializeOwned {

    pub fn build<T>(conn: Connection, table_name: T, factory: fn(Option<i64>, i32, DATA) -> M) -> Self
        where T: Into<String>
    {
        SimpleRepository::build_with_conf(Conf::build(conn, table_name, factory))
    }

    pub fn build_custom<T, I, V, D>(conn: Connection, table_name: T, id_field_name: I,
                                    version_field_name: V, data_field_name: D, factory: fn(Option<i64>, i32, DATA) -> M) -> Self
        where T: Into<String>, I: Into<String>, V: Into<String>, D: Into<String>
    {
        SimpleRepository::build_with_conf(Conf::build_custom(conn, table_name, id_field_name, version_field_name, data_field_name, factory))
    }

    pub fn build_with_conf(conf: Conf<DATA, M>) -> Self
    {
        SimpleRepository{conf}
    }
}

impl <DATA, M: C3p0Model<DATA>> JpoPg<DATA, M> for SimpleRepository<DATA, M>
    where DATA: serde::ser::Serialize + serde::de::DeserializeOwned {
    fn conf(&self) -> &Conf<DATA, M> {
        &self.conf
    }
}