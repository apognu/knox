#!/bin/sh

set -e

CRATES=(libknox knox)

if [ $# -gt 0 ]; then
  CRATES=("$@")
fi

cargo test -- --test-threads=1

for CRATE in ${CRATES[@]}; do
  echo "INFO : processing crate ${CRATE}."
  pushd $CRATE > /dev/null

  VERSION="$(cargo pkgid | cut -d# -f2)"

  if ! curl -s -o /dev/null -w '%{http_code}' https://github.com/apognu/knox/tree/v${VERSION} | grep 200 > /dev/null; then
    echo "ERROR: tag v${VERSION} does not exist in upstream git repository."
    exit 1
  fi
  
  cp Cargo.toml Cargo.toml.bak
  sed -i 's/build =.*/build = false/' Cargo.toml
  sed -i '/^\[dev-dependencies\]/,/^$/d' Cargo.toml

  cargo package --allow-dirty --no-verify
  cargo publish --allow-dirty

  mv Cargo.toml.bak Cargo.toml
  
  popd > /dev/null
done
