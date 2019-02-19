pub trait C3p0Model<DATA>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn c3p0_get_id(&self) -> &Option<i64>;
    fn c3p0_get_version(&self) -> &i32;
    fn c3p0_get_data(&self) -> &DATA;
    fn c3p0_clone_with_id<ID: Into<Option<i64>>>(self, id: ID) -> Self;
}

pub trait C3p0ModelQueryable<DATA>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn c3p0_get_id(&self) -> &i64;
    fn c3p0_get_version(&self) -> &i32;
    fn c3p0_get_data(&self) -> &DATA;
}

pub trait C3p0ModelInsertable<DATA>
where
    DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn c3p0_get_version(&self) -> &i32;
    fn c3p0_get_data(&self) -> &DATA;
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
        return Model {
            id: id.into(),
            version,
            data,
        };
    }

    pub fn new_with_data(data: DATA) -> Model<DATA> {
        return Model {
            id: None,
            version: 0,
            data,
        };
    }
}

/*
pub trait Jpo<DATA, M: C3p0Model<DATA>> where DATA: serde::ser::Serialize + serde::de::DeserializeOwned {

    fn find_by_id(&self, id: i64) -> Option<M>;

    fn save(&self, obj: M) -> M;

}
*/
