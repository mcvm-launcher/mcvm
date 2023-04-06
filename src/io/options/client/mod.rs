mod file;
mod keybinds;

pub use file::create_keys;
pub use file::write_options_txt;

use std::{collections::HashMap, fmt::Display};

use serde::Deserialize;

use crate::util::ToInt;

use self::keybinds::Keybind;

use super::read::EnumOrNumber;

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct KeyOptions {
	pub attack: Option<Keybind>,
	pub r#use: Option<Keybind>,
	pub forward: Option<Keybind>,
	pub left: Option<Keybind>,
	pub back: Option<Keybind>,
	pub right: Option<Keybind>,
	pub jump: Option<Keybind>,
	pub sneak: Option<Keybind>,
	pub sprint: Option<Keybind>,
	pub drop: Option<Keybind>,
	pub inventory: Option<Keybind>,
	pub chat: Option<Keybind>,
	pub playerlist: Option<Keybind>,
	pub pick_item: Option<Keybind>,
	pub command: Option<Keybind>,
	pub social_interactions: Option<Keybind>,
	pub screenshot: Option<Keybind>,
	pub toggle_perspective: Option<Keybind>,
	pub smooth_camera: Option<Keybind>,
	pub fullscreen: Option<Keybind>,
	pub spectator_outlines: Option<Keybind>,
	pub swap_offhand: Option<Keybind>,
	pub save_toolbar: Option<Keybind>,
	pub load_toolbar: Option<Keybind>,
	pub advancements: Option<Keybind>,
	pub hotbar_1: Option<Keybind>,
	pub hotbar_2: Option<Keybind>,
	pub hotbar_3: Option<Keybind>,
	pub hotbar_4: Option<Keybind>,
	pub hotbar_5: Option<Keybind>,
	pub hotbar_6: Option<Keybind>,
	pub hotbar_7: Option<Keybind>,
	pub hotbar_8: Option<Keybind>,
	pub hotbar_9: Option<Keybind>,
}

impl Default for KeyOptions {
	fn default() -> Self {
		Self {
			attack: None,
			r#use: None,
			forward: None,
			left: None,
			back: None,
			right: None,
			jump: None,
			sneak: None,
			sprint: None,
			drop: None,
			inventory: None,
			chat: None,
			playerlist: None,
			pick_item: None,
			command: None,
			social_interactions: None,
			screenshot: None,
			toggle_perspective: None,
			smooth_camera: None,
			fullscreen: None,
			spectator_outlines: None,
			swap_offhand: None,
			save_toolbar: None,
			load_toolbar: None,
			advancements: None,
			hotbar_1: None,
			hotbar_2: None,
			hotbar_3: None,
			hotbar_4: None,
			hotbar_5: None,
			hotbar_6: None,
			hotbar_7: None,
			hotbar_8: None,
			hotbar_9: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ControlOptions {
	pub keys: KeyOptions,
	pub auto_jump: Option<bool>,
	pub discrete_mouse_scroll: Option<bool>,
	pub invert_mouse_y: Option<bool>,
	pub enable_touchscreen: Option<bool>,
	pub toggle_sprint: Option<bool>,
	pub toggle_crouch: Option<bool>,
	pub mouse_sensitivity: Option<f32>,
	pub mouse_wheel_sensitivity: Option<f32>,
	pub raw_mouse_input: Option<bool>,
}

impl Default for ControlOptions {
	fn default() -> Self {
		Self {
			keys: KeyOptions::default(),
			auto_jump: None,
			discrete_mouse_scroll: None,
			invert_mouse_y: None,
			enable_touchscreen: None,
			toggle_sprint: None,
			toggle_crouch: None,
			mouse_sensitivity: None,
			mouse_wheel_sensitivity: None,
			raw_mouse_input: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ChatOptions {
	pub auto_command_suggestions: Option<bool>,
	pub enable_colors: Option<bool>,
	pub enable_links: Option<bool>,
	pub prompt_links: Option<bool>,
	pub force_unicode: Option<bool>,
	pub visibility: Option<EnumOrNumber<ChatVisibility>>,
	pub opacity: Option<f32>,
	pub line_spacing: Option<f32>,
	pub background_opacity: Option<f32>,
	pub background_for_chat_only: Option<bool>,
	pub focused_height: Option<f32>,
	pub unfocused_height: Option<f32>,
	pub delay: Option<f32>,
	pub scale: Option<f32>,
	pub width: Option<f32>,
	pub narrator_mode: Option<EnumOrNumber<NarratorMode>>,
}

impl Default for ChatOptions {
	fn default() -> Self {
		Self {
			auto_command_suggestions: None,
			enable_colors: None,
			enable_links: None,
			prompt_links: None,
			force_unicode: None,
			visibility: None,
			opacity: None,
			line_spacing: None,
			background_opacity: None,
			background_for_chat_only: None,
			focused_height: None,
			unfocused_height: None,
			delay: None,
			scale: None,
			width: None,
			narrator_mode: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct VideoOptions {
	pub vsync: Option<bool>,
	pub entity_shadows: Option<bool>,
	pub fullscreen: Option<bool>,
	pub view_bobbing: Option<bool>,
	pub dark_mojang_background: Option<bool>,
	pub hide_lightning_flashes: Option<bool>,
	pub fov: Option<u8>,
	pub screen_effect_scale: Option<f32>,
	pub fov_effect_scale: Option<f32>,
	pub darkness_effect_scale: Option<f32>,
	pub brightness: Option<f32>,
	pub render_distance: Option<u8>,
	pub simulation_distance: Option<u8>,
	pub entity_distance_scaling: Option<f32>,
	pub gui_scale: Option<u8>,
	pub particles: Option<EnumOrNumber<ParticlesMode>>,
	pub max_fps: Option<u8>,
	pub graphics_mode: Option<EnumOrNumber<GraphicsMode>>,
	pub smooth_lighting: Option<bool>,
	pub chunk_updates_mode: Option<EnumOrNumber<ChunkUpdatesMode>>,
	pub biome_blend: Option<u8>,
	pub clouds: Option<CloudRenderMode>,
	pub mipmap_levels: Option<u8>,
	pub window_width: Option<u16>,
	pub window_height: Option<u16>,
	pub attack_indicator: Option<EnumOrNumber<AttackIndicatorMode>>,
	pub fullscreen_resolution: Option<FullscreenResolution>,
	pub allow_block_alternatives: Option<bool>,
}

impl Default for VideoOptions {
	fn default() -> Self {
		Self {
			vsync: None,
			entity_shadows: None,
			fullscreen: None,
			view_bobbing: None,
			dark_mojang_background: None,
			hide_lightning_flashes: None,
			fov: None,
			screen_effect_scale: None,
			fov_effect_scale: None,
			darkness_effect_scale: None,
			brightness: None,
			render_distance: None,
			simulation_distance: None,
			entity_distance_scaling: None,
			gui_scale: None,
			particles: None,
			max_fps: None,
			graphics_mode: None,
			smooth_lighting: None,
			chunk_updates_mode: None,
			biome_blend: None,
			clouds: None,
			mipmap_levels: None,
			window_width: None,
			window_height: None,
			attack_indicator: None,
			fullscreen_resolution: None,
			allow_block_alternatives: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct VolumeOptions {
	pub master: Option<f32>,
	pub music: Option<f32>,
	pub record: Option<f32>,
	pub weather: Option<f32>,
	pub block: Option<f32>,
	pub hostile: Option<f32>,
	pub neutral: Option<f32>,
	pub player: Option<f32>,
	pub ambient: Option<f32>,
	pub voice: Option<f32>,
}

impl Default for VolumeOptions {
	fn default() -> Self {
		Self {
			master: None,
			music: None,
			record: None,
			weather: None,
			block: None,
			hostile: None,
			neutral: None,
			player: None,
			ambient: None,
			voice: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct SoundOptions {
	pub volume: VolumeOptions,
	pub show_subtitles: Option<bool>,
	pub directional_audio: Option<bool>,
	pub device: Option<String>,
}

impl Default for SoundOptions {
	fn default() -> Self {
		Self {
			volume: VolumeOptions::default(),
			show_subtitles: None,
			directional_audio: None,
			device: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct SkinOptions {
	pub cape: Option<bool>,
	pub jacket: Option<bool>,
	pub left_sleeve: Option<bool>,
	pub right_sleeve: Option<bool>,
	pub left_pants: Option<bool>,
	pub right_pants: Option<bool>,
	pub hat: Option<bool>,
}

impl Default for SkinOptions {
	fn default() -> Self {
		Self {
			cape: None,
			jacket: None,
			left_sleeve: None,
			right_sleeve: None,
			left_pants: None,
			right_pants: None,
			hat: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ClientOptions {
	pub data_version: Option<i16>,
	pub video: VideoOptions,
	pub control: ControlOptions,
	pub chat: ChatOptions,
	pub sound: SoundOptions,
	pub skin: SkinOptions,
	pub custom: HashMap<String, String>,
	pub realms_notifications: Option<bool>,
	pub reduced_debug_info: Option<bool>,
	pub difficulty: Option<EnumOrNumber<Difficulty>>,
	pub resource_packs: Option<Vec<String>>,
	pub language: Option<String>,
	pub tutorial_step: Option<TutorialStep>,
	pub skip_multiplayer_warning: Option<bool>,
	pub skip_realms_32_bit_warning: Option<bool>,
	pub hide_bundle_tutorial: Option<bool>,
	pub joined_server: Option<bool>,
	pub sync_chunk_writes: Option<bool>,
	pub use_native_transport: Option<bool>,
	pub held_item_tooltips: Option<bool>,
	pub advanced_item_tooltips: Option<bool>,
	pub log_level: Option<EnumOrNumber<LogLevel>>,
	pub hide_matched_names: Option<bool>,
	pub pause_on_lost_focus: Option<bool>,
	pub main_hand: Option<MainHand>,
	pub hide_server_address: Option<bool>,
	pub show_autosave_indicator: Option<bool>,
	pub allow_server_listing: Option<bool>,
}

impl Default for ClientOptions {
	fn default() -> Self {
		Self {
			data_version: None,
			video: VideoOptions::default(),
			control: ControlOptions::default(),
			chat: ChatOptions::default(),
			sound: SoundOptions::default(),
			skin: SkinOptions::default(),
			custom: HashMap::default(),
			realms_notifications: None,
			reduced_debug_info: None,
			difficulty: None,
			resource_packs: None,
			language: None,
			tutorial_step: None,
			skip_multiplayer_warning: None,
			skip_realms_32_bit_warning: None,
			hide_bundle_tutorial: None,
			joined_server: None,
			sync_chunk_writes: None,
			use_native_transport: None,
			held_item_tooltips: None,
			advanced_item_tooltips: None,
			log_level: None,
			hide_matched_names: None,
			pause_on_lost_focus: None,
			main_hand: None,
			hide_server_address: None,
			show_autosave_indicator: None,
			allow_server_listing: None,
		}
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GraphicsMode {
	Fast,
	Fancy,
	Fabulous,
}

impl ToInt for GraphicsMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ParticlesMode {
	All,
	Decreased,
	Minimal,
}

impl ToInt for ParticlesMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
	Peaceful,
	Easy,
	Normal,
	Hard,
}

impl ToInt for Difficulty {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ChunkUpdatesMode {
	Threaded,
	SemiBlocking,
	FullyBlocking,
}

impl ToInt for ChunkUpdatesMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CloudRenderMode {
	Fancy,
	Off,
	Fast,
}

impl Display for CloudRenderMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Fancy => "true",
				Self::Off => "false",
				Self::Fast => "fast",
			}
		)
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ChatVisibility {
	Shown,
	CommandsOnly,
	Hidden,
}

impl ToInt for ChatVisibility {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MainHand {
	Left,
	Right,
}

impl Display for MainHand {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Left => "left",
				Self::Right => "right",
			}
		)
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AttackIndicatorMode {
	Off,
	Crosshair,
	Hotbar,
}

impl ToInt for AttackIndicatorMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum NarratorMode {
	Off,
	All,
	Chat,
	System,
}

impl ToInt for NarratorMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TutorialStep {
	Movement,
	FindTree,
	PunchTree,
	OpenInventory,
	CraftPlanks,
	None,
}

impl Display for TutorialStep {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Movement => "movement",
				Self::FindTree => "find_tree",
				Self::PunchTree => "punch_tree",
				Self::OpenInventory => "open_inventory",
				Self::CraftPlanks => "craft_planks",
				Self::None => "none",
			}
		)
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
	None,
	High,
	Medium,
	Low,
	Notification,
}

impl ToInt for LogLevel {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

// TODO: Add sensible defaults for resolution options
#[derive(Deserialize, Debug, Clone)]
pub struct FullscreenResolution {
	pub width: u32,
	pub height: u32,
	pub refresh_rate: u32,
	pub color_bits: u32,
}
