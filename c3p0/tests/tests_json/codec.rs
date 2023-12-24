use c3p0::*;

use crate::utils::{codec::*, *};
use crate::*;

#[test]
fn should_upgrade_structs_on_load() -> Result<(), C3p0Error> {
    test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(|conn| async {
            let table_name = format!("USER_TABLE_{}", rand_string(8));

            let jpo_v1 = Builder::new(&table_name)
                .build_with_codec(UserVersionCoded1 {});

            let jpo_v2 = Builder::new(&table_name)
                .build_with_codec(UserVersionCoded2 {});

            let new_user_v1 = NewModel::new(UserVersion1 {
                username: "user_v1_name".to_owned(),
                email: "user_v1_email@test.com".to_owned(),
            });

            assert!(jpo_v1.create_table_if_not_exists(conn).await.is_ok());
            assert!(jpo_v1.delete_all(conn).await.is_ok());

            let user_v1 = jpo_v1.save(conn, new_user_v1.clone()).await.unwrap();
            println!("user id is {}", user_v1.id);
            println!("total users: {}", jpo_v1.count_all(conn).await.unwrap());
            println!(
                "select all users len: {}",
                jpo_v1.fetch_all(conn).await.unwrap().len()
            );

            let user_v2_found = jpo_v2
                .fetch_one_optional_by_id(conn, &user_v1.id)
                .await
                .unwrap();
            assert!(user_v2_found.is_some());

            let user_v2_found = user_v2_found.unwrap();
            assert_eq!(user_v1.id, user_v2_found.id);
            assert_eq!(user_v1.version, user_v2_found.version);
            assert_eq!(user_v1.data.username, user_v2_found.data.username);
            assert_eq!(user_v1.data.email, user_v2_found.data.email);
            assert_eq!(18, user_v2_found.data.age);
            Ok(())
        })
        .await
    })
}
