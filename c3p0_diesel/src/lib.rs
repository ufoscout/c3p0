use c3p0::{C3p0ModelInsertable, C3p0ModelQueryable};
use diesel::prelude::*;

pub trait JpoDiesel<T>
where
    T: Table,
{
    fn save<I, DATA, Q, C>(self, obj: I, conn: &C) -> QueryResult<Q>
        where Self: Sized,
              I: Insertable<T> + C3p0ModelInsertable<DATA>,
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
        diesel::insert_into(self.table())
            .values(obj)
            .get_result(conn)
    }

    /*
    fn find_by_id<Q, C>(self, id: i64, conn: &C) -> QueryResult<Vec<Q>>
        where Self: Sized,
              C: Connection,
              T::Query: diesel::query_builder::QueryFragment<C::Backend>,
                <C as diesel::connection::Connection>::Backend: diesel::sql_types::HasSqlType<<T as diesel::query_builder::AsQuery>::SqlType>,
              <T as diesel::query_builder::AsQuery>::Query: diesel::query_builder::QueryId,
              Q: diesel::deserialize::Queryable<<<T as diesel::query_source::Table>::AllColumns as diesel::expression::Expression>::SqlType, <C as diesel::connection::Connection>::Backend> + diesel::deserialize::Queryable<<T as diesel::query_builder::AsQuery>::SqlType, C::Backend>

    {
        self.table()
            //.filter(schema::test_table::published.eq(true))
            .limit(5)
            .load::<Q>(conn)
    }
    */

    fn table(self) -> T;
}

pub struct SimpleRepository<T>
where
    T: Table,
{
    table: T,
}

impl<T> SimpleRepository<T>
where
    T: Table,
{
    pub fn new(table: T) -> impl JpoDiesel<T> {
        SimpleRepository { table }
    }
}

impl<T> JpoDiesel<T> for SimpleRepository<T>
where
    T: Table,
{
    fn table(self) -> T {
        self.table
    }
}
