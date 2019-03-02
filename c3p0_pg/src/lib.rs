use postgres::rows::Row;
use postgres::Connection;

pub trait C3p0Model<DATA>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn c3p0_get_id(&self) -> &Option<i64>;
    fn c3p0_get_version(&self) -> &i32;
    fn c3p0_get_data(&self) -> &DATA;
    fn c3p0_clone_with_id<ID: Into<Option<i64>>>(self, id: ID) -> Self;
}

#[derive(Clone)]
pub struct Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub id: Option<i64>,
    pub version: i32,
    pub data: DATA,
}

impl<DATA> C3p0Model<DATA> for Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn c3p0_get_id(&self) -> &Option<i64> {
        &self.id
    }

    fn c3p0_get_version(&self) -> &i32 {
        &self.version
    }

    fn c3p0_get_data(&self) -> &DATA {
        &self.data
    }

    fn c3p0_clone_with_id<ID: Into<Option<i64>>>(self, id: ID) -> Self {
        Model {
            id: id.into(),
            version: self.version,
            data: self.data,
        }
    }
}

impl<DATA> Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new<ID: Into<Option<i64>>>(id: ID, version: i32, data: DATA) -> Model<DATA> {
        Model {
            id: id.into(),
            version,
            data,
        }
    }

    pub fn new_with_data(data: DATA) -> Model<DATA> {
        Model {
            id: None,
            version: 0,
            data,
        }
    }
}

pub struct Conf<DATA, M: C3p0Model<DATA>>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub id_field_name: String,
    pub version_field_name: String,
    pub data_field_name: String,
    pub table_name: String,
    pub conn: Connection,
    pub model_factory: fn(Option<i64>, i32, DATA) -> M,

    pub find_by_id_sql_query: String,
    pub save_sql_query: String,
}

impl<DATA, M: C3p0Model<DATA>> Conf<DATA, M>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn build<T>(
        conn: Connection,
        table_name: T,
        factory: fn(Option<i64>, i32, DATA) -> M,
    ) -> Self
    where
        T: Into<String>,
    {
        Conf::build_custom(conn, table_name, "id", "version", "data", factory)
    }

    pub fn build_custom<T, I, V, D>(
        conn: Connection,
        table_name: T,
        id_field_name: I,
        version_field_name: V,
        data_field_name: D,
        factory: fn(Option<i64>, i32, DATA) -> M,
    ) -> Self
    where
        T: Into<String>,
        I: Into<String>,
        V: Into<String>,
        D: Into<String>,
    {
        let table_name = table_name.into();
        let id_field_name = id_field_name.into();
        let version_field_name = version_field_name.into();
        let data_field_name = data_field_name.into();
        Conf {
            conn,
            table_name: table_name.clone(),
            id_field_name: id_field_name.clone(),
            version_field_name: version_field_name.clone(),
            data_field_name: data_field_name.clone(),
            model_factory: factory,

            find_by_id_sql_query: format!(
                "SELECT {}, {}, {} FROM {} WHERE {} = $1",
                id_field_name.clone(),
                version_field_name.clone(),
                data_field_name.clone(),
                table_name.clone(),
                id_field_name.clone(),
            ),

            save_sql_query: format!(
                "INSERT INTO {} ({}, {}) VALUES ($1, $2) RETURNING {}",
                table_name.clone(),
                version_field_name.clone(),
                data_field_name.clone(),
                id_field_name.clone()
            ),
        }
    }
}

pub trait JpoPg<DATA, M: C3p0Model<DATA>>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn conf(&self) -> &Conf<DATA, M>;

    fn to_model(&self, row: Row) -> M {
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
        let stmt = conf.conn.prepare(&conf.find_by_id_sql_query).unwrap();
        stmt.query(&[&id])
            .unwrap()
            .iter()
            .next()
            .map(|row| self.to_model(row))
    }

    fn save(&self, obj: M) -> M {
        let conf = self.conf();
        let stmt = conf.conn.prepare(&conf.save_sql_query).unwrap();
        let json_data = serde_json::to_value(obj.c3p0_get_data()).unwrap();
        let id: i64 = stmt
            .query(&[obj.c3p0_get_version(), &json_data])
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .get(0);

        obj.c3p0_clone_with_id(id)
    }
}

pub struct SimpleRepository<DATA, M: C3p0Model<DATA>>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    conf: Conf<DATA, M>,
}

impl<DATA, M: C3p0Model<DATA>> SimpleRepository<DATA, M>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn build<T>(
        conn: Connection,
        table_name: T,
        factory: fn(Option<i64>, i32, DATA) -> M,
    ) -> Self
    where
        T: Into<String>,
    {
        SimpleRepository::build_with_conf(Conf::build(conn, table_name, factory))
    }

    pub fn build_custom<T, I, V, D>(
        conn: Connection,
        table_name: T,
        id_field_name: I,
        version_field_name: V,
        data_field_name: D,
        factory: fn(Option<i64>, i32, DATA) -> M,
    ) -> Self
    where
        T: Into<String>,
        I: Into<String>,
        V: Into<String>,
        D: Into<String>,
    {
        SimpleRepository::build_with_conf(Conf::build_custom(
            conn,
            table_name,
            id_field_name,
            version_field_name,
            data_field_name,
            factory,
        ))
    }

    pub fn build_with_conf(conf: Conf<DATA, M>) -> Self {
        SimpleRepository { conf }
    }
}

impl<DATA, M: C3p0Model<DATA>> JpoPg<DATA, M> for SimpleRepository<DATA, M>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn conf(&self) -> &Conf<DATA, M> {
        &self.conf
    }
}
