use c3p0::*;
use serde::{Deserialize, Serialize};

use crate::{utils::run_test, *};

#[test]
fn should_upgrade_structs_on_load() -> Result<(), C3p0Error> {
    run_test(async {
        let data = data(false).await;
        let pool = &data.0;

        pool.transaction(async |conn| {
            let new_user_v1 = NewRecord::new(UserVersion1 {
                username: "user_v1_name".to_owned(),
                email: "user_v1_email@test.com".to_owned(),
            });

            assert!(
                conn.drop_table_if_exists::<UserVersion1>(true)
                    .await
                    .is_ok()
            );
            assert!(
                conn.create_table_if_not_exists::<UserVersion1>()
                    .await
                    .is_ok()
            );
            assert!(conn.delete_all::<UserVersion1>().await.is_ok());

            let user_v1 = conn.save(new_user_v1.clone()).await.unwrap();
            println!("user id is {}", user_v1.id);
            println!(
                "total users: {}",
                conn.count_all::<UserVersion1>().await.unwrap()
            );
            println!(
                "select all users len: {}",
                conn.fetch_all::<UserVersion1>().await.unwrap().len()
            );

            let user_v2_found = conn
                .fetch_one_optional_by_id::<UserVersion2>(user_v1.id)
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

const RAND: u64 = const_random::const_random!(u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UserVersion1 {
    pub username: String,
    pub email: String,
}

impl c3p0::Data for UserVersion1 {
    const TABLE_NAME: &'static str = const_format::concatcp!("CODEC_TEST_TABLE", RAND);
    type CODEC = Versioning1;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UserVersion2 {
    pub username: String,
    pub email: String,
    pub age: u32,
}

impl c3p0::Data for UserVersion2 {
    const TABLE_NAME: &'static str = const_format::concatcp!("CODEC_TEST_TABLE", RAND);
    type CODEC = Versioning2;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "@json_tag")]
enum Versioning1 {
    V1(UserVersion1),
}

impl Codec<UserVersion1> for Versioning1 {
    fn encode(data: UserVersion1) -> Self {
        Versioning1::V1(data)
    }

    fn decode(data: Self) -> UserVersion1 {
        match data {
            Versioning1::V1(user_v1) => user_v1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "@json_tag")]
enum Versioning2 {
    V1(UserVersion1),
    V2(UserVersion2),
}

impl Codec<UserVersion2> for Versioning2 {
    fn encode(data: UserVersion2) -> Self {
        Versioning2::V2(data)
    }

    fn decode(data: Self) -> UserVersion2 {
        match data {
            Versioning2::V1(user_v1) => UserVersion2 {
                username: user_v1.username,
                email: user_v1.email,
                age: 18,
            },
            Versioning2::V2(user_v2) => user_v2,
        }
    }
}
