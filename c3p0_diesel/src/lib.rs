use diesel::connection::Connection;
use diesel::insertable::Insertable;

pub trait Model<DATA> where DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned {
    fn c3p0_get_id(&self) -> Option<i64>;
    fn c3p0_get_version(&self) -> i32;
    fn c3p0_get_data(&self) -> DATA;
    fn c3p0_clone_with_id_and_version(&self, id: Option<i64>, version: i32) -> Self;
}

pub struct JpoDiesel<C: Connection> {
    conn: C
}

impl <C: Connection> JpoDiesel<C> {

    fn save<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned, M: Model<DATA>>(&self, obj: &M) -> M {
        obj.c3p0_clone_with_id_and_version(None, 0)
    }

}