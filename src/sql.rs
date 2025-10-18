use std::fmt::{Debug, Display};

/// An enum to represent the order by of a query.
#[derive(Clone, Debug, PartialEq)]
pub enum OrderBy {
    Asc,
    Desc,
    Default,
}

impl Display for OrderBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderBy::Asc => write!(f, "asc"),
            OrderBy::Desc => write!(f, "desc"),
            OrderBy::Default => write!(f, ""),
        }
    }
}
