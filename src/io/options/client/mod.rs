/// Writing to the options.txt file
mod file;
/// Dealing with configured keybinds
mod keybinds;

pub use file::create_keys;
pub use file::write_options_txt;
use serde::Serialize;

use std::{collections::HashMap, fmt::Display};

use serde::Deserialize;

use crate::util::ToInt;

use self::keybinds::Keybind;

use super::read::EnumOrNumber;

// I do not want to document all of these
pub use deser::*;
#[allow(missing_docs)]
pub mod deser {
	use super::*;
	
	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
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
		pub boss_mode: Option<Keybind>,
		pub decrease_view: Option<Keybind>,
		pub increase_view: Option<Keybind>,
		pub stream_commercial: Option<Keybind>,
		pub stream_pause_unpause: Option<Keybind>,
		pub stream_start_stop: Option<Keybind>,
		pub stream_toggle_microphone: Option<Keybind>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
	#[serde(default)]
	pub struct ControlOptions {
		pub keys: KeyOptions,
		pub auto_jump: Option<bool>,
		pub discrete_mouse_scroll: Option<bool>,
		pub invert_mouse_y: Option<bool>,
		pub enable_touchscreen: Option<bool>,
		pub toggle_sprint: Option<bool>,
		pub toggle_crouch: Option<bool>,
		pub mouse_sensitivity: Option<i16>,
		pub mouse_wheel_sensitivity: Option<f32>,
		pub raw_mouse_input: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
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

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
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

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
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

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
	#[serde(default)]
	pub struct SoundOptions {
		pub volume: VolumeOptions,
		pub show_subtitles: Option<bool>,
		pub directional_audio: Option<bool>,
		pub device: Option<String>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
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

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
	#[serde(default)]
	pub struct StreamOptions {
		pub bytes_per_pixel: Option<f32>,
		pub chat_enabled: Option<bool>,
		pub chat_filter: Option<bool>,
		pub compression: Option<bool>,
		pub fps: Option<f32>,
		pub bitrate: Option<f32>,
		pub microphone_toggle_behavior: Option<bool>,
		pub microphone_volume: Option<f32>,
		pub preferred_server: Option<String>,
		pub send_metadata: Option<bool>,
		pub system_volume: Option<f32>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
	#[serde(default)]
	pub struct ClientOptions {
		pub data_version: Option<i16>,
		pub video: VideoOptions,
		pub control: ControlOptions,
		pub chat: ChatOptions,
		pub sound: SoundOptions,
		pub skin: SkinOptions,
		pub stream: StreamOptions,
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
		pub snooper_enabled: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Clone, Debug)]
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

	#[derive(Deserialize, Serialize, Clone, Debug)]
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

	#[derive(Deserialize, Serialize, Clone, Debug)]
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

	#[derive(Deserialize, Serialize, Clone, Debug)]
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

	#[derive(Deserialize, Serialize, Clone, Debug)]
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

	#[derive(Deserialize, Serialize, Debug, Clone)]
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
	#[derive(Deserialize, Serialize, Debug, Clone)]
	pub struct FullscreenResolution {
		pub width: u32,
		pub height: u32,
		pub refresh_rate: u32,
		pub color_bits: u32,
	}
}
