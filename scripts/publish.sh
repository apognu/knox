#!/bin/sh

set -e

CRATES=(libknox knox)

if [ $# -gt 0 ]; then
  CRATES=("$@")
fi

cargo test -- --test-threads=1

for CRATE in ${CRATES[@]}; do
  pushd $CRATE
  
  cp Cargo.toml Cargo.toml.bak
  sed -i 's/build =.*/build = false/' Cargo.toml
  sed -i '/^\[dev-dependencies\]/,/^$/d' Cargo.toml

  cargo package --allow-dirty --no-verify
  cargo publish --allow-dirty

  mv Cargo.toml.bak Cargo.toml
  
  popd
done
