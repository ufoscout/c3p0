#!/usr/bin/env bash

declare -a features=(
    "pg"
    "sqlite"
    "mysql"
)

for i in "${features[@]}"
do
    LINE_SEPARATOR='--------------------------------------------------------'

    echo $LINE_SEPARATOR
    echo 'Run Cargo with args [' $@ '] and features [' $i ']'
    echo $LINE_SEPARATOR

    cargo $@ --features $i
    rc=$?
    if [[ $rc -ne 0 ]] ; then
        echo "Failure building feature $i"; exit $rc
    fi

done
