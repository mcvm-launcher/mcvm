use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::Context;
use itertools::Itertools;

use crate::read::{read_options_file, EnumOrNumber};
use mcvm_shared::util::ToInt;

use mcvm_shared::versions::{VersionInfo, VersionPattern};

use super::{ClientOptions, CloudRenderMode, FullscreenResolution, GraphicsMode};

const SEP: char = ':';

/// Write options.txt to a file
pub async fn write_options_txt(
	options: HashMap<String, String>,
	path: &Path,
	data_version: &Option<i32>,
) -> anyhow::Result<()> {
	let mut options = merge_options_txt(path, options)
		.await
		.context("Failed to merge with existing options.txt")?;
	// Write the data version so that the game recognizes the options file correctly on first run
	add_data_version_field(&mut options, data_version);
	let file = File::create(path).context("Failed to open file")?;
	let mut file = BufWriter::new(file);
	for (key, value) in options.iter().sorted_by_key(|x| x.0) {
		write_key(key, value, &mut file)
			.with_context(|| format!("Failed to write line for option {key} with value {value}"))?;
	}

	Ok(())
}

/// Collect a hashmap from an existing options.txt file so we can compare with it
pub async fn read_options_txt(path: &Path) -> anyhow::Result<HashMap<String, String>> {
	if path.exists() {
		let contents = std::fs::read_to_string(path).context("Failed to read options.txt")?;
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

/// Adds the data version to the options HashMap. Does not overwrite it if it already exists
fn add_data_version_field(options: &mut HashMap<String, String>, data_version: &Option<i32>) {
	if options.contains_key("version") {
		return;
	}
	if let Some(data_version) = data_version {
		options.insert("version".into(), data_version.to_string());
	}
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
		resolution.width, resolution.height, resolution.refresh_rate, resolution.color_bits,
	)
}

/// Converts Field of View from the integer used in game and the options
/// to the format used in the options.txt. In-game the number is in degrees, but
/// it is a number from -1 to 1 in the options.txt. According to the wiki, the formula is
/// `degrees = 40 * value + 70`.
fn convert_fov(fov: u8) -> f32 {
	(fov as f32 - 70.0) / 40.0
}

/// Converts mouse sensitivity from the 0-200% integer value to 0-1
fn convert_mouse_sensitivity(sensitivity: i16) -> f32 {
	(sensitivity as f32) / 2.0 / 100.0
}

/// Write options options to a list of keys
pub fn create_keys(
	options: &ClientOptions,
	version_info: &VersionInfo,
) -> anyhow::Result<HashMap<String, String>> {
	let mut out = HashMap::new();

	// Version checks
	let after_12w50a = VersionPattern::After("12w50a".into()).matches_info(version_info);
	let after_13w36a = VersionPattern::After("13w36a".into()).matches_info(version_info);
	let after_13w47a = VersionPattern::After("13w47a".into()).matches_info(version_info);
	let after_14w25a = VersionPattern::After("14w25a".into()).matches_info(version_info);
	let after_14w28a = VersionPattern::After("14w28a".into()).matches_info(version_info);
	let after_17w06a = VersionPattern::After("17w06a".into()).matches_info(version_info);
	let after_17w47a = VersionPattern::After("17w47a".into()).matches_info(version_info);
	let after_18w15a = VersionPattern::After("18w15a".into()).matches_info(version_info);
	let after_18w21a = VersionPattern::After("18w21a".into()).matches_info(version_info);
	let after_1_13_pre2 = VersionPattern::After("1.13-pre2".into()).matches_info(version_info);
	let after_1_15_2_pre1 = VersionPattern::After("1.15.2-pre1".into()).matches_info(version_info);
	let after_1_16_4_rc1 = VersionPattern::After("1.16.4-rc1".into()).matches_info(version_info);
	let after_21w13a = VersionPattern::After("21w13a".into()).matches_info(version_info);
	let after_21w37a = VersionPattern::After("21w37a".into()).matches_info(version_info);
	let after_21w38a = VersionPattern::After("21w38a".into()).matches_info(version_info);
	let after_21w42a = VersionPattern::After("21w42a".into()).matches_info(version_info);
	let after_1_18_pre2 = VersionPattern::After("1.18-pre2".into()).matches_info(version_info);
	let after_1_18_2_pre1 = VersionPattern::After("1.18.2-pre1".into()).matches_info(version_info);
	let after_22w11a = VersionPattern::After("22w11a".into()).matches_info(version_info);
	let after_22w15a = VersionPattern::After("22w15a".into()).matches_info(version_info);

	let before_13w42a = VersionPattern::Before("13w42a".into()).matches_info(version_info);
	let before_14w03a = VersionPattern::Before("14w03a".into()).matches_info(version_info);
	let before_15w31a = VersionPattern::Before("15w31a".into()).matches_info(version_info);
	let before_1_13 = VersionPattern::Before("1.13".into()).matches_info(version_info);
	let before_20w27a = VersionPattern::Before("20w27a".into()).matches_info(version_info);
	let before_21w43a = VersionPattern::Before("21w43a".into()).matches_info(version_info);
	let before_1_19_4 = VersionPattern::Before("1.19.4".into()).matches_info(version_info);

	let is_3d_shareware =
		VersionPattern::Single("3D Shareware v1.34".into()).matches_info(version_info);

	let stream_options_enabled = after_13w47a && before_15w31a;

	if let Some(value) = options.data_version {
		out.insert("version".into(), value.to_string());
	}
	if let Some(value) = options.control.auto_jump {
		out.insert("autoJump".into(), value.to_string());
	}
	if let Some(value) = options.video.fullscreen {
		out.insert("fullscreen".into(), value.to_string());
	}
	if let Some(value) = options.chat.auto_command_suggestions {
		if after_17w47a {
			out.insert("autoSuggestions".into(), value.to_string());
		}
	}
	if let Some(value) = options.chat.enable_colors {
		out.insert("chatColors".into(), value.to_string());
	}
	if let Some(value) = options.chat.enable_links {
		out.insert("chatLinks".into(), value.to_string());
	}
	if let Some(value) = options.chat.prompt_links {
		out.insert("chatLinksPrompt".into(), value.to_string());
	}
	if let Some(value) = options.video.vsync {
		out.insert("enableVsync".into(), value.to_string());
	}
	if let Some(value) = options.video.entity_shadows {
		out.insert("entityShadows".into(), value.to_string());
	}
	if let Some(value) = options.chat.force_unicode {
		out.insert("forceUnicodeFont".into(), value.to_string());
	}
	if let Some(value) = options.control.discrete_mouse_scroll {
		out.insert("discrete_mouse_scroll".into(), value.to_string());
	}
	if let Some(value) = options.control.invert_mouse_y {
		out.insert("invertYMouse".into(), value.to_string());
	}
	if let Some(value) = options.realms_notifications {
		out.insert("realmsNotifications".into(), value.to_string());
	}
	if let Some(value) = options.reduced_debug_info {
		out.insert("reducedDebugInfo".into(), value.to_string());
	}
	if let Some(value) = options.sound.show_subtitles {
		out.insert("showSubtitles".into(), value.to_string());
	}
	if let Some(value) = options.sound.directional_audio {
		if after_22w11a {
			out.insert("directionalAudio".into(), value.to_string());
		}
	}
	if let Some(value) = options.control.enable_touchscreen {
		out.insert("touchscreen".into(), value.to_string());
	}
	if let Some(value) = options.video.view_bobbing {
		out.insert("bobView".into(), value.to_string());
	}
	if let Some(value) = options.control.toggle_crouch {
		out.insert("toggleCrouch".into(), value.to_string());
	}
	if let Some(value) = options.control.toggle_sprint {
		out.insert("toggleSprint".into(), value.to_string());
	}
	if let Some(value) = options.video.dark_mojang_background {
		if after_21w13a {
			out.insert("darkMojangStudiosBackground".into(), value.to_string());
		}
	}
	if after_21w37a {
		if let Some(value) = options.video.hide_lightning_flashes {
			out.insert("hideLightningFlashes".into(), value.to_string());
		}
		if let Some(value) = &options.video.chunk_updates_mode {
			out.insert("prioritizeChunkUpdates".into(), value.to_int().to_string());
		}
		if let Some(device) = &options.sound.device {
			out.insert("soundDevice".into(), device.clone());
		}
	}
	if let Some(value) = options.control.mouse_sensitivity {
		out.insert(
			"mouseSensitivity".into(),
			convert_mouse_sensitivity(value).to_string(),
		);
	}
	if let Some(value) = options.video.fov {
		out.insert("fov".into(), convert_fov(value).to_string());
	}
	if let Some(value) = options.video.screen_effect_scale {
		out.insert("screenEffectScale".into(), value.to_string());
	}
	if let Some(value) = options.video.fov_effect_scale {
		out.insert("fovEffectScale".into(), value.to_string());
	}
	if let Some(value) = options.video.darkness_effect_scale {
		if after_22w15a {
			out.insert("darknessEffectScale".into(), value.to_string());
		}
	}
	if let Some(value) = options.video.brightness {
		out.insert("gamma".into(), value.to_string());
	}
	if let Some(value) = options.video.render_distance {
		out.insert("renderDistance".into(), value.to_string());
	}
	if let Some(value) = options.video.simulation_distance {
		if after_21w38a {
			out.insert("simulationDistance".into(), value.to_string());
		}
	}
	if let Some(value) = options.video.entity_distance_scaling {
		out.insert("entityDistanceScaling".into(), value.to_string());
	}
	if let Some(value) = options.video.gui_scale {
		out.insert("guiScale".into(), value.to_string());
	}
	if let Some(value) = &options.video.particles {
		out.insert("particles".into(), value.to_int().to_string());
	}
	if let Some(value) = options.video.max_fps {
		out.insert("maxFps".into(), value.to_string());
	}
	if let Some(value) = &options.difficulty {
		out.insert("difficulty".into(), value.to_int().to_string());
	}
	if let Some(value) = &options.video.graphics_mode {
		if before_20w27a {
			out.insert(
				"fancyGraphics".into(),
				match value {
					EnumOrNumber::Enum(GraphicsMode::Fast) => false,
					EnumOrNumber::Enum(GraphicsMode::Fancy | GraphicsMode::Fabulous) => true,
					EnumOrNumber::Num(num) => num > &0,
				}
				.to_string(),
			);
		} else {
			out.insert("graphicsMode".into(), value.to_int().to_string());
		}
	}
	if let Some(value) = options.video.smooth_lighting {
		out.insert("ao".into(), value.to_string());
	}
	if let Some(value) = options.video.biome_blend {
		if after_18w15a {
			out.insert("biomeBlendRadius".into(), value.to_string());
		}
	}
	if let Some(value) = &options.video.clouds {
		if after_14w25a {
			out.insert("renderClouds".into(), value.to_string());
		} else {
			out.insert(
				"clouds".into(),
				matches!(value, CloudRenderMode::Fancy | CloudRenderMode::Fast).to_string(),
			);
		}
	}
	if let Some(value) = &options.resource_packs {
		out.insert("resourcePacks".into(), write_resource_packs(value));
	}
	if let Some(value) = &options.language {
		out.insert("lang".into(), value.clone());
	}
	if let Some(value) = &options.chat.visibility {
		out.insert("chatVisibility".into(), value.to_int().to_string());
	}
	if let Some(value) = options.chat.opacity {
		out.insert("chatOpacity".into(), value.to_string());
	}
	if let Some(value) = options.chat.line_spacing {
		out.insert("chatLineSpacing".into(), value.to_string());
	}
	if let Some(value) = options.chat.background_opacity {
		out.insert("textBackgroundOpacity".into(), value.to_string());
	}
	if let Some(value) = options.chat.background_for_chat_only {
		out.insert("backgroundForChatOnly".into(), value.to_string());
	}
	if let Some(value) = options.hide_server_address {
		out.insert("hideServerAddress".into(), value.to_string());
	}
	if let Some(value) = options.advanced_item_tooltips {
		out.insert("advancedItemTooltips".into(), value.to_string());
	}
	if let Some(value) = options.pause_on_lost_focus {
		out.insert("pauseOnLostFocus".into(), value.to_string());
	}
	if let Some(value) = options.video.window_width {
		out.insert("overrideWidth".into(), value.to_string());
	}
	if let Some(value) = options.video.window_height {
		out.insert("overrideHeight".into(), value.to_string());
	}
	if let Some(value) = options.held_item_tooltips {
		if after_12w50a && before_1_19_4 {
			out.insert("heldItemTooltips".into(), value.to_string());
		}
	}
	if let Some(value) = options.chat.focused_height {
		out.insert("chatHeightFocused".into(), value.to_string());
	}
	if let Some(value) = options.chat.delay {
		out.insert("chatDelay".into(), value.to_string());
	}
	if let Some(value) = options.chat.unfocused_height {
		out.insert("chatHeightUnfocused".into(), value.to_string());
	}
	if let Some(value) = options.chat.scale {
		out.insert("chatScale".into(), value.to_string());
	}
	if let Some(value) = options.chat.width {
		out.insert("chatWidth".into(), value.to_string());
	}
	if let Some(value) = options.video.mipmap_levels {
		out.insert("mipmapLevels".into(), value.to_string());
	}
	if let Some(value) = options.use_native_transport {
		out.insert("useNativeTransport".into(), value.to_string());
	}
	if let Some(value) = &options.main_hand {
		out.insert("mainHand".into(), value.to_string());
	}
	if after_17w06a {
		if let Some(value) = &options.chat.narrator_mode {
			out.insert("narrator".into(), value.to_int().to_string());
		}
		if let Some(value) = &options.tutorial_step {
			out.insert("tutorialStep".into(), value.to_string());
		}
	}
	if let Some(value) = options.control.mouse_wheel_sensitivity {
		if after_18w21a {
			out.insert("mouseWheelSensitivity".into(), value.to_string());
		}
	}
	if let Some(value) = options.control.raw_mouse_input {
		out.insert("rawMouseInput".into(), value.to_string());
	}
	if let Some(value) = &options.log_level {
		if after_1_13_pre2 {
			out.insert("glDebugVerbosity".into(), value.to_int().to_string());
		}
	}
	if let Some(value) = options.skip_multiplayer_warning {
		if after_1_15_2_pre1 {
			out.insert("skipMultiplayerWarning".into(), value.to_string());
		}
	}
	if let Some(value) = options.skip_realms_32_bit_warning {
		if after_1_18_2_pre1 {
			out.insert("skipRealms32bitWarning".into(), value.to_string());
		}
	}
	if after_1_16_4_rc1 {
		if let Some(value) = options.hide_matched_names {
			out.insert("hideMatchedNames".into(), value.to_string());
		}
		if let Some(value) = options.joined_server {
			out.insert("joinedFirstServer".into(), value.to_string());
		}
	}
	if let Some(value) = options.hide_bundle_tutorial {
		out.insert("hideBundleTutorial".into(), value.to_string());
	}
	if let Some(value) = options.sync_chunk_writes {
		out.insert("syncChunkWrites".into(), value.to_string());
	}
	if let Some(value) = options.show_autosave_indicator {
		if after_21w42a {
			out.insert("showAutosaveIndicator".into(), value.to_string());
		}
	}
	if let Some(value) = options.allow_server_listing {
		if after_1_18_pre2 {
			out.insert("allowServerListing".into(), value.to_string());
		}
	}
	if let Some(value) = options.snooper_enabled {
		if before_21w43a {
			out.insert("snooperEnabled".into(), value.to_string());
		}
	}
	if stream_options_enabled {
		if let Some(value) = options.stream.bytes_per_pixel {
			out.insert("streamBytesPerPixel".into(), value.to_string());
		}
		if let Some(value) = options.stream.chat_enabled {
			out.insert("streamChatEnabled".into(), (value as i32).to_string());
		}
		if let Some(value) = options.stream.chat_filter {
			out.insert("streamChatUserFilter".into(), (value as i32).to_string());
		}
		if let Some(value) = options.stream.compression {
			out.insert("streamCompression".into(), (value as i32).to_string());
		}
		if let Some(value) = options.stream.fps {
			out.insert("streamFps".into(), value.to_string());
		}
		if let Some(value) = options.stream.bitrate {
			out.insert("streamKbps".into(), value.to_string());
		}
		if let Some(value) = options.stream.microphone_toggle_behavior {
			out.insert("streamMicToggleBehavior".into(), (value as i32).to_string());
		}
		if let Some(value) = options.stream.microphone_volume {
			out.insert("streamMicVolume".into(), value.to_string());
		}
		if let Some(value) = &options.stream.preferred_server {
			out.insert("streamKbps".into(), value.clone());
		}
		// No idea why this one is suddenly true/false instead of 1/0 but the wiki says so
		if let Some(value) = options.stream.send_metadata {
			out.insert("streamSendMetadata".into(), value.to_string());
		}
		if let Some(value) = options.stream.system_volume {
			out.insert("streamSystemVolume".into(), value.to_string());
		}
	}

	// Keybinds
	if let Some(value) = &options.control.keys.attack {
		out.insert("key_key.attack".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.r#use {
		out.insert("key_key.use".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.forward {
		out.insert("key_key.forward".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.left {
		out.insert("key_key.left".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.back {
		out.insert("key_key.back".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.right {
		out.insert("key_key.right".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.jump {
		out.insert("key_key.jump".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.sneak {
		out.insert("key_key.sneak".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.sprint {
		out.insert("key_key.sprint".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.drop {
		out.insert("key_key.drop".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.inventory {
		out.insert("key_key.inventory".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.chat {
		out.insert("key_key.chat".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.playerlist {
		out.insert("key_key.playerlist".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.pick_item {
		out.insert("key_key.pickItem".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.command {
		out.insert("key_key.command".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.social_interactions {
		out.insert(
			"key_key.socialInteractions".into(),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.screenshot {
		out.insert("key_key.screenshot".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.toggle_perspective {
		out.insert(
			"key_key.togglePerspective".into(),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.smooth_camera {
		out.insert(
			"key_key.smoothCamera".into(),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.fullscreen {
		out.insert("key_key.fullscreen".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.spectator_outlines {
		out.insert(
			"key_key.spectatorOutlines".into(),
			value.get_keycode(before_1_13),
		);
	}
	if let Some(value) = &options.control.keys.swap_offhand {
		let keybind = if before_20w27a {
			"key_key.swapHands"
		} else {
			"key_key.swapOffhand"
		};
		out.insert(keybind.to_string(), value.get_keycode(before_1_13));
	}
	if after_17w06a {
		if let Some(value) = &options.control.keys.save_toolbar {
			out.insert(
				"key_key.saveToolbarActivator".into(),
				value.get_keycode(before_1_13),
			);
		}
		if let Some(value) = &options.control.keys.load_toolbar {
			out.insert(
				"key_key.loadToolbarActivator".into(),
				value.get_keycode(before_1_13),
			);
		}
		if let Some(value) = &options.control.keys.advancements {
			out.insert(
				"key_key.advancements".into(),
				value.get_keycode(before_1_13),
			);
		}
	}
	if let Some(value) = &options.control.keys.hotbar_1 {
		out.insert("key_key.hotbar.1".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.hotbar_2 {
		out.insert("key_key.hotbar.2".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.hotbar_3 {
		out.insert("key_key.hotbar.3".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.hotbar_4 {
		out.insert("key_key.hotbar.4".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.hotbar_5 {
		out.insert("key_key.hotbar.5".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.hotbar_6 {
		out.insert("key_key.hotbar.6".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.hotbar_7 {
		out.insert("key_key.hotbar.7".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.hotbar_8 {
		out.insert("key_key.hotbar.8".into(), value.get_keycode(before_1_13));
	}
	if let Some(value) = &options.control.keys.hotbar_9 {
		out.insert("key_key.hotbar.9".into(), value.get_keycode(before_1_13));
	}
	if is_3d_shareware {
		if let Some(value) = &options.control.keys.boss_mode {
			out.insert("key_key.boss_mode".into(), value.get_keycode(before_1_13));
		}
		if let Some(value) = &options.control.keys.decrease_view {
			out.insert(
				"key_key.decrease_view".into(),
				value.get_keycode(before_1_13),
			);
		}
		if let Some(value) = &options.control.keys.increase_view {
			out.insert(
				"key_key.increase_view".into(),
				value.get_keycode(before_1_13),
			);
		}
	}
	if stream_options_enabled {
		if let Some(value) = &options.control.keys.stream_commercial {
			out.insert(
				"key_key.streamCommercial".into(),
				value.get_keycode(before_1_13),
			);
		}
		if let Some(value) = &options.control.keys.stream_pause_unpause {
			out.insert(
				"key_key.streamPauseUnpause".into(),
				value.get_keycode(before_1_13),
			);
		}
		if let Some(value) = &options.control.keys.stream_start_stop {
			out.insert(
				"key_key.streamStartStop".into(),
				value.get_keycode(before_1_13),
			);
		}
		// FIXME: Duplicated key
		if let Some(value) = &options.control.keys.stream_commercial {
			out.insert(
				"key_key.streamCommercial".into(),
				value.get_keycode(before_1_13),
			);
		}
		if let Some(value) = &options.control.keys.stream_toggle_microphone {
			out.insert(
				"key_key.streamToggleMic".into(),
				value.get_keycode(before_1_13),
			);
		}
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
			out.insert("soundCategory_master".into(), value.to_string());
		}
		if let Some(value) = options.sound.volume.music {
			out.insert("soundCategory_music".into(), value.to_string());
		}
		if let Some(value) = options.sound.volume.record {
			out.insert(records_key.to_string(), value.to_string());
		}
		if let Some(value) = options.sound.volume.weather {
			out.insert("soundCategory_weather".into(), value.to_string());
		}
		if let Some(value) = options.sound.volume.block {
			out.insert(blocks_key.to_string(), value.to_string());
		}
		if let Some(value) = options.sound.volume.hostile {
			out.insert(mobs_key.to_string(), value.to_string());
		}
		if let Some(value) = options.sound.volume.neutral {
			out.insert(animals_key.to_string(), value.to_string());
		}
		if let Some(value) = options.sound.volume.player {
			out.insert(players_key.to_string(), value.to_string());
		}
		if let Some(value) = options.sound.volume.ambient {
			out.insert("soundCategory_ambient".into(), value.to_string());
		}
		if let Some(value) = options.sound.volume.voice {
			out.insert("soundCategory_voice".into(), value.to_string());
		}
	} else if let Some(value) = options.sound.volume.master {
		let volume_up = value > 0.0;
		out.insert("sound".into(), volume_up.to_string());
	}
	// Model parts
	if let Some(value) = options.skin.cape {
		let key = if before_14w03a {
			"showCape"
		} else {
			"modelPart_cape"
		};
		out.insert(key.to_string(), value.to_string());
	}
	if let Some(value) = options.skin.jacket {
		out.insert("modelPart_jacket".into(), value.to_string());
	}
	if let Some(value) = options.skin.left_sleeve {
		out.insert("modelPart_left_sleeve".into(), value.to_string());
	}
	if let Some(value) = options.skin.right_sleeve {
		out.insert("modelPart_right_sleeve".into(), value.to_string());
	}
	if let Some(value) = options.skin.left_pants {
		out.insert("modelPart_left_pants_leg".into(), value.to_string());
	}
	if let Some(value) = options.skin.right_pants {
		out.insert("modelPart_right_pants_leg".into(), value.to_string());
	}
	if let Some(value) = options.skin.hat {
		out.insert("modelPart_hat".into(), value.to_string());
	}
	if let Some(value) = options.video.allow_block_alternatives {
		if after_14w28a && before_15w31a {
			out.insert("allowBlockAlternatives".into(), value.to_string());
		}
	}

	if let Some(resolution) = &options.video.fullscreen_resolution {
		out.insert(
			"fullscreenResolution".into(),
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
	use crate::read::parse_options_str;

	#[test]
	fn test_create_keys() {
		let options = parse_options_str(r#"{"client": {}, "server": {}}"#).unwrap();
		dbg!(&options);
		let versions = vec!["1.18".to_string(), "1.19.3".to_string()];
		let info = VersionInfo {
			version: "1.19.3".to_string(),
			versions,
		};
		create_keys(&options.client.unwrap(), &info).unwrap();
	}
}
