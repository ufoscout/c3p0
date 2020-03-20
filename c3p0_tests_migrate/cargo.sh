#!/usr/bin/env bash

declare -a features=(
    "pg"
    "pg_015"
    "sqlite"
    "mysql"
)

for i in "${features[@]}"
do
    LINE_SEPARATOR='--------------------------------------------------------'

    echo $LINE_SEPARATOR
    echo 'C3p0_tests_migrate - Run Cargo with args [' $@ '] and features [' $i ']'
    echo $LINE_SEPARATOR

    cargo $@ --features $i
    rc=$?
    if [[ $rc -ne 0 ]] ; then
        echo "Failure building feature $i"; exit $rc
    fi

done
