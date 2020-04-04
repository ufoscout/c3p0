#!/usr/bin/env bash

declare -a features=(
    "in_memory"
    "pg"
    "pg_async"
    "sqlite"
    "mysql"
    "pg_migrate"
    "sqlite_migrate"
    "mysql_migrate"
)

for i in "${features[@]}"
do
    LINE_SEPARATOR='--------------------------------------------------------'

    echo $LINE_SEPARATOR
    echo 'C3p0 - Run Cargo with args [' $@ '] and features [' $i ']'
    echo $LINE_SEPARATOR

    cargo $@ --features $i
    rc=$?
    if [[ $rc -ne 0 ]] ; then
        echo "Failure building feature $i"; exit $rc
    fi

done
