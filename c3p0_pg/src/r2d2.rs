use postgres::{Client, Config};
use r2d2::ManageConnection;
use tokio_postgres::Error;

pub struct PostgresConnectionManager {
    config: Config,
    tls_connector: Box<dyn Fn(&Config) -> Result<Client, Error> + Send + Sync>,
}

impl PostgresConnectionManager {
    pub fn new(
        config: Config,
        tls_connector: Box<dyn Fn(&Config) -> Result<Client, Error> + Send + Sync>,
    ) -> PostgresConnectionManager {
        PostgresConnectionManager {
            config,
            tls_connector,
        }
    }
}

impl ManageConnection for PostgresConnectionManager {
    type Connection = Client;
    type Error = Error;

    fn connect(&self) -> Result<Client, Error> {
        (self.tls_connector)(&self.config)
    }

    fn is_valid(&self, client: &mut Client) -> Result<(), Error> {
        client.simple_query("").map(|_| ())
    }

    fn has_broken(&self, client: &mut Client) -> bool {
        client.is_closed()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use tokio_postgres::tls::NoTls;

    #[test]
    fn new_connection() {
        let tls = NoTls;
        let _manager = PostgresConnectionManager::new(
            "host=localhost user=postgres".parse().unwrap(),
            Box::new(move |config| config.connect(tls.clone())),
        );
    }
}
