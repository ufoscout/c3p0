#![cfg(feature = "pg_async")]

use c3p0::pg_async::bb8::{Pool, PostgresConnectionManager};
use c3p0::pg_async::*;
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use testcontainers::*;

pub use c3p0::pg_async::driver::row::Row;
pub use c3p0::pg_async::driver::tls::NoTls;
use futures::executor::block_on;
use futures::Future;

pub type C3p0Impl = PgC3p0Pool;

//mod tests;
mod tests_json_async;

pub mod utils;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        C3p0Impl,
        Container<'static, clients::Cli, images::postgres::Postgres>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    C3p0Impl,
    Container<'static, clients::Cli, images::postgres::Postgres>,
) {
    let node = DOCKER.run(images::postgres::Postgres::default());

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        )
        .parse()
        .unwrap(),
        NoTls,
    );


    let pool = block_on(Pool::builder().min_idle(Some(10)).build(manager)).unwrap();

    let pool = PgC3p0Pool::new(pool);

    (pool, node)
}

pub fn test<F: Future<Output = Result<(), C3p0Error>>>(callback: fn(C3p0Impl) -> F) {
    SINGLETON.get(|(c3p0, _)| {
        let clone: C3p0Impl = c3p0.clone();
        block_on(callback(clone)).unwrap();
    });
}

/*
pub fn call_async<'a, 'b: 'a, F: 'a + Future<Output = Result<(), ()>>>(callback: fn(&'b str) -> F) {
}

#[test]
fn should_call_async() {
    call_async(|value| async {
        let value_ref = value;
        Ok(())
    })
}
*/