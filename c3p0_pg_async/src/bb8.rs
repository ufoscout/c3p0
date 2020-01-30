pub use bb8;
pub use tokio_postgres;

use async_trait::async_trait;
use futures::prelude::*;
use tokio_postgres::config::Config;
use tokio_postgres::tls::{MakeTlsConnect, TlsConnect};
use tokio_postgres::{Client, Error, Socket};

use std::fmt;
use std::str::FromStr;

pub struct PostgresConnectionManager
{
    config: Config,
    tls_connector: Box<dyn Fn(&Config) -> Result<Client, Error> + Send + Sync>,
}

impl PostgresConnectionManager
{
    /// Create a new `PostgresConnectionManager` with the specified `config`.
    pub fn new(
        config: Config,
        tls_connector: Box<dyn Fn(&Config) -> Result<Client, Error> + Send + Sync>,
    ) -> PostgresConnectionManager {
        PostgresConnectionManager {
            config,
            tls_connector,
        }
    }

    /// Create a new `PostgresConnectionManager`, parsing the config from `params`.
    pub fn new_from_stringlike<T>(
        params: T,
        tls_connector: Box<dyn Fn(&Config) -> Result<Client, Error> + Send + Sync>,
    ) -> Result<PostgresConnectionManager, Error>
        where
            T: ToString,
    {
        let stringified_params = params.to_string();
        let config = Config::from_str(&stringified_params)?;
        Ok(Self::new(config, tls_connector))
    }
}

#[async_trait]
pub trait ConnectionProvider: Send + Sync + 'static {

    /// The connection type this manager deals with.
    type Connection: Send + 'static;
    /// The error type returned by `Connection`s.
    type Error: fmt::Debug + Send + 'static;

    /// Attempts to create a new connection.
    async fn connect(&self) -> Result<Self::Connection, Self::Error>;
}

#[async_trait]
impl bb8::ManageConnection for PostgresConnectionManager
{
    type Connection = Client;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        unimplemented!()

        /*
        let tls = (self.tls_connector)(&self.config)?;
        self.config
            .connect(tls)
            .await
            .map(|(client, connection)| {
                // The connection object performs the actual communication with the database,
                // so spawn it off to run on its own.
                tokio::spawn(connection.map(|_| ()));

                client
            })
            */
    }

    async fn is_valid(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        conn.simple_query("").await.map(|_| conn)
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.is_closed()
    }
}

impl fmt::Debug for PostgresConnectionManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PostgresConnectionManager")
            .field("config", &self.config)
            .finish()
    }
}
