use std::time::{SystemTime, UNIX_EPOCH};

use crate::json::types::EpochMillisType;

/// Returns the current unix timestamp in millis
pub fn get_current_epoch_millis() -> EpochMillisType {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?")
        .as_millis() as EpochMillisType
}

#[cfg(test)]
mod test {
    use super::get_current_epoch_millis;

    #[test]
    fn should_return_the_current_epoch_millis() {
        assert!(get_current_epoch_millis() > 1_500_000_000_000);
    }
}
