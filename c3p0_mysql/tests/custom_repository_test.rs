use crate::shared::*;
use c3p0::NewModel;
use c3p0_mysql::{C3p0, MySqlManager, MySqlManagerBuilder};

mod shared;

struct TestTableRepository {
    conf: MySqlManager<TestData>,
}

impl C3p0<TestData> for TestTableRepository {
    fn conf(&self) -> &MySqlManager<TestData> {
        &self.conf
    }
}

#[test]
fn mysql_basic_crud() {
    shared::SINGLETON.get(|(pool, _)| {
        let mut conn = pool.get().unwrap();
        conn.prep_exec(
            "create table TEST_TABLE (
                            ID bigint primary key auto_increment,
                            VERSION int not null,
                            DATA JSON
                        );
              ",
            (),
        )
        .unwrap();

        let conf = MySqlManagerBuilder::new("TEST_TABLE").build();

        let jpo = TestTableRepository { conf };

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(&mut conn, model.clone()).unwrap();
        assert!(saved_model.id >= 0);

        let found_model = jpo.find_by_id(&mut conn, &saved_model.id).unwrap().unwrap();
        assert_eq!(saved_model.id, found_model.id);
        assert_eq!(saved_model.version, found_model.version);
        assert_eq!(saved_model.data.first_name, found_model.data.first_name);
        assert_eq!(saved_model.data.last_name, found_model.data.last_name);
    });
}
