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

#[derive(Clone, Debug, PartialEq)]
pub enum ForUpdate {
    Default,
    SkipLocked,
    NoWait,
    Wait,
    No,
}

/*
impl ForUpdate {
    pub fn to_sql(&self) -> &str {
        match self {
            ForUpdate::ForUpdate => "for update",
            ForUpdate::No => "",
        }
    }
}
*/