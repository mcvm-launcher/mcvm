#!/bin/sh
# Run from the root directory of the repository

cd crates

# mcvm_shared
cd shared
cargo publish
cd ..

# mcvm_auth
cd auth
cargo publish
cd ..

# mcvm_net
cd net
cargo publish
cd ..

# mcvm_core
cd core
cargo publish
cd ..

# mcvm_mods
cd mods
cargo publish
cd ..

# mcvm_parse
cd parse
cargo publish
cd ..

# mcvm_pkg
cd pkg
cargo publish
cd ..

# mcvm_config
cd config
cargo publish
cd ..

# mcvm_options
cd options
cargo publish
cd ..

# mcvm_plugin
cd plugin
cargo publish
cd ..

# mcvm
cd ..
cargo publish

# mcvm_cli
cd crates/cli
cargo publish

cd ../..
