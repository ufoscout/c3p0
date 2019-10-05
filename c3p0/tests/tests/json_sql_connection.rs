use crate::utils::*;
use crate::*;

#[test]
fn should_fetch_by_sql() {
    SINGLETON.get(|(pool, _)| {
        let conn = pool.connection().unwrap();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        let jpo = C3p0JsonBuilder::new(table_name.clone()).build();

        assert!(jpo.create_table_if_not_exists(&conn).is_ok());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let _model = jpo.save(&conn, model.clone()).unwrap();

        let one = jpo
            .fetch_one_with_sql(
                &conn,
                &format!("select id, version, data from {}", table_name),
                &[],
            )
            .unwrap();
        assert!(one.is_some());

        let all = jpo
            .fetch_all_with_sql(
                &conn,
                &format!("select id, version, data from {}", table_name),
                &[],
            )
            .unwrap();
        assert!(!all.is_empty());
    });
}
