#[derive(Clone, Debug, PartialEq)]
pub enum OrderBy {
    Asc,
    Desc,
    Default,
}

impl OrderBy {
    pub fn to_sql(&self) -> &str {
        match self {
            OrderBy::Asc => "asc",
            OrderBy::Desc => "desc",
            OrderBy::Default => "",
        }
    }
}
