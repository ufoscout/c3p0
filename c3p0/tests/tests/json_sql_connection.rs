use crate::utils::*;
use crate::*;

#[test]
fn should_fetch_by_sql() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|mut conn| async move {
            let conn = &mut conn;
            let table_name = format!("TEST_TABLE_{}", rand_string(8));
            let jpo = C3p0JsonBuilder::new(table_name.clone()).build();

            assert!(jpo.create_table_if_not_exists(conn).await.is_ok());

            let model = NewModel::new(TestData {
                first_name: "my_first_name".to_owned(),
                last_name: "my_last_name".to_owned(),
            });

            let _model = jpo.save(conn, model.clone()).await.unwrap();

            let one = jpo
                .fetch_one_optional_with_sql(
                    conn,
                    &format!("select id, version, data from {}", table_name),
                    &[],
                )
                .await
                .unwrap();
            assert!(one.is_some());

            let all = jpo
                .fetch_all_with_sql(
                    conn,
                    &format!("select id, version, data from {}", table_name),
                    &[],
                )
                .await
                .unwrap();
            assert!(!all.is_empty());
            Ok(())
        })
        .await
    })
}
