use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserVersion1 {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserVersion2 {
    pub username: String,
    pub email: String,
    pub age: u32,
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
