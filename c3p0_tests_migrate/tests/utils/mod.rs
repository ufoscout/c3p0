use rand::{Rng, distr::Alphanumeric};

pub fn rand_string(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(len)
        .collect::<String>()
}

#[derive(Debug, PartialEq)]
pub enum DbType {
    Pg,
}
