install: install.args \
				 install.backup \
				 install.options \
				 install.docs \
				 install.stats \
				 install.server_restart

install.lang:
	@cp plugins/lang/plugin.json ~/.local/share/mcvm/plugins/lang.json

install.extra_versions:
	@cp plugins/extra_versions/plugin.json ~/.local/share/mcvm/plugins/extra_versions.json

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

install.stats: build.stats
	@cargo install --path . --bin mcvm_plugin_stats --offline
	@cp plugins/stats/plugin.json ~/.local/share/mcvm/plugins/stats.json

build.stats: plugins/stats
	@cargo build -r --bin mcvm_plugin_stats

install.server_restart: build.server_restart
	@cargo install --path . --bin mcvm_plugin_server_restart --offline
	@cp plugins/server_restart/plugin.json ~/.local/share/mcvm/plugins/server_restart.json

build.server_restart: plugins/server_restart
	@cargo build -r --bin mcvm_plugin_server_restart

install.modrinth: build.modrinth
	@cargo install --path . --bin mcvm_plugin_modrinth --offline
	@cp plugins/modrinth/plugin.json ~/.local/share/mcvm/plugins/modrinth.json

build.modrinth: plugins/modrinth
	@cargo build -r --bin mcvm_plugin_modrinth

install.gen_pkg: build.gen_pkg
	@cargo install --path . --bin mcvm_plugin_gen_pkg --offline
	@cp plugins/gen_pkg/plugin.json ~/.local/share/mcvm/plugins/gen_pkg.json

build.gen_pkg: plugins/gen_pkg
	@cargo build -r --bin mcvm_plugin_gen_pkg
