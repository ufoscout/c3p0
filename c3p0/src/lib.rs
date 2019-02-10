#[derive(Clone)]
pub struct Model<DATA> where DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned {
    pub id: Option<i64>,
    pub version: i32,
    pub data: DATA
}

impl <DATA> Model<DATA> where DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned {
    pub fn new(data: DATA) -> Model<DATA> {
        return Model{
            id: None,
            version: 0,
            data
        }
    }
}

pub trait Jpo<DATA> where DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned {

    fn find_by_id(&self, id: i64) -> Option<Model<DATA>>;

    fn save(&self, obj: &Model<DATA>) -> Model<DATA>;

}
