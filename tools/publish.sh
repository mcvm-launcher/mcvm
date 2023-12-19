#!/bin/sh
# Run from the root directory of the repository

cd crates

# mcvm_shared
cd mcvm_shared
cargo publish
cd ..

# mcvm_auth
cd mcvm_auth
cargo publish
cd ..

# mcvm_core
cd mcvm_core
cargo publish
cd ..

# mcvm_mods
cd mcvm_mods
cargo publish
cd ..

# mcvm_parse
cd mcvm_parse
cargo publish
cd ..

# mcvm_pkg
cd mcvm_pkg
cargo publish
cd ..

# mcvm_options
cd mcvm_options
cargo publish
cd ..

# mcvm
cd ..
cargo publish

# mcvm_cli
cd crates/mcvm_cli
cargo publish

cd ../..
