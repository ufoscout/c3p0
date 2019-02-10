#!/bin/bash

declare -a steps=(
  "cargo test"
  "cargo build"
  "cd ./auth         && cargo publish && cd .."
  "cd ./json         && cargo publish && cd .."
  "cd ./jwt          && cargo publish && cd .."
  "cd ./logger       && cargo publish && cd .."
  "cd ./module       && cargo publish && cd .."
  "cd ./module_core  && cargo publish && cd .."
)

for i in "${steps[@]}"
do
    echo "Execute step: '$i'"
    eval $i
    rc=$?
    if [[ $rc -ne 0 ]] ; then
        echo "Failure executing: $i"; exit $rc
    fi
done
