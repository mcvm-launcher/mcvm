/// Writing to the options.txt file
mod file;
/// Dealing with configured keybinds
mod keybinds;

pub use file::create_keys;
pub use file::write_options_txt;

use std::{collections::HashMap, fmt::Display};

use mcvm_shared::util::{DefaultExt, ToInt};
use serde::Deserialize;
use serde::Serialize;

use self::keybinds::Keybind;
use super::read::EnumOrNumber;

// I do not want to document all of these
pub use deser::*;
#[allow(missing_docs)]
pub mod deser {
	#[cfg(feature = "schema")]
	use schemars::JsonSchema;

	use super::*;

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct ClientOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub data_version: Option<i16>,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub video: VideoOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub control: ControlOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub chat: ChatOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub sound: SoundOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub skin: SkinOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub stream: StreamOptions,
		#[serde(skip_serializing_if = "HashMap::is_empty")]
		pub custom: HashMap<String, String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub realms_notifications: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub reduced_debug_info: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub difficulty: Option<EnumOrNumber<Difficulty>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub resource_packs: Option<Vec<String>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub language: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub tutorial_step: Option<TutorialStep>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub skip_multiplayer_warning: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub skip_realms_32_bit_warning: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hide_bundle_tutorial: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub joined_server: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub sync_chunk_writes: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub use_native_transport: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub held_item_tooltips: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub advanced_item_tooltips: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub log_level: Option<EnumOrNumber<LogLevel>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hide_matched_names: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub pause_on_lost_focus: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub main_hand: Option<MainHand>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hide_server_address: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub show_autosave_indicator: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub allow_server_listing: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub snooper_enabled: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct ControlOptions {
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub keys: KeyOptions,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub auto_jump: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub discrete_mouse_scroll: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub invert_mouse_y: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable_touchscreen: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub toggle_sprint: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub toggle_crouch: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub mouse_sensitivity: Option<i16>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub mouse_wheel_sensitivity: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub raw_mouse_input: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct KeyOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub attack: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub r#use: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub forward: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub left: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub back: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub right: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub jump: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub sneak: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub sprint: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub drop: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub inventory: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub chat: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub playerlist: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub pick_item: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub command: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub social_interactions: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub screenshot: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub toggle_perspective: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub smooth_camera: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub fullscreen: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub spectator_outlines: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub swap_offhand: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub save_toolbar: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub load_toolbar: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub advancements: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hotbar_1: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hotbar_2: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hotbar_3: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hotbar_4: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hotbar_5: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hotbar_6: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hotbar_7: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hotbar_8: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hotbar_9: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub boss_mode: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub decrease_view: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub increase_view: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub stream_commercial: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub stream_pause_unpause: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub stream_start_stop: Option<Keybind>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub stream_toggle_microphone: Option<Keybind>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct ChatOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub auto_command_suggestions: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable_colors: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable_links: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub prompt_links: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub force_unicode: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub visibility: Option<EnumOrNumber<ChatVisibility>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub opacity: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub line_spacing: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub background_opacity: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub background_for_chat_only: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub focused_height: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub unfocused_height: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub delay: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub scale: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub width: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub narrator_mode: Option<EnumOrNumber<NarratorMode>>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct VideoOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub vsync: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub entity_shadows: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub fullscreen: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub view_bobbing: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub dark_mojang_background: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hide_lightning_flashes: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub fov: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub screen_effect_scale: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub fov_effect_scale: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub darkness_effect_scale: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub brightness: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub render_distance: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub simulation_distance: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub entity_distance_scaling: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub gui_scale: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub particles: Option<EnumOrNumber<ParticlesMode>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub max_fps: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub graphics_mode: Option<EnumOrNumber<GraphicsMode>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub smooth_lighting: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub chunk_updates_mode: Option<EnumOrNumber<ChunkUpdatesMode>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub biome_blend: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub clouds: Option<CloudRenderMode>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub mipmap_levels: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub window_width: Option<u16>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub window_height: Option<u16>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub attack_indicator: Option<EnumOrNumber<AttackIndicatorMode>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub fullscreen_resolution: Option<FullscreenResolution>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub allow_block_alternatives: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct VolumeOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub master: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub music: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub record: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub weather: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub block: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hostile: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub neutral: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub player: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub ambient: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub voice: Option<f32>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct SoundOptions {
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub volume: VolumeOptions,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub show_subtitles: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub directional_audio: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub device: Option<String>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct SkinOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub cape: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub jacket: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub left_sleeve: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub right_sleeve: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub left_pants: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub right_pants: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hat: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct StreamOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub bytes_per_pixel: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub chat_enabled: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub chat_filter: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub compression: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub fps: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub bitrate: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub microphone_toggle_behavior: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub microphone_volume: Option<f32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub preferred_server: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub send_metadata: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub system_volume: Option<f32>,
	}

	#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, Clone, Debug)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, Debug, Clone)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

	#[derive(Deserialize, Serialize, Clone, Debug)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
	#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	pub struct FullscreenResolution {
		pub width: u32,
		pub height: u32,
		pub refresh_rate: u32,
		pub color_bits: u32,
	}
}
