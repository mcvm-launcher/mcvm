package:
	@cargo build --release --bin mcvm_plugin_args
	@cargo build --release --bin mcvm_plugin_backup
	@cargo build --release --bin mcvm_plugin_docs
	@cargo build --release --bin mcvm_plugin_gen_pkg
	@cargo build --release --bin mcvm_plugin_mcvm_transfer
	@cargo build --release --bin mcvm_plugin_modrinth
	@cargo build --release --bin mcvm_plugin_options
	@cargo build --release --bin mcvm_plugin_server_restart
	@cargo build --release --bin mcvm_plugin_stats

	@mkdir -p release

	@rm -f release/args-linux.zip && zip -j release/args-linux.zip ../target/release/mcvm_plugin_args plugins/args/plugin.json
	@rm -f release/backup-linux.zip && zip -j release/backup-linux.zip ../target/release/mcvm_plugin_backup plugins/backup/plugin.json
	@rm -f release/docs-linux.zip && zip -j release/docs-linux.zip ../target/release/mcvm_plugin_docs plugins/docs/plugin.json
	@rm -f release/gen_pkg-linux.zip && zip -j release/gen_pkg-linux.zip ../target/release/mcvm_plugin_gen_pkg plugins/gen_pkg/plugin.json
	@rm -f release/mcvm_transfer-linux.zip && zip -j release/mcvm_transfer-linux.zip ../target/release/mcvm_plugin_mcvm_transfer plugins/mcvm_transfer/plugin.json
	@rm -f release/modrinth-linux.zip && zip -j release/modrinth-linux.zip ../target/release/mcvm_plugin_modrinth plugins/modrinth/plugin.json
	@rm -f release/options-linux.zip && zip -j release/options-linux.zip ../target/release/mcvm_plugin_options plugins/options/plugin.json
	@rm -f release/server_restart-linux.zip && zip -j release/server_restart-linux.zip ../target/release/mcvm_plugin_server_restart plugins/server_restart/plugin.json
	@rm -f release/stats-linux.zip && zip -j release/stats-linux.zip ../target/release/mcvm_plugin_stats plugins/stats/plugin.json
	
	@cp plugins/extra_versions/plugin.json release/extra_versions-universal.json
	@cp plugins/lang/plugin.json release/lang-universal.json

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
	@cat plugins/args/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/args.json

install.backup:
	@cargo install --path . --bin mcvm_plugin_backup --offline
	@cat plugins/backup/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/backup.json

install.options:
	@cargo install --path . --bin mcvm_plugin_options --offline
	@cat plugins/options/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/options.json

install.docs:
	@cargo install --path . --bin mcvm_plugin_docs --offline
	@cat plugins/docs/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/docs.json

install.stats:
	@cargo install --path . --bin mcvm_plugin_stats --offline
	@cat plugins/stats/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/stats.json

install.server_restart:
	@cargo install --path . --bin mcvm_plugin_server_restart --offline
	@cat plugins/server_restart/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/server_restart.json

install.modrinth:
	@cargo install --path . --bin mcvm_plugin_modrinth --offline
	@cat plugins/modrinth/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/modrinth.json

install.gen_pkg:
	@cargo install --path . --bin mcvm_plugin_gen_pkg --offline
	@cat plugins/gen_pkg/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/gen_pkg.json

install.mcvm_transfer:
	@cargo install --path . --bin mcvm_plugin_mcvm_transfer --offline
	@cat plugins/mcvm_transfer/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/mcvm_transfer.json
