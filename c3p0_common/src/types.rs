pub struct OptString {
    pub value: Option<String>,
}

impl From<String> for OptString {
    fn from(val: String) -> Self {
        OptString { value: Some(val) }
    }
}

impl From<&str> for OptString {
    fn from(val: &str) -> Self {
        OptString {
            value: Some(val.to_owned()),
        }
    }
}

impl From<Option<String>> for OptString {
    fn from(value: Option<String>) -> Self {
        OptString { value }
    }
}

impl From<Option<&str>> for OptString {
    fn from(value: Option<&str>) -> Self {
        OptString {
            value: value.map(std::borrow::ToOwned::to_owned),
        }
    }
}
