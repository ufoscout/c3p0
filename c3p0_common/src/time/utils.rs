use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current unix timestamp in millis
pub fn get_current_epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?")
        .as_millis()
}

#[cfg(test)]
mod test {
    use super::get_current_epoch_millis;


    #[test]
    fn should_return_the_current_epoch_millis() {
        assert!(get_current_epoch_millis() > 1_500_000_000_000);
    }

}
