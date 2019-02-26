#[macro_use]
extern crate diesel;
use c3p0_diesel_macro::*;

mod schema {
    table! {
        test_table (id) {
            id -> Int8,
            version -> Int4,
            data -> Jsonb,
        }
    }
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, DieselJson)]
pub struct CustomData {
    pub name: String,
}

use schema::*;

//-GENERATED-MODEL-START

#[derive(Insertable)]
#[table_name = "test_table"]
pub struct NewCustomDataModel {
    pub version: i32,
    pub data: CustomData,
}

#[derive(Queryable)]
pub struct CustomDataModel {
    pub id: i64,
    pub version: i32,
    pub data: CustomData,
}

use test_table::dsl as tt_dsl;

pub trait CustomDataRepository {
    fn save(
        obj: NewCustomDataModel,
        conn: &diesel::pg::PgConnection,
    ) -> diesel::result::QueryResult<CustomDataModel> {
        use diesel::prelude::*;

        diesel::insert_into(tt_dsl::test_table)
            .values(obj)
            .get_result(conn)
    }

    fn find_by_id(
        id: i64,
        conn: &diesel::pg::PgConnection,
    ) -> diesel::result::QueryResult<Vec<CustomDataModel>> {
        use diesel::prelude::*;

        tt_dsl::test_table.filter(tt_dsl::id.eq(id)).load(conn)
    }
}

//-GENERATED-MODEL-END
