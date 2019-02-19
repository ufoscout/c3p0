use c3p0::{C3p0ModelQueryable, C3p0ModelInsertable};
use diesel::backend::Backend;
use diesel::prelude::*;

pub trait JpoDiesel {

    fn save<I, DATA, Q, T, C>(&self, obj: I, table: T, conn: &C) -> QueryResult<Q>
        where I: Insertable<T> + C3p0ModelInsertable<DATA>,
              T: Table,
              C: Connection,
              Q: diesel::deserialize::Queryable<<<T as diesel::query_source::Table>::AllColumns as diesel::expression::Expression>::SqlType, <C as diesel::connection::Connection>::Backend> + C3p0ModelQueryable<DATA>,
              T::FromClause: diesel::query_builder::QueryFragment<C::Backend>,
                C::Backend: diesel::backend::SupportsReturningClause +  diesel::sql_types::HasSqlType<<<T as diesel::query_source::Table>::AllColumns as diesel::expression::Expression>::SqlType>,
                I::Values: diesel::insertable::CanInsertInSingleQuery<<C as diesel::connection::Connection>::Backend>,
              <I as diesel::insertable::Insertable<T>>::Values: diesel::query_builder::QueryFragment<<C as diesel::connection::Connection>::Backend>,
              <T as diesel::query_source::Table>::AllColumns: diesel::query_builder::QueryFragment<<C as diesel::connection::Connection>::Backend>,
                //T::AllColumns: C,
              DATA: serde::ser::Serialize + serde::de::DeserializeOwned
    {
        diesel::insert_into(table)
            .values(obj)
            .get_result(conn)
    }

}

pub struct SimpleRepository{}

impl SimpleRepository {
    pub fn new() -> impl JpoDiesel {
        SimpleRepository{}
    }
}

impl JpoDiesel for SimpleRepository {
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