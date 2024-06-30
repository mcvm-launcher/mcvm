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

install.args:
	@cargo install --path . --bin mcvm_plugin_args --offline
	@cp plugins/args/plugin.json ~/.local/share/mcvm/plugins/args.json

install.backup:
	@cargo install --path . --bin mcvm_plugin_backup --offline
	@cp plugins/backup/plugin.json ~/.local/share/mcvm/plugins/backup.json

install.options:
	@cargo install --path . --bin mcvm_plugin_options --offline
	@cp plugins/options/plugin.json ~/.local/share/mcvm/plugins/options.json

install.docs:
	@cargo install --path . --bin mcvm_plugin_docs --offline
	@cp plugins/docs/plugin.json ~/.local/share/mcvm/plugins/docs.json

install.stats:
	@cargo install --path . --bin mcvm_plugin_stats --offline
	@cp plugins/stats/plugin.json ~/.local/share/mcvm/plugins/stats.json

install.server_restart:
	@cargo install --path . --bin mcvm_plugin_server_restart --offline
	@cp plugins/server_restart/plugin.json ~/.local/share/mcvm/plugins/server_restart.json

install.modrinth:
	@cargo install --path . --bin mcvm_plugin_modrinth --offline
	@cp plugins/modrinth/plugin.json ~/.local/share/mcvm/plugins/modrinth.json

install.gen_pkg:
	@cargo install --path . --bin mcvm_plugin_gen_pkg --offline
	@cp plugins/gen_pkg/plugin.json ~/.local/share/mcvm/plugins/gen_pkg.json
