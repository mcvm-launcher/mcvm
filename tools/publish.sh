#!/bin/sh
# Run from the root directory of the repository

cd crates

cd mcvm_shared
cargo publish
cd ..

cd mcvm_auth
cargo publish
cd ..

cd mcvm_core
cargo publish
cd ..

cd mcvm_mods
cargo publish
cd ..

cd mcvm_parse
cargo publish
cd ..

cd mcvm_pkg
cargo publish
cd ..

cd ..
cargo publish

cd crates/mcvm_cli
cargo publish

cd ../..
