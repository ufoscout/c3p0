use postgres::rows::Row;
use postgres::Connection;

pub struct PgMigrate {

}

struct Migration {
    up: String,
    down: String
}

struct SqlScript {
    sql: String
}

#[cfg(test)]
mod test {

    #[test]
    fn md5_spike() {
        use md5::{Md5, Digest};
        use std::str;

        let some_value = "22341242141241242142";
        let mut md5 = Md5::default();
        md5.input(&some_value);
        let md5_result_hex = md5.result();
        println!("result is: [{:?}]", &md5_result_hex);

        let md5_result_str = md5_result_hex.iter().map(|&c| format!("{:02x}", c)).collect::<String>();
        println!("result is: [{}]", &md5_result_str);

        assert_eq!("5f759e6f82017c8cd17cd75f3c7d52a4", &md5_result_str);
    }

}