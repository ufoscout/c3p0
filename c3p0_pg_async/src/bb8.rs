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
    tls_connector: Box<dyn Send + Sync + 'static + Fn(&Config) -> (dyn Future<Output = Result<Client, Error>> + 'static)>,
}

impl PostgresConnectionManager
{
    /// Create a new `PostgresConnectionManager` with the specified `config`.
    pub fn new(
        config: Config,
        tls_connector: Box<dyn Send + Sync + 'static + Fn(&Config) -> (dyn Future<Output = Result<Client, Error>> + 'static)>,
    ) -> PostgresConnectionManager {
        PostgresConnectionManager {
            config,
            tls_connector,
        }
    }

    /// Create a new `PostgresConnectionManager`, parsing the config from `params`.
    pub fn new_from_stringlike<T>(
        params: T,
        tls_connector: Box<dyn Send + Sync + 'static + Fn(&Config) -> (dyn Future<Output = Result<Client, Error>> + 'static)>,
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
impl bb8::ManageConnection for PostgresConnectionManager
{
    type Connection = Client;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let val = (self.tls_connector)(&self.config);
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

struct MyStructOne;
struct MyError;

pub struct FutureWithFn {
    one: MyStructOne,
    func_one: Box<dyn Send + Sync + 'static + Fn(&MyStructOne) -> Box<dyn Future<Output = Result<(), MyError>> + 'static + Unpin>>,
    func_two: Box<dyn Send + Sync + 'static + Fn(&MyStructOne) -> (dyn Future<Output = Result<(), MyError>> + 'static + Unpin)>,
}

impl FutureWithFn {

    async fn call_one(&self) -> Result<(), MyError> {
        (self.func_one)(&self.one).await;    // OK!
        Ok(())
    }

    async fn call_two(&self) -> Result<(), MyError> {
        (self.func_two)(&self.one).await;     // <-- ERROR: doesn't have a size known at compile-time
                                              //            the trait `std::marker::Sized` is not implemented for `dyn core::future::future::Future<Output = std::result::Result<(), MyError>> + std::marker::Unpin`
        Ok(())
    }
}

fn use_it() {

    let fut_with_fn = FutureWithFn {
        one: MyStructOne{},
        func_one: Box::new(|_config| async {  // <-- ERROR: expected struct `std::boxed::Box`, found opaque type
                                                    // expected type `std::boxed::Box<dyn core::future::future::Future<Output = std::result::Result<(), MyError>> + std::marker::Unpin>`
                                                    // found type `impl core::future::future::Future`

            Ok(())
        }),
        func_two: Box::new(|_config| async { // <-- ERROR: doesn't have a size known at compile-time
            Ok(())                              // <-- ERROR: expected trait core::future::future::Future, found opaque type
        }),
    };

}