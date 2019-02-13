use diesel::connection::Connection;
use diesel::insertable::Insertable;
use diesel::prelude::Queryable;
use c3p0::{C3p0ModelQueryable, C3p0ModelInsertable};
use diesel::backend::Backend;

pub trait JpoDiesel<C, Q, I, T, DATA, ST, DB>
    where C: Connection,
          Q: Queryable<ST, DB> + C3p0ModelQueryable<DATA>,
          I: Insertable<T> + C3p0ModelInsertable<DATA>,
          DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
          DB: Backend
{
    fn conn(&self) -> &C;
}


impl <C, Q, I, T, DATA, ST, DB> JpoDiesel<C, Q, I, T, DATA, ST, DB>
    where C: Connection,
          Q: Queryable<ST, DB> + C3p0ModelQueryable<DATA>,
          I: Insertable<T> + C3p0ModelInsertable<DATA>,
          DATA: serde::ser::Serialize + serde::de::DeserializeOwned,
          DB: Backend
{

    fn save(&self, obj: &I) -> Q {
        unimplemented!()
    }

}

trait Ab<A, B> {
    fn a() -> A;
    fn b() -> B;
}

trait Cd<C, D>
{
    fn c() -> C;
    fn d() -> D;
}

/*
trait other_wrong<AB, CD>
    where AB: Ab, CD: Cd
{
    fn ab() -> AB;
    fn cd() -> CD;
}
*/

trait other_ok<AB, CD, A, B, C, D>
    where AB: Ab<A,B>, CD: Cd<C,D>
{
    fn ab() -> AB;
    fn cd() -> CD;
}