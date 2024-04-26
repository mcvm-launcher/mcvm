install: install.args install.backup install.options

install.args: build.args
	@cargo install --path . --bin mcvm_plugin_args --offline
	@cp plugins/args/plugin.json ~/.local/share/mcvm/plugins/args.json

build.args: plugins/args
	@cargo build -r --bin mcvm_plugin_args

install.backup: build.backup
	@cargo install --path . --bin mcvm_plugin_backup --offline
	@cp plugins/backup/plugin.json ~/.local/share/mcvm/plugins/backup.json

build.backup: plugins/backup
	@cargo build -r --bin mcvm_plugin_backup

install.options: build.options
	@cargo install --path . --bin mcvm_plugin_options --offline
	@cp plugins/options/plugin.json ~/.local/share/mcvm/plugins/options.json

build.options: plugins/options
	@cargo build -r --bin mcvm_plugin_options

install.docs: build.docs
	@cargo install --path . --bin mcvm_plugin_docs --offline
	@cp plugins/docs/plugin.json ~/.local/share/mcvm/plugins/docs.json

build.docs: plugins/docs
	@cargo build -r --bin mcvm_plugin_docs
