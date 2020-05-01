use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

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
enum Versioning1<'a> {
    V1(Cow<'a, UserVersion1>),
}

#[derive(Clone)]
pub struct UserVersionCoded1 {}

impl JsonCodec<UserVersion1> for UserVersionCoded1 {
    fn from_value(&self, value: Value) -> Result<UserVersion1, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let user = match versioning {
            Versioning1::V1(user_v1) => user_v1.into_owned(),
        };
        Ok(user)
    }

    fn to_value(&self, data: &UserVersion1) -> Result<Value, C3p0Error> {
        serde_json::to_value(Versioning1::V1(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "@json_tag")]
enum Versioning2<'a> {
    V1(UserVersion1),
    V2(Cow<'a, UserVersion2>),
}

#[derive(Clone)]
pub struct UserVersionCoded2 {}

impl JsonCodec<UserVersion2> for UserVersionCoded2 {
    fn from_value(&self, value: Value) -> Result<UserVersion2, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let user = match versioning {
            Versioning2::V1(user_v1) => UserVersion2 {
                username: user_v1.username,
                email: user_v1.email,
                age: 18,
            },
            Versioning2::V2(user_v2) => user_v2.into_owned(),
        };
        Ok(user)
    }

    fn to_value(&self, data: &UserVersion2) -> Result<Value, C3p0Error> {
        serde_json::to_value(Versioning2::V2(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}
