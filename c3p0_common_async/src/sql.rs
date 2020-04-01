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
    No,
}

impl ForUpdate {
    pub fn to_sql(&self) -> &str {
        match self {
            ForUpdate::Default => "for update",
            ForUpdate::SkipLocked => "for update skip locked",
            ForUpdate::NoWait => "for update NoWait",
            ForUpdate::No => "",
        }
    }
}
