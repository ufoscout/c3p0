use c3p0_pg_migrate::PgMigrateBuilder;
use testcontainers::clients;

mod shared;

#[test]
fn should_create_and_drop_table() {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let pg_migrate = PgMigrateBuilder::new().build();
}
