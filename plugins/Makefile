TARGET_DIR ?= "../target/release"

package: package.platform_dependent package.cross_platform

package.cross_platform: build.cross_platform
	@mkdir -p release

	@rm -f release/auto_mcs-universal.zip
	@rm -f release/automate-universal.zip
	@rm -f release/backup-universal.zip
	@rm -f release/better_jsons-universal.zip
	@rm -f release/completions-universal.zip
	@rm -f release/config_split-universal.zip
	@rm -f release/custom_files-universal.zip
	@rm -f release/doctor-universal.zip
	@rm -f release/extra_versions-universal.zip
	@rm -f release/gamepad-universal.zip
	@rm -f release/glfw_fix-universal.zip
	@rm -f release/graalvm-universal.zip
	@rm -f release/lang-universal.zip
	@rm -f release/mojang_transfer-universal.zip
	@rm -f release/multiply-universal.zip
	@rm -f release/nitro_transfer-universal.zip
	@rm -f release/octane-universal.zip
	@rm -f release/purpur-universal.zip
	@rm -f release/share-universal.zip
	@rm -f release/shortcut-universal.zip
	@rm -f release/skin_stealer-universal.zip
	@rm -f release/stats-universal.zip
	@rm -f release/themes-universal.zip
	@rm -f release/webtools-universal.zip
	@rm -f release/weld-universal.zip
	@rm -f release/zulu-universal.zip

	@zip -j release/auto_mcs-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_auto_mcs.wasm plugins/auto_mcs/plugin.json
	@printf "@ nitro_plugin_auto_mcs.wasm\\n@=plugin.wasm\\n" | zipnote -w release/auto_mcs-universal.zip
	@zip -j release/automate-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_automate.wasm plugins/automate/plugin.json
	@printf "@ nitro_plugin_automate.wasm\\n@=plugin.wasm\\n" | zipnote -w release/automate-universal.zip
	@zip -j release/backup-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_backup.wasm plugins/backup/plugin.json
	@printf "@ nitro_plugin_backup.wasm\\n@=plugin.wasm\\n" | zipnote -w release/backup-universal.zip
	@zip -j release/better_jsons-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_better_jsons.wasm plugins/better_jsons/plugin.json
	@printf "@ nitro_plugin_better_jsons.wasm\\n@=plugin.wasm\\n" | zipnote -w release/better_jsons-universal.zip
	@zip -j release/config_split-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_config_split.wasm plugins/config_split/plugin.json
	@printf "@ nitro_plugin_config_split.wasm\\n@=plugin.wasm\\n" | zipnote -w release/config_split-universal.zip

	@zip -j release/completions-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_completions.wasm plugins/completions/plugin.json plugins/completions/zsh/_nitro
	@printf "@ nitro_plugin_completions.wasm\\n@=plugin.wasm\\n" | zipnote -w release/completions-universal.zip
	@printf "@ _nitro\\n@=completions/zsh/_nitro\\n" | zipnote -w release/completions-universal.zip
	
	@zip -j release/custom_files-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_custom_files.wasm plugins/custom_files/plugin.json
	@printf "@ nitro_plugin_custom_files.wasm\\n@=plugin.wasm\\n" | zipnote -w release/custom_files-universal.zip
	@zip -j release/doctor-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_doctor.wasm plugins/doctor/plugin.json
	@printf "@ nitro_plugin_doctor.wasm\\n@=plugin.wasm\\n" | zipnote -w release/doctor-universal.zip
	@zip -j release/glfw_fix-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_glfw_fix.wasm plugins/glfw_fix/plugin.json
	@printf "@ nitro_plugin_glfw_fix.wasm\\n@=plugin.wasm\\n" | zipnote -w release/glfw_fix-universal.zip
	@zip -j release/graalvm-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_graalvm.wasm plugins/graalvm/plugin.json
	@printf "@ nitro_plugin_graalvm.wasm\\n@=plugin.wasm\\n" | zipnote -w release/graalvm-universal.zip
	@zip -j release/mojang_transfer-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_mojang_transfer.wasm plugins/mojang_transfer/plugin.json
	@printf "@ nitro_plugin_mojang_transfer.wasm\\n@=plugin.wasm\\n" | zipnote -w release/mojang_transfer-universal.zip
	@zip -j release/multiply-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_multiply.wasm plugins/multiply/plugin.json
	@printf "@ nitro_plugin_multiply.wasm\\n@=plugin.wasm\\n" | zipnote -w release/multiply-universal.zip
	@zip -j release/nitro_transfer-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_nitro_transfer.wasm plugins/nitro_transfer/plugin.json
	@printf "@ nitro_plugin_nitro_transfer.wasm\\n@=plugin.wasm\\n" | zipnote -w release/nitro_transfer-universal.zip
	@zip -j release/octane-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_octane.wasm plugins/octane/plugin.json
	@printf "@ nitro_plugin_octane.wasm\\n@=plugin.wasm\\n" | zipnote -w release/octane-universal.zip
	@zip -j release/purpur-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_purpur.wasm plugins/purpur/plugin.json
	@printf "@ nitro_plugin_purpur.wasm\\n@=plugin.wasm\\n" | zipnote -w release/purpur-universal.zip

	@zip -j release/share-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_share.wasm plugins/share/plugin.json plugins/share/export_template.js plugins/share/import_template.js
	@printf "@ nitro_plugin_share.wasm\\n@=plugin.wasm\\n" | zipnote -w release/share-universal.zip

	@zip -j release/shortcut-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_shortcut.wasm plugins/shortcut/plugin.json
	@printf "@ nitro_plugin_shortcut.wasm\\n@=plugin.wasm\\n" | zipnote -w release/shortcut-universal.zip
	@zip -j release/skin_stealer-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_skin_stealer.wasm plugins/skin_stealer/plugin.json
	@printf "@ nitro_plugin_skin_stealer.wasm\\n@=plugin.wasm\\n" | zipnote -w release/skin_stealer-universal.zip
	@zip -j release/stats-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_stats.wasm plugins/stats/plugin.json
	@printf "@ nitro_plugin_stats.wasm\\n@=plugin.wasm\\n" | zipnote -w release/stats-universal.zip
	@zip -j release/webtools-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_webtools.wasm plugins/webtools/plugin.json
	@printf "@ nitro_plugin_webtools.wasm\\n@=plugin.wasm\\n" | zipnote -w release/webtools-universal.zip
	@zip -j release/zulu-universal.zip ../target/wasm32-wasip2/release/nitro_plugin_zulu.wasm plugins/zulu/plugin.json
	@printf "@ nitro_plugin_zulu.wasm\\n@=plugin.wasm\\n" | zipnote -w release/zulu-universal.zip

	@zip -j release/extra_versions-universal.zip plugins/extra_versions/plugin.json
	@zip -j release/gamepad-universal.zip plugins/gamepad/plugin.json plugins/gamepad/gamepad.js
	@zip -j release/lang-universal.zip plugins/lang/plugin.json
	@zip -j release/themes-universal.zip plugins/themes/**
	@zip -j release/weld-universal.zip plugins/weld/plugin.json plugins/weld/weld.pyz

build.cross_platform: build.weld
	@cargo build -p "nitro_plugin_*" --release --target wasm32-wasip2

package.platform_dependent: build.platform_dependent package_zips.platform_dependent

build.platform_dependent:
	@cargo build --release --bins

package_zips.platform_dependent:
	@mkdir -p release
	
	@rm -f release/beet-${OS}.zip
	@rm -f release/cleanup-${OS}.zip
	@rm -f release/curseforge_api-${OS}.zip
	@rm -f release/discord_status-${OS}.zip
	@rm -f release/docs-${OS}.zip
	@rm -f release/fabric_quilt-${OS}.zip
	@rm -f release/forge-${OS}.zip
	@rm -f release/gen_pkg-${OS}.zip
	@rm -f release/guardian-${OS}.zip
	@rm -f release/modrinth-${OS}.zip
	@rm -f release/modrinth_api-${OS}.zip
	@rm -f release/multimc_transfer-${OS}.zip
	@rm -f release/options-${OS}.zip
	@rm -f release/packhost-${OS}.zip
	@rm -f release/paper-${OS}.zip
	@rm -f release/server_restart-${OS}.zip
	@rm -f release/smithed-${OS}.zip
	@rm -f release/smithed_api-${OS}.zip
	@rm -f release/sponge-${OS}.zip
	@rm -f release/weld-${OS}.zip
	@rm -f release/xmcl_transfer-${OS}.zip

	@zip -j release/beet-${OS}.zip $(TARGET_DIR)/nitro_plugin_beet${EXTENSION} plugins/beet/plugin.json
	@zip -j release/cleanup-${OS}.zip $(TARGET_DIR)/nitro_plugin_cleanup${EXTENSION} plugins/cleanup/plugin.json
	@zip -j release/curseforge_api-${OS}.zip $(TARGET_DIR)/nitro_plugin_curseforge_api${EXTENSION} plugins/curseforge_api/plugin.json
	@zip -j release/discord_status-${OS}.zip $(TARGET_DIR)/nitro_plugin_discord_status${EXTENSION} plugins/discord_status/plugin.json
	@zip -j release/docs-${OS}.zip $(TARGET_DIR)/nitro_plugin_docs${EXTENSION} plugins/docs/plugin.json
	@zip -j release/fabric_quilt-${OS}.zip $(TARGET_DIR)/nitro_plugin_fabric_quilt${EXTENSION} plugins/fabric_quilt/plugin.json
	@zip -j release/forge-${OS}.zip $(TARGET_DIR)/nitro_plugin_forge${EXTENSION} plugins/forge/plugin.json
	@zip -j release/gen_pkg-${OS}.zip $(TARGET_DIR)/nitro_plugin_gen_pkg${EXTENSION} plugins/gen_pkg/plugin.json
	@zip -j release/guardian-${OS}.zip $(TARGET_DIR)/nitro_plugin_guardian${EXTENSION} plugins/guardian/plugin.json
	@zip -j release/modrinth-${OS}.zip $(TARGET_DIR)/nitro_plugin_modrinth${EXTENSION} plugins/modrinth/plugin.json plugins/modrinth/icon.svg
	@zip -j release/modrinth_api-${OS}.zip $(TARGET_DIR)/nitro_plugin_modrinth_api${EXTENSION} plugins/modrinth_api/plugin.json plugins/modrinth_api/page.html
	@zip -j release/multimc_transfer-${OS}.zip $(TARGET_DIR)/nitro_plugin_multimc_transfer${EXTENSION} plugins/multimc_transfer/plugin.json
	@zip -j release/options-${OS}.zip $(TARGET_DIR)/nitro_plugin_options${EXTENSION} plugins/options/plugin.json
	@zip -j release/packhost-${OS}.zip $(TARGET_DIR)/nitro_plugin_packhost${EXTENSION} plugins/packhost/plugin.json
	@zip -j release/paper-${OS}.zip $(TARGET_DIR)/nitro_plugin_paper${EXTENSION} plugins/paper/plugin.json
	@zip -j release/server_restart-${OS}.zip $(TARGET_DIR)/nitro_plugin_server_restart${EXTENSION} plugins/server_restart/plugin.json
	@zip -j release/smithed-${OS}.zip $(TARGET_DIR)/nitro_plugin_smithed${EXTENSION} plugins/smithed/plugin.json plugins/smithed/page.html plugins/smithed/icon.svg
	@zip -j release/smithed_api-${OS}.zip $(TARGET_DIR)/nitro_plugin_smithed_api${EXTENSION} plugins/smithed_api/plugin.json
	@zip -j release/sponge-${OS}.zip $(TARGET_DIR)/nitro_plugin_sponge${EXTENSION} plugins/sponge/plugin.json
	@zip -j release/xmcl_transfer-${OS}.zip $(TARGET_DIR)/nitro_plugin_xmcl_transfer${EXTENSION} plugins/xmcl_transfer/plugin.json

build.weld:
	@shiv -o plugins/weld/weld.pyz -e nitro_weld.main --site-packages plugins/weld smithed

install.auto_mcs:
	@cargo build -p nitro_plugin_auto_mcs --release --target wasm32-wasip2
	@cp -r plugins/auto_mcs ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_auto_mcs.wasm ~/.local/share/nitro/plugins/auto_mcs/plugin.wasm

install.automate:
	@cargo build -p nitro_plugin_automate --release --target wasm32-wasip2
	@cp -r plugins/automate ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_automate.wasm ~/.local/share/nitro/plugins/automate/plugin.wasm

install.backup:
	@cargo build -p nitro_plugin_backup --release --target wasm32-wasip2
	@cp -r plugins/backup ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_backup.wasm ~/.local/share/nitro/plugins/backup/plugin.wasm

install.beet:
	@cargo install --path . --bin nitro_plugin_beet --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/beet
	@cat plugins/beet/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/beet/plugin.json

install.better_jsons:
	@cargo build -p nitro_plugin_better_jsons --release --target wasm32-wasip2
	@cp -r plugins/better_jsons ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_better_jsons.wasm ~/.local/share/nitro/plugins/better_jsons/plugin.wasm

install.cleanup:
	@cargo install --path . --bin nitro_plugin_cleanup --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/cleanup
	@cat plugins/cleanup/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/cleanup/plugin.json

install.completions:
	@cargo build -p nitro_plugin_completions --release --target wasm32-wasip2
	@cp -r plugins/completions ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_completions.wasm ~/.local/share/nitro/plugins/completions/plugin.wasm

install.config_split:
	@cargo build -p nitro_plugin_config_split --release --target wasm32-wasip2
	@cp -r plugins/config_split ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_config_split.wasm ~/.local/share/nitro/plugins/config_split/plugin.wasm

install.curseforge:
	@cargo install --path . --bin nitro_plugin_curseforge --locked --profile fast_release
	@cp -r plugins/curseforge ~/.local/share/nitro/plugins
	@cat plugins/curseforge/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/curseforge/plugin.json

install.curseforge_api:
	@cargo install --path . --bin nitro_plugin_curseforge_api --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/curseforge_api
	@cat plugins/curseforge_api/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/curseforge_api/plugin.json

install.custom_files:
	@cargo build -p nitro_plugin_custom_files --release --target wasm32-wasip2
	@cp -r plugins/custom_files ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_custom_files.wasm ~/.local/share/nitro/plugins/custom_files/plugin.wasm

install.discord_status:
	@cargo install --path . --bin nitro_plugin_discord_status --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/discord_status
	@cat plugins/discord_status/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/discord_status/plugin.json

install.docs:
	@cargo install --path . --bin nitro_plugin_docs --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/docs
	@cat plugins/docs/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/docs/plugin.json

install.doctor:
	@cargo build -p nitro_plugin_doctor --release --target wasm32-wasip2
	@cp -r plugins/doctor ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_doctor.wasm ~/.local/share/nitro/plugins/doctor/plugin.wasm
	
install.extra_versions:
	@cp -r plugins/extra_versions ~/.local/share/nitro/plugins

install.fabric_quilt:
	@cargo install --path . --bin nitro_plugin_fabric_quilt --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/fabric_quilt
	@cat plugins/fabric_quilt/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/fabric_quilt/plugin.json

install.forge:
	@cargo install --path . --bin nitro_plugin_forge --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/forge
	@cat plugins/forge/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/forge/plugin.json

install.gamepad:
	@cp -r plugins/gamepad ~/.local/share/nitro/plugins

install.gen_pkg:
	@cargo install --path . --bin nitro_plugin_gen_pkg --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/gen_pkg
	@cat plugins/gen_pkg/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/gen_pkg/plugin.json

install.glfw_fix:
	@cargo build -p nitro_plugin_glfw_fix --release --target wasm32-wasip2
	@cp -r plugins/glfw_fix ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_glfw_fix.wasm ~/.local/share/nitro/plugins/glfw_fix/plugin.wasm

install.graalvm:
	@cargo build -p nitro_plugin_graalvm --release --target wasm32-wasip2
	@cp -r plugins/graalvm ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_graalvm.wasm ~/.local/share/nitro/plugins/graalvm/plugin.wasm

install.guardian:
	@cargo install --path . --bin nitro_plugin_guardian --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/guardian
	@cat plugins/guardian/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/guardian/plugin.json

install.lang:
	@cp -r plugins/lang ~/.local/share/nitro/plugins

install.modrinth:
	@cargo install --path . --bin nitro_plugin_modrinth --locked --profile fast_release
	@cp -r plugins/modrinth ~/.local/share/nitro/plugins
	@cat plugins/modrinth/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/modrinth/plugin.json

install.modrinth_api:
	@cargo install --path . --bin nitro_plugin_modrinth_api --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/modrinth_api
	@cat plugins/modrinth_api/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/modrinth_api/plugin.json

install.mojang_transfer:
	@cargo build -p nitro_plugin_mojang_transfer --release --target wasm32-wasip2
	@cp -r plugins/mojang_transfer ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_mojang_transfer.wasm ~/.local/share/nitro/plugins/mojang_transfer/plugin.wasm

install.multimc_transfer:
	@cargo install --path . --bin nitro_plugin_multimc_transfer --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/multimc_transfer
	@cat plugins/multimc_transfer/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/multimc_transfer/plugin.json

install.multiply:
	@cargo build -p nitro_plugin_multiply --release --target wasm32-wasip2
	@cp -r plugins/multiply ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_multiply.wasm ~/.local/share/nitro/plugins/multiply/plugin.wasm

install.nitro_transfer:
	@cargo build -p nitro_plugin_nitro_transfer --release --target wasm32-wasip2
	@cp -r plugins/nitro_transfer ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_nitro_transfer.wasm ~/.local/share/nitro/plugins/nitro_transfer/plugin.wasm

install.octane:
	@cargo build -p nitro_plugin_octane --release --target wasm32-wasip2
	@cp -r plugins/octane ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_octane.wasm ~/.local/share/nitro/plugins/octane/plugin.wasm

install.options:
	@cargo install --path . --bin nitro_plugin_options --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/options
	@cat plugins/options/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/options/plugin.json

install.packhost:
	@cargo install --path . --bin nitro_plugin_packhost --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/packhost
	@cat plugins/packhost/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/packhost/plugin.json

install.paper:
	@cargo install --path . --bin nitro_plugin_paper --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/paper
	@cat plugins/paper/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/paper/plugin.json

install.purpur:
	@cargo build -p nitro_plugin_purpur --release --target wasm32-wasip2
	@cp -r plugins/purpur ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_purpur.wasm ~/.local/share/nitro/plugins/purpur/plugin.wasm

install.server_restart:
	@cargo install --path . --bin nitro_plugin_server_restart --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/server_restart
	@cat plugins/server_restart/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/server_restart/plugin.json

install.share:
	@cargo build -p nitro_plugin_share --release --target wasm32-wasip2
	@cp -r plugins/share ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_share.wasm ~/.local/share/nitro/plugins/share/plugin.wasm

install.shortcut:
	@cargo build -p nitro_plugin_shortcut --release --target wasm32-wasip2
	@cp -r plugins/shortcut ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_shortcut.wasm ~/.local/share/nitro/plugins/shortcut/plugin.wasm

install.skin_stealer:
	@cargo build -p nitro_plugin_skin_stealer --release --target wasm32-wasip2
	@cp -r plugins/skin_stealer ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_skin_stealer.wasm ~/.local/share/nitro/plugins/skin_stealer/plugin.wasm

install.smithed:
	@cargo install --path . --bin nitro_plugin_smithed --locked --profile fast_release
	@cp -r plugins/smithed ~/.local/share/nitro/plugins
	@cat plugins/smithed/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/smithed/plugin.json

install.smithed_api:
	@cargo install --path . --bin nitro_plugin_smithed_api --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/smithed_api
	@cat plugins/smithed_api/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/smithed_api/plugin.json

install.sponge:
	@cargo install --path . --bin nitro_plugin_sponge --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/sponge
	@cat plugins/sponge/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/sponge/plugin.json

install.stats:
	@cargo build -p nitro_plugin_stats --release --target wasm32-wasip2
	@cp -r plugins/stats ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_stats.wasm ~/.local/share/nitro/plugins/stats/plugin.wasm
	
install.template_share:
	@cargo install --path . --bin nitro_plugin_template_share --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/template_share
	@cat plugins/template_share/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/template_share/plugin.json
	@cp plugins/template_share/import.js ~/.local/share/nitro/plugins/template_share
	@cp plugins/template_share/export.js ~/.local/share/nitro/plugins/template_share

install.themes:
	@cp -r plugins/themes ~/.local/share/nitro/plugins

install.webtools:
	@cargo build -p nitro_plugin_webtools --release --target wasm32-wasip2
	@cp -r plugins/webtools ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_webtools.wasm ~/.local/share/nitro/plugins/webtools/plugin.wasm

install.weld:
	@mkdir -p ~/.local/share/nitro/plugins/weld
	@cp plugins/weld/nitro_weld.py ~/.local/share/nitro/plugins/weld
	@cp plugins/weld/local_plugin.json ~/.local/share/nitro/plugins/weld/plugin.json

install.xmcl_transfer:
	@cargo install --path . --bin nitro_plugin_xmcl_transfer --locked --profile fast_release
	@mkdir -p ~/.local/share/nitro/plugins/xmcl_transfer
	@cat plugins/xmcl_transfer/plugin.json | sed 's,\$${PLUGIN_DIR}/,,' > ~/.local/share/nitro/plugins/xmcl_transfer/plugin.json

install.zulu:
	@cargo build -p nitro_plugin_zulu --release --target wasm32-wasip2
	@cp -r plugins/zulu ~/.local/share/nitro/plugins
	@cp -r ../target/wasm32-wasip2/release/nitro_plugin_zulu.wasm ~/.local/share/nitro/plugins/zulu/plugin.wasm
