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
pub fn write_options_txt(
	options: HashMap<String, String>,
	path: &Path,
	data_version: &Option<i32>,
) -> anyhow::Result<()> {
	let mut options =
		merge_options_txt(path, options).context("Failed to merge with existing options.txt")?;
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
pub fn read_options_txt(path: &Path) -> anyhow::Result<HashMap<String, String>> {
	if path.exists() {
		let contents = std::fs::read_to_string(path).context("Failed to read options.txt")?;
		read_options_file(&contents, SEP)
	} else {
		Ok(HashMap::new())
	}
}

/// Merge keys with an existing file
pub fn merge_options_txt(
	path: &Path,
	keys: HashMap<String, String>,
) -> anyhow::Result<HashMap<String, String>> {
	let mut file_keys =
		read_options_txt(path).context("Failed to open options file for merging")?;
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

macro_rules! match_key {
	($out:ident, $option:expr, $key:literal) => {
		if let Some(value) = $option {
			$out.insert($key.into(), value.to_string());
		}
	};

	($out:ident, $option:expr, $key:literal, $version:expr) => {
		if $version {
			match_key!($out, $option, $key)
		}
	};
}

macro_rules! match_key_int {
	($out:ident, $option:expr, $key:literal) => {
		if let Some(value) = $option {
			$out.insert($key.into(), value.to_int().to_string());
		}
	};

	($out:ident, $option:expr, $key:literal, $version:expr) => {
		if $version {
			match_key_int!($out, $option, $key)
		}
	};
}

macro_rules! match_keybind {
	($out:ident, $option:expr, $key:literal, $before_1_13:expr) => {
		if let Some(value) = $option {
			$out.insert($key.into(), value.get_keycode($before_1_13));
		}
	};

	($out:ident, $option:expr, $key:literal, $before_1_13:expr, $version:expr) => {
		if $version {
			match_keybind!($out, $option, $key, $before_1_13)
		}
	};
}

/// Write options options to a list of keys
#[rustfmt::skip]
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

	match_key!(out, options.data_version, "version");
	match_key!(out, options.control.auto_jump, "autoJump");
	match_key!(out, options.video.fullscreen, "fullscreen");
	match_key!(out, options.chat.auto_command_suggestions, "autoSuggestions", after_17w47a);
	match_key!(out, options.chat.enable_colors, "chatColors");
	match_key!(out, options.chat.enable_links, "chatLinks");
	match_key!(out, options.chat.prompt_links, "chatLinksPrompt");
	match_key!(out, options.video.vsync, "enableVsync");
	match_key!(out, options.video.entity_shadows, "entityShadows");
	match_key!(out, options.chat.force_unicode, "forceUnicodeFont");
	match_key!(out, options.control.discrete_mouse_scroll, "discrete_mouse_scroll");
	match_key!(out, options.control.invert_mouse_y, "invertYMouse");
	match_key!(out, options.realms_notifications, "realmsNotifications");
	match_key!(out, options.reduced_debug_info, "reducedDebugInfo");
	match_key!(out, options.sound.show_subtitles, "showSubtitles");
	match_key!(out, options.sound.directional_audio, "directionalAudio", after_22w11a);
	match_key!(out, options.control.enable_touchscreen, "touchscreen");
	match_key!(out, options.video.view_bobbing, "bobView");
	match_key!(out, options.control.toggle_crouch, "toggleCrouch");
	match_key!(out, options.control.toggle_sprint, "toggleSprint");
	match_key!(out, options.video.dark_mojang_background, "darkMojangStudiosBackground", after_21w13a);	
	if after_21w37a {
		match_key!(out, options.video.hide_lightning_flashes, "hideLightningFlashes");
		match_key!(out, &options.sound.device, "soundDevice");
		match_key_int!(out, &options.video.chunk_updates_mode, "prioritizeChunkUpdates");
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
	match_key!(out, options.video.screen_effect_scale, "screenEffectScale");
	match_key!(out, options.video.fov_effect_scale, "fovEffectScale");
	match_key!(out, options.video.darkness_effect_scale, "darknessEffectScale", after_22w15a);
	match_key!(out, options.video.brightness, "gamma");
	match_key!(out, options.video.render_distance, "renderDistance");
	match_key!(out, options.video.simulation_distance, "simulationDistance", after_21w38a);
	match_key!(out, options.video.entity_distance_scaling, "entityDistanceScaling");
	match_key!(out, options.video.gui_scale, "guiScale");
	if let Some(value) = &options.video.particles {
		out.insert("particles".into(), value.to_int().to_string());
	}
	match_key!(out, options.video.max_fps, "maxFps");
	match_key_int!(out, &options.difficulty, "difficulty");
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
	match_key!(out, options.video.smooth_lighting, "ao");
	match_key!(out, options.video.biome_blend, "biomeBlendRadius", after_18w15a);
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
	match_key!(out, &options.language, "lang");
	match_key_int!(out, &options.chat.visibility, "chatVisibility");
	match_key!(out, options.chat.opacity, "chatOpacity");
	match_key!(out, options.chat.line_spacing, "chatLineSpacing");
	match_key!(out, options.chat.background_opacity, "textBackgroundOpacity");
	match_key!(out, options.chat.background_for_chat_only, "backgroundForChatOnly");
	match_key!(out, options.hide_server_address, "hideServerAddress");
	match_key!(out, options.advanced_item_tooltips, "advancedItemTooltips");
	match_key!(out, options.pause_on_lost_focus, "pauseOnLostFocus");
	match_key!(out, options.video.window_width, "overrideWidth");
	match_key!(out, options.video.window_height, "overrideHeight");
	match_key!(out, options.held_item_tooltips, "heldItemTooltips", after_12w50a && before_1_19_4);
	match_key!(out, options.chat.focused_height, "chatHeightFocused");
	match_key!(out, options.chat.delay, "chatDelay");
	match_key!(out, options.chat.unfocused_height, "chatHeightUnfocused");
	match_key!(out, options.chat.scale, "chatScale");
	match_key!(out, options.chat.width, "chatWidth");
	match_key!(out, options.video.mipmap_levels, "mipmapLevels");
	match_key!(out, options.use_native_transport, "useNativeTransport");
	match_key!(out, &options.main_hand, "mainHand");
	if after_17w06a {
		match_key_int!(out, &options.chat.narrator_mode, "narrator");
		match_key!(out, &options.tutorial_step, "tutorialStep");
	}
	match_key!(out, options.control.mouse_wheel_sensitivity, "mouseWheelSensitivity", after_18w21a);
	match_key!(out, options.control.raw_mouse_input, "rawMouseInput");
	match_key_int!(out, &options.log_level, "glDebugVerbosity", after_1_13_pre2);
	match_key!(out, options.skip_multiplayer_warning, "skipMultiplayerWarning", after_1_15_2_pre1);
	match_key!(out, options.skip_realms_32_bit_warning, "skipRealms32bitWarning", after_1_18_2_pre1);
	match_key!(out, options.hide_matched_names, "hideMatchedNames", after_1_16_4_rc1);
	match_key!(out, options.joined_server, "joinedFirstServer", after_1_16_4_rc1);
	match_key!(out, options.hide_bundle_tutorial, "hideBundleTutorial");
	match_key!(out, options.sync_chunk_writes, "syncChunkWrites");
	match_key!(out, options.show_autosave_indicator, "showAutosaveIndicator", after_21w42a);
	match_key!(out, options.allow_server_listing, "allowServerListing", after_1_18_pre2);
	match_key!(out, options.snooper_enabled, "snooperEnabled", before_21w43a);

	if stream_options_enabled {
		match_key!(out, options.stream.bytes_per_pixel, "streamBytesPerPixel");
		match_key_int!(out, options.stream.chat_enabled, "streamChatEnabled");
		match_key_int!(out, options.stream.chat_filter, "streamChatUserFilter");
		match_key_int!(out, options.stream.compression, "streamCompression");
		match_key!(out, options.stream.bytes_per_pixel, "streamBytesPerPixel");
		match_key!(out, options.stream.fps, "streamFps");
		match_key!(out, options.stream.bitrate, "streamKbps");
		match_key_int!(out, options.stream.microphone_toggle_behavior, "streamMicToggleBehavior");
		match_key!(out, options.stream.microphone_volume, "streamMicVolume");
		match_key!(out, &options.stream.preferred_server, "streamPreferredServer");
		match_key!(out, options.stream.send_metadata, "streamSendMetadata");
		match_key!(out, options.stream.system_volume, "streamSystemVolume");
	}

	// Keybinds
	match_keybind!(out, &options.control.keys.attack, "key_key.attack", before_1_13);
	match_keybind!(out, &options.control.keys.r#use, "key_key.use", before_1_13);
	match_keybind!(out, &options.control.keys.forward, "key_key.forward", before_1_13);
	match_keybind!(out, &options.control.keys.back, "key_key.back", before_1_13);
	match_keybind!(out, &options.control.keys.left, "key_key.left", before_1_13);
	match_keybind!(out, &options.control.keys.right, "key_key.right", before_1_13);
	match_keybind!(out, &options.control.keys.jump, "key_key.jump", before_1_13);
	match_keybind!(out, &options.control.keys.sneak, "key_key.sneak", before_1_13);
	match_keybind!(out, &options.control.keys.sprint, "key_key.sprint", before_1_13);
	match_keybind!(out, &options.control.keys.drop, "key_key.drop", before_1_13);
	match_keybind!(out, &options.control.keys.inventory, "key_key.inventory", before_1_13);
	match_keybind!(out, &options.control.keys.chat, "key_key.chat", before_1_13);
	match_keybind!(out, &options.control.keys.playerlist, "key_key.playerlist", before_1_13);
	match_keybind!(out, &options.control.keys.pick_item, "key_key.pickItem", before_1_13);
	match_keybind!(out, &options.control.keys.command, "key_key.command", before_1_13);
	match_keybind!(out, &options.control.keys.social_interactions, "key_key.socialInteractions", before_1_13);
	match_keybind!(out, &options.control.keys.screenshot, "key_key.screenshot", before_1_13);
	match_keybind!(out, &options.control.keys.toggle_perspective, "key_key.togglePerspective", before_1_13);
	match_keybind!(out, &options.control.keys.smooth_camera, "key_key.smoothCamera", before_1_13);
	match_keybind!(out, &options.control.keys.fullscreen, "key_key.fullscreen", before_1_13);
	match_keybind!(out, &options.control.keys.spectator_outlines, "key_key.spectatorOutlines", before_1_13);
	match_keybind!(out, &options.control.keys.swap_offhand, "key_key.swapHands", before_1_13, before_20w27a);
	match_keybind!(out, &options.control.keys.swap_offhand, "key_key.swapOffhand", before_1_13, !before_20w27a);
	match_keybind!(out, &options.control.keys.save_toolbar, "key_key.saveToolbarActivator", before_1_13, after_17w06a);
	match_keybind!(out, &options.control.keys.load_toolbar, "key_key.loadToolbarActivator", before_1_13, after_17w06a);
	match_keybind!(out, &options.control.keys.advancements, "key_key.advancements", before_1_13, after_17w06a);
	match_keybind!(out, &options.control.keys.hotbar_1, "key_key.hotbar.1", before_1_13);
	match_keybind!(out, &options.control.keys.hotbar_2, "key_key.hotbar.2", before_1_13);
	match_keybind!(out, &options.control.keys.hotbar_3, "key_key.hotbar.3", before_1_13);
	match_keybind!(out, &options.control.keys.hotbar_4, "key_key.hotbar.4", before_1_13);
	match_keybind!(out, &options.control.keys.hotbar_5, "key_key.hotbar.5", before_1_13);
	match_keybind!(out, &options.control.keys.hotbar_6, "key_key.hotbar.6", before_1_13);
	match_keybind!(out, &options.control.keys.hotbar_7, "key_key.hotbar.7", before_1_13);
	match_keybind!(out, &options.control.keys.hotbar_8, "key_key.hotbar.8", before_1_13);
	match_keybind!(out, &options.control.keys.hotbar_9, "key_key.hotbar.9", before_1_13);
	match_keybind!(out, &options.control.keys.boss_mode, "key_key.boss_mode", before_1_13, is_3d_shareware);
	match_keybind!(out, &options.control.keys.decrease_view, "key_key.decrease_view", before_1_13, is_3d_shareware);
	match_keybind!(out, &options.control.keys.increase_view, "key_key.increase_view", before_1_13, is_3d_shareware);
	match_keybind!(out, &options.control.keys.stream_commercial, "key_key.streamCommercial", before_1_13, stream_options_enabled);
	match_keybind!(out, &options.control.keys.stream_pause_unpause, "key_key.streamPauseUnpause", before_1_13, stream_options_enabled);
	match_keybind!(out, &options.control.keys.stream_start_stop, "key_key.streamStartStop", before_1_13, stream_options_enabled);
	match_keybind!(out, &options.control.keys.stream_toggle_microphone, "key_key.streamToggleMic", before_1_13, stream_options_enabled);

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
