package: build.weld
	@cargo build --release --bin mcvm_plugin_archive
	@cargo build --release --bin mcvm_plugin_args
	@cargo build --release --bin mcvm_plugin_backup
	@cargo build --release --bin mcvm_plugin_config_split
	@cargo build --release --bin mcvm_plugin_custom_files
	@cargo build --release --bin mcvm_plugin_docs
	@cargo build --release --bin mcvm_plugin_fabric_quilt
	@cargo build --release --bin mcvm_plugin_gen_pkg
	@cargo build --release --bin mcvm_plugin_mcvm_transfer
	@cargo build --release --bin mcvm_plugin_modrinth
	@cargo build --release --bin mcvm_plugin_modrinth_api
	@cargo build --release --bin mcvm_plugin_options
	@cargo build --release --bin mcvm_plugin_packhost
	@cargo build --release --bin mcvm_plugin_paper
	@cargo build --release --bin mcvm_plugin_scripthook
	@cargo build --release --bin mcvm_plugin_server_restart
	@cargo build --release --bin mcvm_plugin_smithed
	@cargo build --release --bin mcvm_plugin_smithed_api
	@cargo build --release --bin mcvm_plugin_sponge
	@cargo build --release --bin mcvm_plugin_stats
	@cargo build --release --bin mcvm_plugin_webtools
	@cargo build --release --bin mcvm_plugin_xmcl_transfer

	@mkdir -p release

	@rm -f release/archive-linux.zip && zip -j release/archive-linux.zip ../target/release/mcvm_plugin_archive plugins/archive/plugin.json
	@rm -f release/args-linux.zip && zip -j release/args-linux.zip ../target/release/mcvm_plugin_args plugins/args/plugin.json
	@rm -f release/backup-linux.zip && zip -j release/backup-linux.zip ../target/release/mcvm_plugin_backup plugins/backup/plugin.json
	@rm -f release/config_split-linux.zip && zip -j release/config_split-linux.zip ../target/release/mcvm_plugin_config_split plugins/config_split/plugin.json
	@rm -f release/custom_files-linux.zip && zip -j release/custom_files-linux.zip ../target/release/mcvm_plugin_custom_files plugins/custom_files/plugin.json
	@rm -f release/docs-linux.zip && zip -j release/docs-linux.zip ../target/release/mcvm_plugin_docs plugins/docs/plugin.json
	@rm -f release/fabric_quilt-linux.zip && zip -j release/fabric_quilt-linux.zip ../target/release/mcvm_plugin_fabric_quilt plugins/fabric_quilt/plugin.json
	@rm -f release/gen_pkg-linux.zip && zip -j release/gen_pkg-linux.zip ../target/release/mcvm_plugin_gen_pkg plugins/gen_pkg/plugin.json
	@rm -f release/mcvm_transfer-linux.zip && zip -j release/mcvm_transfer-linux.zip ../target/release/mcvm_plugin_mcvm_transfer plugins/mcvm_transfer/plugin.json
	@rm -f release/modrinth-linux.zip && zip -j release/modrinth-linux.zip ../target/release/mcvm_plugin_modrinth plugins/modrinth/plugin.json
	@rm -f release/modrinth_api-linux.zip && zip -j release/modrinth_api-linux.zip ../target/release/mcvm_plugin_modrinth_api plugins/modrinth_api/plugin.json plugins/modrinth_api/page.html
	@rm -f release/options-linux.zip && zip -j release/options-linux.zip ../target/release/mcvm_plugin_options plugins/options/plugin.json
	@rm -f release/packhost-linux.zip && zip -j release/packhost-linux.zip ../target/release/mcvm_plugin_packhost plugins/packhost/plugin.json
	@rm -f release/paper-linux.zip && zip -j release/paper-linux.zip ../target/release/mcvm_plugin_paper plugins/paper/plugin.json
	@rm -f release/scripthook-linux.zip && zip -j release/scripthook-linux.zip ../target/release/mcvm_plugin_scripthook plugins/scripthook/plugin.json
	@rm -f release/server_restart-linux.zip && zip -j release/server_restart-linux.zip ../target/release/mcvm_plugin_server_restart plugins/server_restart/plugin.json
	@rm -f release/smithed-linux.zip && zip -j release/smithed-linux.zip ../target/release/mcvm_plugin_smithed plugins/smithed/plugin.json plugins/smithed/page.html
	@rm -f release/smithed_api-linux.zip && zip -j release/smithed_api-linux.zip ../target/release/mcvm_plugin_smithed_api plugins/smithed_api/plugin.json
	@rm -f release/sponge-linux.zip && zip -j release/sponge-linux.zip ../target/release/mcvm_plugin_sponge plugins/sponge/plugin.json
	@rm -f release/stats-linux.zip && zip -j release/stats-linux.zip ../target/release/mcvm_plugin_stats plugins/stats/plugin.json
	@rm -f release/webtools-linux.zip && zip -j release/webtools-linux.zip ../target/release/mcvm_plugin_webtools plugins/webtools/plugin.json
	@rm -f release/weld-linux.zip && zip -j release/weld-linux.zip plugins/weld/plugin.json plugins/weld/weld.pyz
	@rm -f release/xmcl_transfer-linux.zip && zip -j release/xmcl_transfer-linux.zip ../target/release/mcvm_plugin_xmcl_transfer plugins/xmcl_transfer/plugin.json
	
	@cp plugins/extra_versions/plugin.json release/extra_versions-universal.json
	@cp plugins/lang/plugin.json release/lang-universal.json

build.weld:
	@shiv -o plugins/weld/weld.pyz -e mcvm_weld.main --site-packages plugins/weld smithed

install.archive:
	@cargo install --path . --bin mcvm_plugin_archive --offline
	@cat plugins/archive/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/archive.json

install.args:
	@cargo install --path . --bin mcvm_plugin_args --offline
	@cat plugins/args/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/args.json

install.backup:
	@cargo install --path . --bin mcvm_plugin_backup --offline
	@cat plugins/backup/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/backup.json

install.config_split:
	@cargo install --path . --bin mcvm_plugin_config_split --offline
	@cat plugins/config_split/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/config_split.json

install.custom_files:
	@cargo install --path . --bin mcvm_plugin_custom_files --offline
	@cat plugins/custom_files/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/custom_files.json

install.docs:
	@cargo install --path . --bin mcvm_plugin_docs --offline
	@cat plugins/docs/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/docs.json
	
install.extra_versions:
	@cp plugins/extra_versions/plugin.json ~/.local/share/mcvm/plugins/extra_versions.json

install.fabric_quilt:
	@cargo install --path . --bin mcvm_plugin_fabric_quilt --offline
	@cat plugins/fabric_quilt/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/fabric_quilt.json

install.gen_pkg:
	@cargo install --path . --bin mcvm_plugin_gen_pkg --offline
	@cat plugins/gen_pkg/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/gen_pkg.json

install.lang:
	@cp plugins/lang/plugin.json ~/.local/share/mcvm/plugins/lang.json

install.mcvm_transfer:
	@cargo install --path . --bin mcvm_plugin_mcvm_transfer --offline
	@cat plugins/mcvm_transfer/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/mcvm_transfer.json

install.modrinth:
	@cargo install --path . --bin mcvm_plugin_modrinth --offline
	@cp -r plugins/modrinth ~/.local/share/mcvm/plugins
	@cat plugins/modrinth/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/modrinth/plugin.json

install.modrinth_api:
	@cargo install --path . --bin mcvm_plugin_modrinth_api --offline
	@cat plugins/modrinth_api/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/modrinth_api.json

install.options:
	@cargo install --path . --bin mcvm_plugin_options --offline
	@cat plugins/options/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/options.json

install.packhost:
	@cargo install --path . --bin mcvm_plugin_packhost --offline
	@cat plugins/packhost/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/packhost.json

install.paper:
	@cargo install --path . --bin mcvm_plugin_paper --offline
	@cat plugins/paper/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/paper.json

install.scripthook:
	@cargo install --path . --bin mcvm_plugin_scripthook --offline
	@cat plugins/scripthook/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/scripthook.json

install.server_restart:
	@cargo install --path . --bin mcvm_plugin_server_restart --offline
	@cat plugins/server_restart/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/server_restart.json

install.smithed:
	@cargo install --path . --bin mcvm_plugin_smithed --offline
	@cp -r plugins/smithed ~/.local/share/mcvm/plugins
	@cat plugins/smithed/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/smithed/plugin.json

install.smithed_api:
	@cargo install --path . --bin mcvm_plugin_smithed_api --offline
	@cat plugins/smithed_api/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/smithed_api.json

install.sponge:
	@cargo install --path . --bin mcvm_plugin_sponge --offline
	@cat plugins/sponge/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/sponge.json

install.stats:
	@cargo install --path . --bin mcvm_plugin_stats --offline
	@cat plugins/stats/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/stats.json

install.webtools:
	@cargo install --path . --bin mcvm_plugin_webtools --offline
	@cp -r plugins/webtools ~/.local/share/mcvm/plugins
	@cat plugins/webtools/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/webtools/plugin.json

install.weld:
	@mkdir -p ~/.local/share/mcvm/plugins/weld
	@cp plugins/weld/mcvm_weld.py ~/.local/share/mcvm/plugins/weld
	@cp plugins/weld/local_plugin.json ~/.local/share/mcvm/plugins/weld/plugin.json

install.xmcl_transfer:
	@cargo install --path . --bin mcvm_plugin_xmcl_transfer --offline
	@cat plugins/xmcl_transfer/plugin.json | sed 's,\${PLUGIN_DIR}/,,' > ~/.local/share/mcvm/plugins/xmcl_transfer.json
