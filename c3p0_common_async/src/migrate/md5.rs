use md5::{Digest, Md5};
use std::str;

pub fn calculate_md5(source: &str) -> String {
    let mut md5 = Md5::default();
    md5.input(source);
    let md5_result_hex = md5.result();
    md5_result_hex
        .iter()
        .map(|&c| format!("{:02x}", c))
        .collect::<String>()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn md5_spike() {
        let md5_result_str = calculate_md5("22341242141241242142");
        println!("result is: [{:?}]", &md5_result_str);
        assert_eq!("5f759e6f82017c8cd17cd75f3c7d52a4", &md5_result_str);
    }
}
