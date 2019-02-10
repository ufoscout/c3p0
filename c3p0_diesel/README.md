It requires the postgres library to be installed on the host.

On Ubuntu:
> sudo apt install libpq-dev

The install diesel_cli:
> cargo install diesel_cli --no-default-features --features postgres
