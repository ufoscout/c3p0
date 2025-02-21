
# Builds all artifacts
[group('build')]
build:
  cargo build
  cargo build --all-features

# Publishes all artifacts on crates.io
[group('build')]
publish:
  #!/usr/bin/env bash

  ## exit if something fails
  set -e

  declare -a publish_list=(
      "c3p0_common"
      "c3p0_postgres"
      "c3p0_sqlx"
      "c3p0"
  )

  for i in "${publish_list[@]}"
  do
      LINE_SEPARATOR='--------------------------------------------------------'

      cd $i
      echo $LINE_SEPARATOR
      echo 'C3p0 - Run Cargo publish for [' $i ']'
      echo $LINE_SEPARATOR

      cargo publish
      sleep 2
      cd ..
      rc=$?
      if [[ $rc -ne 0 ]] ; then
          echo "Failure publishing $i";
      fi

  done
