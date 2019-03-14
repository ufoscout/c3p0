-- Your SQL goes here

create table TEST_TABLE (
    ID bigserial primary key,
    VERSION int not null,
    DATA JSONB not null
)