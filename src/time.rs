use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::C3p0Error;

/// Returns the current unix timestamp in millis
pub fn get_current_epoch_millis() -> Result<i64, C3p0Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .map_err(|e| C3p0Error::Error {
            cause: format!("System clock is before UNIX epoch: {e}"),
        })
}

#[cfg(test)]
mod test {
    use super::get_current_epoch_millis;

    #[test]
    fn should_return_the_current_epoch_millis() {
        assert!(get_current_epoch_millis().unwrap() > 1_500_000_000_000);
    }
}
