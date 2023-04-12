use std::{collections::HashMap, fs::File, io::Write, path::Path};

use anyhow::Context;
use itertools::Itertools;

use crate::{
	io::options::read::{read_options_file, EnumOrNumber},
	util::{versions::VersionPattern, ToInt},
};

use super::{ClientOptions, CloudRenderMode, FullscreenResolution, GraphicsMode};

static SEP: char = ':';

/// Write options.txt to a file
pub async fn write_options_txt(
	options: HashMap<String, String>,
	path: &Path,
) -> anyhow::Result<()> {
	let options = merge_options_txt(path, options)
		.await
		.context("Failed to merge with existing options.txt")?;
	let mut file = File::create(path).context("Failed to open file")?;
	for (key, value) in options.iter().sorted_by_key(|x| x.0) {
		write_key(key, value, &mut file)
			.with_context(|| format!("Failed to write line for option {key} with value {value}"))?;
	}

	Ok(())
}

/// Collect a hashmap from an existing options.txt file so we can compare with it
pub async fn read_options_txt(path: &Path) -> anyhow::Result<HashMap<String, String>> {
	if path.exists() {
		let contents = tokio::fs::read_to_string(path)
			.await
			.context("Failed to read options.txt")?;
		read_options_file(&contents, SEP)
	} else {
		Ok(HashMap::new())
	}
}

/// Merge keys with an existing file
pub async fn merge_options_txt(
	path: &Path,
	keys: HashMap<String, String>,
) -> anyhow::Result<HashMap<String, String>> {
	let mut file_keys = read_options_txt(path)
		.await
		.context("Failed to open options file for merging")?;
	file_keys.extend(keys);
	Ok(file_keys)
}

/// Write a options options key to a writer
pub fn write_key<W: Write>(key: &str, value: &str, writer: &mut W) -> anyhow::Result<()> {
	writeln!(writer, "{key}:{value}")?;

	Ok(())
}

/// Creates the string for the list of resource packs
fn write_resource_packs(resource_packs: &[String]) -> String {
	let names = resource_packs.iter().map(|x| format!("\"{x}\","));
	let mut names_joined = String::new();
	for name in names {
		names_joined.push_str(&name);
	}

	format!("[{names_joined}]")
}

/// Creates the string for fullscreen resolution
fn write_fullscreen_resolution(resolution: &FullscreenResolution) -> String {
	format!(
		"{}x{}@{}:{}",
		resolution.width.to_string(),
		resolution.height.to_string(),
		resolution.refresh_rate.to_string(),
		resolution.color_bits.to_string()
	)
}

/// Converts Field of View from the integer used in game and the options
/// to the format used in the options.txt. In-game the number is in degrees, but
/// it is a number from -1 to 1 in the options.txt. According to the wiki, the formula is
/// `degrees = 40 * value + 70`.
fn convert_fov(fov: u8) -> f32 {
	(fov as f32 - 70.0) / 40.0
}

/// Write options options to a list of keys
pub fn create_keys(
	options: &ClientOptions,
	version: &str,
	versions: &[String],
) -> anyhow::Result<HashMap<String, String>> {
	let mut out = HashMap::new();

	// Version checks
	let after_12w50a =
		VersionPattern::After(String::from("12w50a")).matches_single(version, versions);
	let after_13w36a =
		VersionPattern::After(String::from("13w36a")).matches_single(version, versions);
	let after_14w25a =
		VersionPattern::After(String::from("14w25a")).matches_single(version, versions);
	let after_14w28a =
		VersionPattern::After(String::from("14w28a")).matches_single(version, versions);
	let after_17w06a =
		VersionPattern::After(String::from("17w06a")).matches_single(version, versions);
	let after_17w47a =
		VersionPattern::After(String::from("17w47a")).matches_single(version, versions);
	let after_18w15a =
		VersionPattern::After(String::from("18w15a")).matches_single(version, versions);
	let after_18w21a =
		VersionPattern::After(String::from("18w21a")).matches_single(version, versions);
	let after_1_13_pre2 =
		VersionPattern::After(String::from("1.13-pre2")).matches_single(version, versions);
	let after_1_15_2_pre1 =
		VersionPattern::After(String::from("1.15.2-pre1")).matches_single(version, versions);
	let after_1_16_4_rc1 =
		VersionPattern::After(String::from("1.16.4-rc1")).matches_single(version, versions);
	let after_21w13a =
		VersionPattern::After(String::from("21w13a")).matches_single(version, versions);
	let after_21w37a =
		VersionPattern::After(String::from("21w37a")).matches_single(version, versions);
	let after_21w38a =
		VersionPattern::After(String::from("21w38a")).matches_single(version, versions);
	let after_21w42a =
		VersionPattern::After(String::from("21w42a")).matches_single(version, versions);
	let after_1_18_pre2 =
		VersionPattern::After(String::from("1.18-pre2")).matches_single(version, versions);
	let after_1_18_2_pre1 =
		VersionPattern::After(String::from("1.18.2-pre1")).matches_single(version, versions);
	let after_22w11a =
		VersionPattern::After(String::from("22w11a")).matches_single(version, versions);
	let after_22w15a =
		VersionPattern::After(String::from("22w15a")).matches_single(version, versions);

	let before_13w42a =
		VersionPattern::Before(String::from("13w42a")).matches_single(version, versions);
	let before_15w31a =
		VersionPattern::Before(String::from("15w31a")).matches_single(version, versions);
	let before_1_13 =
		VersionPattern::Before(String::from("1.13")).matches_single(version, versions);
	let before_20w27a =
		VersionPattern::Before(String::from("20w27a")).matches_single(version, versions);
	let before_1_19_4 =
		VersionPattern::Before(String::from("1.19.4")).matches_single(version, versions);

	// TODO: Add actual data version
	if let Some(value) = options.data_version {
		out.insert(String::from("version"), value.to_string());
	}
	if let Some(value) = options.control.auto_jump {
		out.insert(String::from("autoJump"), value.to_string());
	}
	if let Some(value) = options.video.fullscreen {
		out.insert(String::from("fullscreen"), value.to_string());
	}
	if let Some(value) = options.chat.auto_command_suggestions {
		if after_17w47a {
			out.insert(String::from("autoSuggestions"), value.to_string());
		}
	}
	if let Some(value) = options.chat.enable_colors {
		out.insert(String::from("chatColors"), value.to_string());
	}
	if let Some(value) = options.chat.enable_links {
		out.insert(String::from("chatLinks"), value.to_string());
	}
	if let Some(value) = options.chat.prompt_links {
		out.insert(String::from("chatLinksPrompt"), value.to_string());
	}
	if let Some(value) = options.video.vsync {
		out.insert(String::from("enableVsync"), value.to_string());
	}
	if let Some(value) = options.video.entity_shadows {
		out.insert(String::from("entityShadows"), value.to_string());
	}
	if let Some(value) = options.chat.force_unicode {
		out.insert(String::from("forceUnicodeFont"), value.to_string());
	}
	if let Some(value) = options.control.discrete_mouse_scroll {
		out.insert(String::from("discrete_mouse_scroll"), value.to_string());
	}
	if let Some(value) = options.control.invert_mouse_y {
		out.insert(String::from("invertYMouse"), value.to_string());
	}
	if let Some(value) = options.realms_notifications {
		out.insert(String::from("realmsNotifications"), value.to_string());
	}
	if let Some(value) = options.reduced_debug_info {
		out.insert(String::from("reducedDebugInfo"), value.to_string());
	}
	if let Some(value) = options.sound.show_subtitles {
		out.insert(String::from("showSubtitles"), value.to_string());
	}
	if let Some(value) = options.sound.directional_audio {
		if after_22w11a {
			out.insert(String::from("directionalAudio"), value.to_string());
		}
	}
	if let Some(value) = options.control.enable_touchscreen {
		out.insert(String::from("touchscreen"), value.to_string());
	}
	if let Some(value) = options.video.view_bobbing {
		out.insert(String::from("bobView"), value.to_string());
	}
	if let Some(value) = options.control.toggle_crouch {
		out.insert(String::from("toggleCrouch"), value.to_string());
	}
	if let Some(value) = options.control.toggle_sprint {
		out.insert(String::from("toggleSprint"), value.to_string());
	}
	if let Some(value) = options.video.dark_mojang_background {
		if after_21w13a {
			out.insert(
				String::from("darkMojangStudiosBackground"),
				value.to_string(),
			);
		}
	}
	if after_21w37a {
		if let Some(value) = options.video.hide_lightning_flashes {
			out.insert(String::from("hideLightningFlashes"), value.to_string());
		}
		if let Some(value) = &options.video.chunk_updates_mode {
			out.insert(
				String::from("prioritizeChunkUpdates"),
				value.to_int().to_string(),
			);
		}
		if let Some(device) = &options.sound.device {
			out.insert(String::from("soundDevice"), device.clone());
		}
	}
	if let Some(value) = options.control.mouse_sensitivity {
		out.insert(String::from("mouseSensitivity"), value.to_string());
	}
	if let Some(value) = options.video.fov {
		out.insert(String::from("fov"), convert_fov(value).to_string());
	}
	if let Some(value) = options.video.screen_effect_scale {
		out.insert(String::from("screenEffectScale"), value.to_string());
	}
	if let Some(value) = options.video.fov_effect_scale {
		out.insert(String::from("fovEffectScale"), value.to_string());
	}
	if let Some(value) = options.video.darkness_effect_scale {
		if after_22w15a {
			out.insert(String::from("darknessEffectScale"), value.to_string());
		}
	}
	if let Some(value) = options.video.brightness {
		out.insert(String::from("gamma"), value.to_string());
	}
	if let Some(value) = options.video.render_distance {
		out.insert(String::from("renderDistance"), value.to_string());
	}
	if let Some(value) = options.video.simulation_distance {
		if after_21w38a {
			out.insert(String::from("simulationDistance"), value.to_string());
		}
	}
	if let Some(value) = options.video.entity_distance_scaling {
		out.insert(String::from("entityDistanceScaling"), value.to_string());
	}
	if let Some(value) = options.video.gui_scale {
		out.insert(String::from("guiScale"), value.to_string());
	}
	if let Some(value) = &options.video.particles {
		out.insert(String::from("particles"), value.to_int().to_string());
	}
	if let Some(value) = options.video.max_fps {
		out.insert(String::from("maxFps"), value.to_string());
	}
	if let Some(value) = &options.difficulty {
		out.insert(String::from("difficulty"), value.to_int().to_string());
	}
	if let Some(value) = &options.video.graphics_mode {
		if before_20w27a {
			out.insert(
				String::from("fancyGraphics"),
				match value {
					EnumOrNumber::Enum(GraphicsMode::Fast) => false,
					EnumOrNumber::Enum(GraphicsMode::Fancy | GraphicsMode::Fabulous) => true,
					EnumOrNumber::Num(num) => num > &0,
				}
				.to_string(),
			);
		} else {
			out.insert(String::from("graphicsMode"), value.to_int().to_string());
		}
	}
	if let Some(value) = options.video.smooth_lighting {
		out.insert(String::from("ao"), value.to_string());
	}
	if let Some(value) = options.video.biome_blend {
		if after_18w15a {
			out.insert(String::from("biomeBlendRadius"), value.to_string());
		}
	}
	if let Some(value) = &options.video.clouds {
		if after_14w25a {
			out.insert(String::from("renderClouds"), value.to_string());
		} else {
			out.insert(
				String::from("clouds"),
				matches!(value, CloudRenderMode::Fancy | CloudRenderMode::Fast).to_string(),
			);
		}
	}
	if let Some(value) = &options.resource_packs {
		out.insert(String::from("resourcePacks"), write_resource_packs(&value));
	}
	if let Some(value) = &options.language {
		out.insert(String::from("lang"), value.clone());
	}
	if let Some(value) = &options.chat.visibility {
		out.insert(String::from("chatVisibility"), value.to_int().to_string());
	}
	if let Some(value) = options.chat.opacity {
		out.insert(String::from("chatOpacity"), value.to_string());
	}
	if let Some(value) = options.chat.line_spacing {
		out.insert(String::from("chatLineSpacing"), value.to_string());
	}
	if let Some(value) = options.chat.background_opacity {
		out.insert(String::from("textBackgroundOpacity"), value.to_string());
	}
	if let Some(value) = options.chat.background_for_chat_only {
		out.insert(String::from("backgroundForChatOnly"), value.to_string());
	}
	if let Some(value) = options.hide_server_address {
		out.insert(String::from("hideServerAddress"), value.to_string());
	}
	if let Some(value) = options.advanced_item_tooltips {
		out.insert(String::from("advancedItemTooltips"), value.to_string());
	}
	if let Some(value) = options.pause_on_lost_focus {
		out.insert(String::from("pauseOnLostFocus"), value.to_string());
	}
	if let Some(value) = options.video.window_width {
		out.insert(String::from("overrideWidth"), value.to_string());
	}
	if let Some(value) = options.video.window_height {
		out.insert(String::from("overrideHeight"), value.to_string());
	}
	if let Some(value) = options.held_item_tooltips {
		if after_12w50a && before_1_19_4 {
			out.insert(String::from("heldItemTooltips"), value.to_string());
		}
	}
	if let Some(value) = options.chat.focused_height {
		out.insert(String::from("chatHeightFocused"), value.to_string());
	}
	if let Some(value) = options.chat.delay {
		out.insert(String::from("chatDelay"), value.to_string());
	}
	if let Some(value) = options.chat.unfocused_height {
		out.insert(String::from("chatHeightUnfocused"), value.to_string());
	}
	if let Some(value) = options.chat.scale {
		out.insert(String::from("chatScale"), value.to_string());
	}
	if let Some(value) = options.chat.width {
		out.insert(String::from("chatWidth"), value.to_string());
	}
	if let Some(value) = options.video.mipmap_levels {
		out.insert(String::from("mipmapLevels"), value.to_string());
	}
	if let Some(value) = options.use_native_transport {
		out.insert(String::from("useNativeTransport"), value.to_string());
	}
	if let Some(value) = &options.main_hand {
		out.insert(String::from("mainHand"), value.to_string());
	}
	if after_17w06a {
		if let Some(value) = &options.chat.narrator_mode {
			out.insert(String::from("narrator"), value.to_int().to_string());
		}
		if let Some(value) = &options.tutorial_step {
			out.insert(String::from("tutorialStep"), value.to_string());
		}
	}
	if let Some(value) = options.control.mouse_wheel_sensitivity {
		if after_18w21a {
			out.insert(String::from("mouseWheelSensitivity"), value.to_string());
		}
	}
	if let Some(value) = options.control.raw_mouse_input {
		out.insert(String::from("rawMouseInput"), value.to_string());
	}
	if let Some(value) = &options.log_level {
		if after_1_13_pre2 {
			out.insert(String::from("glDebugVerbosity"), value.to_int().to_string());
		}
	}
	if let Some(value) = options.skip_multiplayer_warning {
		if after_1_15_2_pre1 {
			out.insert(String::from("skipMultiplayerWarning"), value.to_string());
		}
	}
	if let Some(value) = options.skip_realms_32_bit_warning {
		if after_1_18_2_pre1 {
			out.insert(String::from("skipRealms32bitWarning"), value.to_string());
		}
	}
	if after_1_16_4_rc1 {
		if let Some(value) = options.hide_matched_names {
			out.insert(String::from("hideMatchedNames"), value.to_string());
		}
		if let Some(value) = options.joined_server {
			out.insert(String::from("joinedFirstServer"), value.to_string());
		}
	}
	if let Some(value) = options.hide_bundle_tutorial {
		out.insert(String::from("hideBundleTutorial"), value.to_string());
	}
	if let Some(value) = options.sync_chunk_writes {
		out.insert(String::from("syncChunkWrites"), value.to_string());
	}
	if let Some(value) = options.show_autosave_indicator {
		if after_21w42a {
			out.insert(String::from("showAutosaveIndicator"), value.to_string());
		}
	}
	if let Some(value) = options.allow_server_listing {
		if after_1_18_pre2 {
			out.insert(String::from("allowServerListing"), value.to_string());
		}
	}
	// Keybinds
	if let Some(value) = &options.control.keys.attack {
		out.insert(
			String::from("key_key.attack"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.r#use {
		out.insert(String::from("key_key.use"), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.forward {
		out.insert(
			String::from("key_key.forward"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.left {
		out.insert(String::from("key_key.left"), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.back {
		out.insert(String::from("key_key.back"), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.right {
		out.insert(
			String::from("key_key.right"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.jump {
		out.insert(String::from("key_key.jump"), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.sneak {
		out.insert(
			String::from("key_key.sneak"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.sprint {
		out.insert(
			String::from("key_key.sprint"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.drop {
		out.insert(String::from("key_key.drop"), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.inventory {
		out.insert(
			String::from("key_key.inventory"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.chat {
		out.insert(String::from("key_key.chat"), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.playerlist {
		out.insert(
			String::from("key_key.playerlist"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.pick_item {
		out.insert(
			String::from("key_key.pickItem"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.command {
		out.insert(
			String::from("key_key.command"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.social_interactions {
		out.insert(
			String::from("key_key.socialInteractions"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.screenshot {
		out.insert(
			String::from("key_key.screenshot"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.toggle_perspective {
		out.insert(
			String::from("key_key.togglePerspective"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.smooth_camera {
		out.insert(
			String::from("key_key.smoothCamera"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.fullscreen {
		out.insert(
			String::from("key_key.fullscreen"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.spectator_outlines {
		out.insert(
			String::from("key_key.spectatorOutlines"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.swap_offhand {
		out.insert(
			String::from("key_key.swapOffhand"),
			value.get_keycode(before_1_13),
		);
	}
	if after_17w06a {
		if let Some(value) = &options.control.keys.save_toolbar {
			out.insert(
				String::from("key_key.saveToolbarActivator"),
				value.get_keycode(before_1_13),
			);
		}
		if let Some(value) = &options.control.keys.load_toolbar {
			out.insert(
				String::from("key_key.loadToolbarActivator"),
				value.get_keycode(before_1_13),
			);
		}
		if let Some(value) = &options.control.keys.advancements {
			out.insert(
				String::from("key_key.advancements"),
				value.get_keycode(before_1_13),
			);
		}
	}
	if let Some(value) = &options.control.keys.hotbar_1 {
		out.insert(
			String::from("key_key.hotbar.1"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.hotbar_2 {
		out.insert(
			String::from("key_key.hotbar.2"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.hotbar_3 {
		out.insert(
			String::from("key_key.hotbar.3"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.hotbar_4 {
		out.insert(
			String::from("key_key.hotbar.4"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.hotbar_5 {
		out.insert(
			String::from("key_key.hotbar.5"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.hotbar_6 {
		out.insert(
			String::from("key_key.hotbar.6"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.hotbar_7 {
		out.insert(
			String::from("key_key.hotbar.7"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.hotbar_8 {
		out.insert(
			String::from("key_key.hotbar.8"),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.hotbar_9 {
		out.insert(
			String::from("key_key.hotbar.9"),
			value.get_keycode(before_1_13),
		);
	}

	// Volumes
	if after_13w36a {
		let (animals_key, blocks_key, mobs_key, players_key, records_key) = {
			if before_13w42a {
				(
					"soundCategory_animals",
					"soundCategory_blocks",
					"soundCategory_mobs",
					"soundCategory_players",
					"soundCategory_records",
				)
			} else {
				(
					"soundCategory_neutral",
					"soundCategory_block",
					"soundCategory_hostile",
					"soundCategory_player",
					"soundCategory_record",
				)
			}
		};
		if let Some(value) = options.sound.volume.master {
			out.insert(String::from("soundCategory_master"), value.to_string());
		}
		if let Some(value) = options.sound.volume.music {
			out.insert(String::from("soundCategory_music"), value.to_string());
		}
		if let Some(value) = options.sound.volume.record {
			out.insert(String::from(records_key), value.to_string());
		}
		if let Some(value) = options.sound.volume.weather {
			out.insert(String::from("soundCategory_weather"), value.to_string());
		}
		if let Some(value) = options.sound.volume.block {
			out.insert(String::from(blocks_key), value.to_string());
		}
		if let Some(value) = options.sound.volume.hostile {
			out.insert(String::from(mobs_key), value.to_string());
		}
		if let Some(value) = options.sound.volume.neutral {
			out.insert(String::from(animals_key), value.to_string());
		}
		if let Some(value) = options.sound.volume.player {
			out.insert(String::from(players_key), value.to_string());
		}
		if let Some(value) = options.sound.volume.ambient {
			out.insert(String::from("soundCategory_ambient"), value.to_string());
		}
		if let Some(value) = options.sound.volume.voice {
			out.insert(String::from("soundCategory_voice"), value.to_string());
		}
	} else {
		if let Some(value) = options.sound.volume.master {
			let volume_up = value > 0.0;
			out.insert(String::from("sound"), volume_up.to_string());
		}
	}
	// Model parts
	if let Some(value) = options.skin.cape {
		out.insert(String::from("modelPart_cape"), value.to_string());
	}
	if let Some(value) = options.skin.jacket {
		out.insert(String::from("modelPart_jacket"), value.to_string());
	}
	if let Some(value) = options.skin.left_sleeve {
		out.insert(String::from("modelPart_left_sleeve"), value.to_string());
	}
	if let Some(value) = options.skin.right_sleeve {
		out.insert(String::from("modelPart_right_sleeve"), value.to_string());
	}
	if let Some(value) = options.skin.left_pants {
		out.insert(String::from("modelPart_left_pants_leg"), value.to_string());
	}
	if let Some(value) = options.skin.right_pants {
		out.insert(String::from("modelPart_right_pants_leg"), value.to_string());
	}
	if let Some(value) = options.skin.hat {
		out.insert(String::from("modelPart_hat"), value.to_string());
	}
	if let Some(value) = options.video.allow_block_alternatives {
		if after_14w28a && before_15w31a {
			out.insert(String::from("allowBlockAlternatives"), value.to_string());
		}
	}

	if let Some(resolution) = &options.video.fullscreen_resolution {
		out.insert(
			String::from("fullscreenResolution"),
			write_fullscreen_resolution(resolution),
		);
	}

	let custom_clone = options.custom.clone();
	out.extend(custom_clone);

	Ok(out)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::io::options::read::parse_options_str;

	#[test]
	fn test_create_keys() {
		let options = parse_options_str(r#"{"client": {}, "server": {}}"#).unwrap();
		dbg!(&options);
		let versions = [String::from("1.18"), String::from("1.19.3")];
		create_keys(&options.client.unwrap(), "1.19.3", &versions).unwrap();
	}
}
