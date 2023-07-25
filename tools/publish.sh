#!/bin/sh
# Run from the root directory of the repository

cd crates
cd mcvm_shared
cargo publish
cd ..
cd mcvm_parse
cargo publish
cd ..
cd ..
cargo publish
