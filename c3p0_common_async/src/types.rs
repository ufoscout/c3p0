pub struct OptString {
    pub value: Option<String>,
}

impl Into<OptString> for String {
    fn into(self) -> OptString {
        OptString { value: Some(self) }
    }
}

impl Into<OptString> for &str {
    fn into(self) -> OptString {
        OptString {
            value: Some(self.to_owned()),
        }
    }
}

impl Into<OptString> for Option<String> {
    fn into(self) -> OptString {
        OptString { value: self }
    }
}

impl Into<OptString> for Option<&str> {
    fn into(self) -> OptString {
        OptString {
            value: self.map(std::borrow::ToOwned::to_owned),
        }
    }
}
