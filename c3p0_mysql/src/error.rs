use c3p0_common::error::C3p0Error;
use mysql_async::Error;

pub fn into_c3p0_error(error: Error) -> C3p0Error {
    let code = match &error {
        //Error::MySqlError(e) => Some(e.code.to_string()),
        _ => None,
    };

    C3p0Error::DbError {
        db: "mysql",
        cause: format!("{}", error),
        code,
    }
}
