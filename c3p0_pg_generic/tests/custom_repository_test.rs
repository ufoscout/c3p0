use crate::shared::*;
use c3p0::{C3p0, NewModel};
use c3p0_pg_generic::{PostgresManager, PostgresManagerBuilder};

mod shared;

struct TestTableRepository<'a> {
    db: PostgresManager<'a, TestData>,
}

impl <'a> C3p0<TestData, PostgresManager<'a, TestData>> for TestTableRepository<'a> {
    fn db_manager(&self) -> &PostgresManager<'a, TestData> {
        &self.db
    }
}

#[test]
fn postgres_basic_crud() {
    shared::SINGLETON.get(|(pool, _)| {
        let conn = pool.get().unwrap();
        conn.batch_execute(
            "create table TEST_TABLE (
                            ID bigserial primary key,
                            VERSION int not null,
                            DATA JSONB
                        );
              ",
        )
        .unwrap();

        let db = PostgresManagerBuilder::new("TEST_TABLE").build();

        let jpo = TestTableRepository { db };

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(&conn, model.clone()).unwrap();
        assert!(saved_model.id >= 0);

        let found_model = jpo.find_by_id(&conn, &saved_model.id).unwrap().unwrap();
        assert_eq!(saved_model.id, found_model.id);
        assert_eq!(saved_model.version, found_model.version);
        assert_eq!(saved_model.data.first_name, found_model.data.first_name);
        assert_eq!(saved_model.data.last_name, found_model.data.last_name);
    });
}
