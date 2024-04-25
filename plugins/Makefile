install: install.args

install.args: build.args
	@cargo install --path . --bin mcvm_plugin_args --offline
	@cp plugins/args/plugin.json ~/.local/share/mcvm/plugins/args.json

build.args: plugins/args
	@cargo build -r --bin mcvm_plugin_args

install.snapshot: build.snapshot
	@cargo install --path . --bin mcvm_plugin_snapshot --offline
	@cp plugins/snapshot/plugin.json ~/.local/share/mcvm/plugins/snapshot.json

build.snapshot: plugins/snapshot
	@cargo build -r --bin mcvm_plugin_snapshot
