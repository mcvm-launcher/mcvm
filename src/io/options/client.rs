use std::{fmt::Display, collections::HashMap, io::Write};

use serde::Deserialize;

use crate::util::{mojang::TARGET_64_BIT, ToInt, versions::VersionPattern};

use super::read::EnumOrNumber;

#[derive(Deserialize, Debug, Clone)]
pub struct KeyOptions {
	#[serde(default = "default_key_attack")]
	pub attack: String,
	#[serde(default = "default_key_use")]
	pub r#use: String,
	#[serde(default = "default_key_forward")]
	pub forward: String,
	#[serde(default = "default_key_left")]
	pub left: String,
	#[serde(default = "default_key_back")]
	pub back: String,
	#[serde(default = "default_key_right")]
	pub right: String,
	#[serde(default = "default_key_jump")]
	pub jump: String,
	#[serde(default = "default_key_sneak")]
	pub sneak: String,
	#[serde(default = "default_key_sprint")]
	pub sprint: String,
	#[serde(default = "default_key_drop")]
	pub drop: String,
	#[serde(default = "default_key_inventory")]
	pub inventory: String,
	#[serde(default = "default_key_chat")]
	pub chat: String,
	#[serde(default = "default_key_playerlist")]
	pub playerlist: String,
	#[serde(default = "default_key_pick_item")]
	pub pick_item: String,
	#[serde(default = "default_key_command")]
	pub command: String,
	#[serde(default = "default_key_social_interactions")]
	pub social_interactions: String,
	#[serde(default = "default_key_screenshot")]
	pub screenshot: String,
	#[serde(default = "default_key_toggle_perspective")]
	pub toggle_perspective: String,
	#[serde(default = "default_key_smooth_camera")]
	pub smooth_camera: String,
	#[serde(default = "default_key_fullscreen")]
	pub fullscreen: String,
	#[serde(default = "default_key_spectator_outlines")]
	pub spectator_outlines: String,
	#[serde(default = "default_key_swap_offhand")]
	pub swap_offhand: String,
	#[serde(default = "default_key_save_toolbar")]
	pub save_toolbar: String,
	#[serde(default = "default_key_load_toolbar")]
	pub load_toolbar: String,
	#[serde(default = "default_key_advancements")]
	pub advancements: String,
	#[serde(default = "default_key_hotbar_1")]
	pub hotbar_1: String,
	#[serde(default = "default_key_hotbar_2")]
	pub hotbar_2: String,
	#[serde(default = "default_key_hotbar_3")]
	pub hotbar_3: String,
	#[serde(default = "default_key_hotbar_4")]
	pub hotbar_4: String,
	#[serde(default = "default_key_hotbar_5")]
	pub hotbar_5: String,
	#[serde(default = "default_key_hotbar_6")]
	pub hotbar_6: String,
	#[serde(default = "default_key_hotbar_7")]
	pub hotbar_7: String,
	#[serde(default = "default_key_hotbar_8")]
	pub hotbar_8: String,
	#[serde(default = "default_key_hotbar_9")]
	pub hotbar_9: String,
}

impl Default for KeyOptions {
	fn default() -> Self {
		Self {
			attack: default_key_attack(),
			r#use: default_key_use(),
			forward: default_key_forward(),
			left: default_key_left(),
			back: default_key_back(),
			right: default_key_right(),
			jump: default_key_jump(),
			sneak: default_key_sneak(),
			sprint: default_key_sprint(),
			drop: default_key_drop(),
			inventory: default_key_inventory(),
			chat: default_key_chat(),
			playerlist: default_key_playerlist(),
			pick_item: default_key_pick_item(),
			command: default_key_command(),
			social_interactions: default_key_social_interactions(),
			screenshot: default_key_screenshot(),
			toggle_perspective: default_key_toggle_perspective(),
			smooth_camera: default_key_smooth_camera(),
			fullscreen: default_key_fullscreen(),
			spectator_outlines: default_key_spectator_outlines(),
			swap_offhand: default_key_swap_offhand(),
			save_toolbar: default_key_save_toolbar(),
			load_toolbar: default_key_load_toolbar(),
			advancements: default_key_advancements(),
			hotbar_1: default_key_hotbar_1(),
			hotbar_2: default_key_hotbar_2(),
			hotbar_3: default_key_hotbar_3(),
			hotbar_4: default_key_hotbar_4(),
			hotbar_5: default_key_hotbar_5(),
			hotbar_6: default_key_hotbar_6(),
			hotbar_7: default_key_hotbar_7(),
			hotbar_8: default_key_hotbar_8(),
			hotbar_9: default_key_hotbar_9(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct ControlOptions {
	#[serde(default)]
	pub keys: KeyOptions,
	#[serde(default = "default_auto_jump")]
	pub auto_jump: bool,
	#[serde(default = "default_discrete_mouse_scroll")]
	pub discrete_mouse_scroll: bool,
	#[serde(default = "default_invert_mouse_y")]
	pub invert_mouse_y: bool,
	#[serde(default = "default_enable_touchscreen")]
	pub enable_touchscreen: bool,
	#[serde(default = "default_toggle_sprint")]
	pub toggle_sprint: bool,
	#[serde(default = "default_toggle_crouch")]
	pub toggle_crouch: bool,
	#[serde(default = "default_mouse_sensitivity")]
	pub mouse_sensitivity: f32,
	#[serde(default = "default_mouse_wheel_sensitivity")]
	pub mouse_wheel_sensitivity: f32,
	#[serde(default = "default_raw_mouse_input")]
	pub raw_mouse_input: bool,
}

impl Default for ControlOptions {
	fn default() -> Self {
		Self {
			keys: KeyOptions::default(),
			auto_jump: default_auto_jump(),
			discrete_mouse_scroll: default_discrete_mouse_scroll(),
			invert_mouse_y: default_invert_mouse_y(),
			enable_touchscreen: default_enable_touchscreen(),
			toggle_sprint: default_toggle_sprint(),
			toggle_crouch: default_toggle_crouch(),
			mouse_sensitivity: default_mouse_sensitivity(),
			mouse_wheel_sensitivity: default_mouse_wheel_sensitivity(),
			raw_mouse_input: default_raw_mouse_input(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChatOptions {
	#[serde(default = "default_auto_command_suggestions")]
	pub auto_command_suggestions: bool,
	#[serde(default = "default_enable_chat_colors")]
	pub enable_colors: bool,
	#[serde(default = "default_enable_chat_links")]
	pub enable_links: bool,
	#[serde(default = "default_prompt_links")]
	pub prompt_links: bool,
	#[serde(default = "default_force_unicode")]
	pub force_unicode: bool,
	#[serde(default = "default_chat_visibility")]
	pub visibility: EnumOrNumber<ChatVisibility>,
	#[serde(default = "default_chat_opacity")]
	pub opacity: f32,
	#[serde(default = "default_chat_line_spacing")]
	pub line_spacing: f32,
	#[serde(default = "default_text_background_opacity")]
	pub background_opacity: f32,
	#[serde(default = "default_background_for_chat_only")]
	pub background_for_chat_only: bool,
	#[serde(default = "default_chat_focused_height")]
	pub focused_height: f32,
	#[serde(default = "default_chat_unfocused_height")]
	pub unfocused_height: f32,
	#[serde(default = "default_chat_delay")]
	pub delay: f32,
	#[serde(default = "default_chat_scale")]
	pub scale: f32,
	#[serde(default = "default_chat_width")]
	pub width: f32,
	#[serde(default = "default_narrator_mode")]
	pub narrator_mode: EnumOrNumber<NarratorMode>,
}

impl Default for ChatOptions {
	fn default() -> Self {
		Self {
			auto_command_suggestions: default_auto_command_suggestions(),
			enable_colors: default_enable_chat_colors(),
			enable_links: default_enable_chat_links(),
			prompt_links: default_prompt_links(),
			force_unicode: default_force_unicode(),
			visibility: default_chat_visibility(),
			opacity: default_chat_opacity(),
			line_spacing: default_chat_line_spacing(),
			background_opacity: default_text_background_opacity(),
			background_for_chat_only: default_background_for_chat_only(),
			focused_height: default_chat_focused_height(),
			unfocused_height: default_chat_unfocused_height(),
			delay: default_chat_delay(),
			scale: default_chat_scale(),
			width: default_chat_width(),
			narrator_mode: default_narrator_mode(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct VideoOptions {
	#[serde(default = "default_vsync")]
	pub vsync: bool,
	#[serde(default = "default_entity_shadows")]
	pub entity_shadows: bool,
	#[serde(default = "default_fullscreen")]
	pub fullscreen: bool,
	#[serde(default = "default_view_bobbing")]
	pub view_bobbing: bool,
	#[serde(default = "default_dark_mojang_background")]
	pub dark_mojang_background: bool,
	#[serde(default = "default_hide_lightning_flashes")]
	pub hide_lightning_flashes: bool,
	#[serde(default = "default_fov")]
	pub fov: u8,
	#[serde(default = "default_screen_effect_scale")]
	pub screen_effect_scale: f32,
	#[serde(default = "default_fov_effect_scale")]
	pub fov_effect_scale: f32,
	#[serde(default = "default_darkness_effect_scale")]
	pub darkness_effect_scale: f32,
	#[serde(default = "default_brightness")]
	pub brightness: f32,
	#[serde(default = "default_render_distance")]
	pub render_distance: u8,
	#[serde(default = "default_simulation_distance")]
	pub simulation_distance: u8,
	#[serde(default = "default_entity_distance_scaling")]
	pub entity_distance_scaling: f32,
	#[serde(default = "default_gui_scale")]
	pub gui_scale: u8,
	#[serde(default = "default_particles")]
	pub particles: EnumOrNumber<ParticlesMode>,
	#[serde(default = "default_max_fps")]
	pub max_fps: u8,
	#[serde(default = "default_graphics_mode")]
	pub graphics_mode: EnumOrNumber<GraphicsMode>,
	#[serde(default = "default_smooth_lighting")]
	pub smooth_lighting: bool,
	#[serde(default = "default_chunk_updates_mode")]
	pub chunk_updates_mode: EnumOrNumber<ChunkUpdatesMode>,
	#[serde(default = "default_biome_blend")]
	pub biome_blend: u8,
	#[serde(default = "default_clouds")]
	pub clouds: CloudRenderMode,
	#[serde(default = "default_mipmap_levels")]
	pub mipmap_levels: u8,
	#[serde(default = "default_window_width")]
	pub window_width: u16,
	#[serde(default = "default_window_height")]
	pub window_height: u16,
	#[serde(default = "default_attack_indicator")]
	pub attack_indicator: EnumOrNumber<AttackIndicatorMode>,
	#[serde(default = "default_fullscreen_resolution")]
	pub fullscreen_resolution: Option<FullscreenResolution>,
	#[serde(default = "default_allow_block_alternatives")]
	pub allow_block_alternatives: bool,
}

impl Default for VideoOptions {
	fn default() -> Self {
		Self {
			vsync: default_vsync(),
			entity_shadows: default_entity_shadows(),
			fullscreen: default_fullscreen(),
			view_bobbing: default_view_bobbing(),
			dark_mojang_background: default_dark_mojang_background(),
			hide_lightning_flashes: default_hide_lightning_flashes(),
			fov: default_fov(),
			screen_effect_scale: default_screen_effect_scale(),
			fov_effect_scale: default_fov_effect_scale(),
			darkness_effect_scale: default_darkness_effect_scale(),
			brightness: default_brightness(),
			render_distance: default_render_distance(),
			simulation_distance: default_simulation_distance(),
			entity_distance_scaling: default_entity_distance_scaling(),
			gui_scale: default_gui_scale(),
			particles: default_particles(),
			max_fps: default_max_fps(),
			graphics_mode: default_graphics_mode(),
			smooth_lighting: default_smooth_lighting(),
			chunk_updates_mode: default_chunk_updates_mode(),
			biome_blend: default_biome_blend(),
			clouds: default_clouds(),
			mipmap_levels: default_mipmap_levels(),
			window_width: default_window_width(),
			window_height: default_window_height(),
			attack_indicator: default_attack_indicator(),
			fullscreen_resolution: default_fullscreen_resolution(),
			allow_block_alternatives: default_allow_block_alternatives(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct VolumeOptions {
	#[serde(default = "default_sound_volume")]
	pub master: f32,
	#[serde(default = "default_sound_volume")]
	pub music: f32,
	#[serde(default = "default_sound_volume")]
	pub record: f32,
	#[serde(default = "default_sound_volume")]
	pub weather: f32,
	#[serde(default = "default_sound_volume")]
	pub block: f32,
	#[serde(default = "default_sound_volume")]
	pub hostile: f32,
	#[serde(default = "default_sound_volume")]
	pub neutral: f32,
	#[serde(default = "default_sound_volume")]
	pub player: f32,
	#[serde(default = "default_sound_volume")]
	pub ambient: f32,
	#[serde(default = "default_sound_volume")]
	pub voice: f32,
}

impl Default for VolumeOptions {
	fn default() -> Self {
		Self {
			master: default_sound_volume(),
			music: default_sound_volume(),
			record: default_sound_volume(),
			weather: default_sound_volume(),
			block: default_sound_volume(),
			hostile: default_sound_volume(),
			neutral: default_sound_volume(),
			player: default_sound_volume(),
			ambient: default_sound_volume(),
			voice: default_sound_volume(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct SoundOptions {
	#[serde(default)]
	pub volume: VolumeOptions,
	#[serde(default = "default_show_subtitles")]
	pub show_subtitles: bool,
	#[serde(default = "default_directional_audio")]
	pub directional_audio: bool,
	#[serde(default = "default_sound_device")]
	pub device: Option<String>,
}

impl Default for SoundOptions {
	fn default() -> Self {
		Self {
			volume: VolumeOptions::default(),
			show_subtitles: default_show_subtitles(),
			directional_audio: default_directional_audio(),
			device: default_sound_device(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct SkinOptions {
	#[serde(default = "default_skin_part")]
	pub cape: bool,
	#[serde(default = "default_skin_part")]
	pub jacket: bool,
	#[serde(default = "default_skin_part")]
	pub left_sleeve: bool,
	#[serde(default = "default_skin_part")]
	pub right_sleeve: bool,
	#[serde(default = "default_skin_part")]
	pub left_pants: bool,
	#[serde(default = "default_skin_part")]
	pub right_pants: bool,
	#[serde(default = "default_skin_part")]
	pub hat: bool,
}

impl Default for SkinOptions {
	fn default() -> Self {
		Self {
			cape: default_skin_part(),
			jacket: default_skin_part(),
			left_sleeve: default_skin_part(),
			right_sleeve: default_skin_part(),
			left_pants: default_skin_part(),
			right_pants: default_skin_part(),
			hat: default_skin_part(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientOptions {
	#[serde(default = "default_data_version")]
	pub data_version: i16,
	#[serde(default)]
	pub video: VideoOptions,
	#[serde(default)]
	pub control: ControlOptions,
	#[serde(default)]
	pub chat: ChatOptions,
	#[serde(default)]
	pub sound: SoundOptions,
	#[serde(default)]
	pub skin: SkinOptions,
	#[serde(default)]
	pub custom: HashMap<String, String>,
	#[serde(default = "default_realms_notifications")]
	pub realms_notifications: bool,
	#[serde(default = "default_reduced_debug_info")]
	pub reduced_debug_info: bool,
	#[serde(default = "default_difficulty")]
	pub difficulty: EnumOrNumber<Difficulty>,
	#[serde(default = "default_resource_packs")]
	pub resource_packs: Vec<String>,
	#[serde(default = "default_language")]
	pub language: String,
	#[serde(default = "default_tutorial_step")]
	pub tutorial_step: TutorialStep,
	#[serde(default = "default_skip_multiplayer_warning")]
	pub skip_multiplayer_warning: bool,
	#[serde(default = "default_skip_realms_32_bit_warning")]
	pub skip_realms_32_bit_warning: bool,
	#[serde(default = "default_hide_bundle_tutorial")]
	pub hide_bundle_tutorial: bool,
	#[serde(default = "default_joined_server")]
	pub joined_server: bool,
	#[serde(default = "default_sync_chunk_writes")]
	pub sync_chunk_writes: bool,
	#[serde(default = "default_use_native_transport")]
	pub use_native_transport: bool,
	#[serde(default = "default_held_item_tooltips")]
	pub held_item_tooltips: bool,
	#[serde(default = "default_advanced_item_tooltips")]
	pub advanced_item_tooltips: bool,
	#[serde(default = "default_log_level")]
	pub log_level: EnumOrNumber<LogLevel>,
	#[serde(default = "default_hide_matched_names")]
	pub hide_matched_names: bool,
	#[serde(default = "default_pause_on_lost_focus")]
	pub pause_on_lost_focus: bool,
	#[serde(default = "default_main_hand")]
	pub main_hand: MainHand,
	#[serde(default = "default_hide_server_address")]
	pub hide_server_address: bool,
	#[serde(default = "default_show_autosave_indicator")]
	pub show_autosave_indicator: bool,
	#[serde(default = "default_allow_server_listing")]
	pub allow_server_listing: bool,
}

impl Default for ClientOptions {
	fn default() -> Self {
		Self {
			data_version: default_data_version(),
			video: VideoOptions::default(),
			control: ControlOptions::default(),
			chat: ChatOptions::default(),
			sound: SoundOptions::default(),
			skin: SkinOptions::default(),
			custom: HashMap::default(),
			realms_notifications: default_realms_notifications(),
			reduced_debug_info: default_reduced_debug_info(),
			difficulty: default_difficulty(),
			resource_packs: default_resource_packs(),
			language: default_language(),
			tutorial_step: default_tutorial_step(),
			skip_multiplayer_warning: default_skip_multiplayer_warning(),
			skip_realms_32_bit_warning: default_skip_realms_32_bit_warning(),
			hide_bundle_tutorial: default_hide_bundle_tutorial(),
			joined_server: default_joined_server(),
			sync_chunk_writes: default_sync_chunk_writes(),
			use_native_transport: default_use_native_transport(),
			held_item_tooltips: default_held_item_tooltips(),
			advanced_item_tooltips: default_advanced_item_tooltips(),
			log_level: default_log_level(),
			hide_matched_names: default_hide_matched_names(),
			pause_on_lost_focus: default_pause_on_lost_focus(),
			main_hand: default_main_hand(),
			hide_server_address: default_hide_server_address(),
			show_autosave_indicator: default_show_autosave_indicator(),
			allow_server_listing: default_allow_server_listing(),
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
		write!(f, "{}", match self {
			Self::Fancy => "true",
			Self::Off => "false",
			Self::Fast => "fast",
		})
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
		write!(f, "{}", match self {
			Self::Left => "left",
			Self::Right => "right",
		})
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
		write!(f, "{}", match self {
			Self::Movement => "movement",
			Self::FindTree => "find_tree",
			Self::PunchTree => "punch_tree",
			Self::OpenInventory => "open_inventory",
			Self::CraftPlanks => "craft_planks",
			Self::None => "none",
		})
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

fn default_data_version() -> i16 { 3337 }
fn default_auto_jump() -> bool { true }
fn default_auto_command_suggestions() -> bool { true }
fn default_enable_chat_colors() -> bool { true }
fn default_enable_chat_links() -> bool { true }
fn default_prompt_links() -> bool { true }
fn default_vsync() -> bool { true }
fn default_entity_shadows() -> bool { true }
fn default_force_unicode() -> bool { false }
fn default_discrete_mouse_scroll() -> bool { false }
fn default_invert_mouse_y() -> bool { false }
fn default_realms_notifications() -> bool { true }
fn default_reduced_debug_info() -> bool { false }
fn default_show_subtitles() -> bool { false }
fn default_directional_audio() -> bool { false }
fn default_enable_touchscreen() -> bool { false }
fn default_fullscreen() -> bool { false }
fn default_view_bobbing() -> bool { true }
fn default_toggle_sprint() -> bool { false }
fn default_toggle_crouch() -> bool { false }
fn default_dark_mojang_background() -> bool { false }
fn default_hide_lightning_flashes() -> bool { false }
fn default_mouse_sensitivity() -> f32 { 0.5 }
fn default_fov() -> u8 { 0 }
fn default_screen_effect_scale() -> f32 { 1.0 }
fn default_fov_effect_scale() -> f32 { 1.0 }
fn default_darkness_effect_scale() -> f32 { 1.0 }
fn default_brightness() -> f32 { 0.5 }
fn default_render_distance() -> u8 {
	if TARGET_64_BIT {
		12
	} else {
		8
	}
}
fn default_simulation_distance() -> u8 { default_render_distance() }
fn default_entity_distance_scaling() -> f32 { 1.0 }
fn default_gui_scale() -> u8 { 0 }
fn default_particles() -> EnumOrNumber<ParticlesMode> { EnumOrNumber::Enum(ParticlesMode::All) }
fn default_max_fps() -> u8 { 120 }
fn default_difficulty() -> EnumOrNumber<Difficulty> { EnumOrNumber::Enum(Difficulty::Normal) }
fn default_graphics_mode() -> EnumOrNumber<GraphicsMode> { EnumOrNumber::Enum(GraphicsMode::Fancy) }
fn default_smooth_lighting() -> bool { true }
fn default_chunk_updates_mode() -> EnumOrNumber<ChunkUpdatesMode> { EnumOrNumber::Enum(ChunkUpdatesMode::Threaded) }
fn default_biome_blend() -> u8 { 2 }
fn default_clouds() -> CloudRenderMode { CloudRenderMode::Fancy }
fn default_resource_packs() -> Vec<String> { vec![] }
fn default_language() -> String { String::from("en_us") }
fn default_sound_device() -> Option<String> { None }
fn default_chat_visibility() -> EnumOrNumber<ChatVisibility> { EnumOrNumber::Enum(ChatVisibility::Shown) }
fn default_chat_opacity() -> f32 { 1.0 }
fn default_chat_line_spacing() -> f32 { 0.0 }
fn default_text_background_opacity() -> f32 { 0.5 }
fn default_background_for_chat_only() -> bool { true }
fn default_hide_server_address() -> bool { false }
fn default_advanced_item_tooltips() -> bool { false }
fn default_pause_on_lost_focus() -> bool { false }
fn default_window_width() -> u16 { 0 }
fn default_window_height() -> u16 { 0 }
fn default_held_item_tooltips() -> bool { true }
fn default_chat_focused_height() -> f32 { 1.0 }
fn default_chat_unfocused_height() -> f32 { 0.4375 }
fn default_chat_delay() -> f32 { 0.0 }
fn default_chat_scale() -> f32 { 1.0 }
fn default_chat_width() -> f32 { 1.0 }
fn default_mipmap_levels() -> u8 { 4 }
fn default_use_native_transport() -> bool { true }
fn default_main_hand() -> MainHand { MainHand::Right }
fn default_attack_indicator() -> EnumOrNumber<AttackIndicatorMode> { EnumOrNumber::Enum(AttackIndicatorMode::Crosshair) }
fn default_narrator_mode() -> EnumOrNumber<NarratorMode> { EnumOrNumber::Enum(NarratorMode::Off) }
fn default_tutorial_step() -> TutorialStep { TutorialStep::None }
fn default_mouse_wheel_sensitivity() -> f32 { 1.0 }
fn default_raw_mouse_input() -> bool { true }
fn default_log_level() -> EnumOrNumber<LogLevel> { EnumOrNumber::Enum(LogLevel::High) }
fn default_skip_multiplayer_warning() -> bool { true }
fn default_skip_realms_32_bit_warning() -> bool { true }
fn default_hide_matched_names() -> bool { true }
fn default_joined_server() -> bool { true }
fn default_hide_bundle_tutorial() -> bool { true }
fn default_sync_chunk_writes() -> bool {
	if cfg!(target_os = "windows") {
		false
	} else {
		true
	}
}
fn default_show_autosave_indicator() -> bool { true }
fn default_allow_server_listing() -> bool { true }
fn default_sound_volume() -> f32 { 1.0 }
fn default_fullscreen_resolution() -> Option<FullscreenResolution> { None }
fn default_key_attack() -> String { String::from("key.mouse.left") }
fn default_key_use() -> String { String::from("key.mouse.right") }
fn default_key_forward() -> String { String::from("key.keyboard.w") }
fn default_key_left() -> String { String::from("key.keyboard.a") }
fn default_key_back() -> String { String::from("key.keyboard.s") }
fn default_key_right() -> String { String::from("key.keyboard.d") }
fn default_key_jump() -> String { String::from("key.keyboard.space") }
fn default_key_sneak() -> String { String::from("key.keyboard.left.control") }
fn default_key_sprint() -> String { String::from("key.keyboard.left.shift") }
fn default_key_drop() -> String { String::from("key.keyboard.q") }
fn default_key_inventory() -> String { String::from("key.keyboard.e") }
fn default_key_chat() -> String { String::from("key.keyboard.t") }
fn default_key_playerlist() -> String { String::from("key.keyboard.tab") }
fn default_key_pick_item() -> String { String::from("key.mouse.middle") }
fn default_key_command() -> String { String::from("key.keyboard.slash") }
fn default_key_social_interactions() -> String { String::from("key.keyboard.p") }
fn default_key_screenshot() -> String { String::from("key.keyboard.f2") }
fn default_key_toggle_perspective() -> String { String::from("key.keyboard.f5") }
fn default_key_smooth_camera() -> String { String::from("key.keyboard.unknown") }
fn default_key_fullscreen() -> String { String::from("key.keyboard.f11") }
fn default_key_spectator_outlines() -> String { String::from("key.keyboard.unknown") }
fn default_key_swap_offhand() -> String { String::from("key.keyboard.f") }
fn default_key_save_toolbar() -> String { String::from("key.keyboard.c") }
fn default_key_load_toolbar() -> String { String::from("key.keyboard.x") }
fn default_key_advancements() -> String { String::from("key.keyboard.l") }
fn default_key_hotbar_1() -> String { String::from("key.keyboard.1") }
fn default_key_hotbar_2() -> String { String::from("key.keyboard.2") }
fn default_key_hotbar_3() -> String { String::from("key.keyboard.3") }
fn default_key_hotbar_4() -> String { String::from("key.keyboard.4") }
fn default_key_hotbar_5() -> String { String::from("key.keyboard.5") }
fn default_key_hotbar_6() -> String { String::from("key.keyboard.6") }
fn default_key_hotbar_7() -> String { String::from("key.keyboard.7") }
fn default_key_hotbar_8() -> String { String::from("key.keyboard.8") }
fn default_key_hotbar_9() -> String { String::from("key.keyboard.9") }
fn default_skin_part() -> bool { true }
fn default_allow_block_alternatives() -> bool { true }

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

/// Write client options to a list of keys
pub fn create_keys(
	client: &ClientOptions,
	version: &str,
	versions: &[String],
) -> anyhow::Result<HashMap<String, String>> {
	let mut out = HashMap::new();

	// Version checks
	let after_12w50a = VersionPattern::After(String::from("12w50a")).matches_single(version, versions);
	let after_13w36a = VersionPattern::After(String::from("13w36a")).matches_single(version, versions);
	let after_14w25a = VersionPattern::After(String::from("14w25a")).matches_single(version, versions);
	let after_14w28a = VersionPattern::After(String::from("14w28a")).matches_single(version, versions);
	let after_17w06a = VersionPattern::After(String::from("17w06a")).matches_single(version, versions);
	let after_17w47a = VersionPattern::After(String::from("17w47a")).matches_single(version, versions);
	let after_18w15a = VersionPattern::After(String::from("18w15a")).matches_single(version, versions);
	let after_18w21a = VersionPattern::After(String::from("18w21a")).matches_single(version, versions);
	let after_1_13_pre2 = VersionPattern::After(String::from("1.13-pre2")).matches_single(version, versions);
	let after_1_15_2_pre1 = VersionPattern::After(String::from("1.15.2-pre1")).matches_single(version, versions);
	let after_1_16_4_rc1 = VersionPattern::After(String::from("1.16.4-rc1")).matches_single(version, versions);
	let after_21w13a = VersionPattern::After(String::from("21w13a")).matches_single(version, versions);
	let after_21w37a = VersionPattern::After(String::from("21w37a")).matches_single(version, versions);
	let after_21w38a = VersionPattern::After(String::from("21w38a")).matches_single(version, versions);
	let after_21w42a = VersionPattern::After(String::from("21w42a")).matches_single(version, versions);
	let after_1_18_pre2 = VersionPattern::After(String::from("1.18-pre2")).matches_single(version, versions);
	let after_1_18_2_pre1 = VersionPattern::After(String::from("1.18.2-pre1")).matches_single(version, versions);
	let after_22w11a = VersionPattern::After(String::from("22w11a")).matches_single(version, versions);
	let after_22w15a = VersionPattern::After(String::from("22w15a")).matches_single(version, versions);

	let before_13w42a = VersionPattern::Before(String::from("13w42a")).matches_single(version, versions);
	let before_15w31a = VersionPattern::Before(String::from("15w31a")).matches_single(version, versions);
	let before_20w27a = VersionPattern::Before(String::from("20w27a")).matches_single(version, versions);
	let before_1_19_4 = VersionPattern::Before(String::from("1.19.4")).matches_single(version, versions);

	// TODO: Add actual data version
	// out.insert(String::from("version"), client.data_version.to_string());
	out.insert(String::from("autoJump"), client.control.auto_jump.to_string());
	out.insert(String::from("fullscreen"), client.video.fullscreen.to_string());
	if after_17w47a {
		out.insert(String::from("autoSuggestions"), client.chat.auto_command_suggestions.to_string());
	}
	out.insert(String::from("chatColors"), client.chat.enable_colors.to_string());
	out.insert(String::from("chatLinks"), client.chat.enable_links.to_string());
	out.insert(String::from("chatLinksPrompt"), client.chat.prompt_links.to_string());
	out.insert(String::from("enableVsync"), client.video.vsync.to_string());
	out.insert(String::from("entityShadows"), client.video.entity_shadows.to_string());
	out.insert(String::from("forceUnicodeFont"), client.chat.force_unicode.to_string());
	out.insert(String::from("discrete_mouse_scroll"), client.control.discrete_mouse_scroll.to_string());
	out.insert(String::from("invertYMouse"), client.control.invert_mouse_y.to_string());
	out.insert(String::from("realmsNotifications"), client.realms_notifications.to_string());
	out.insert(String::from("reducedDebugInfo"), client.reduced_debug_info.to_string());
	out.insert(String::from("showSubtitles"), client.sound.show_subtitles.to_string());
	if after_22w11a {
		out.insert(String::from("directionalAudio"), client.sound.directional_audio.to_string());
	}
	out.insert(String::from("touchscreen"), client.control.enable_touchscreen.to_string());
	out.insert(String::from("bobView"), client.video.view_bobbing.to_string());
	out.insert(String::from("toggleCrouch"), client.control.toggle_crouch.to_string());
	out.insert(String::from("toggleSprint"), client.control.toggle_sprint.to_string());
	if after_21w13a {
		out.insert(String::from("darkMojangStudiosBackground"), client.video.dark_mojang_background.to_string());
	}
	if after_21w37a {
		out.insert(String::from("hideLightningFlashes"), client.video.hide_lightning_flashes.to_string());
		out.insert(String::from("prioritizeChunkUpdates"), client.video.chunk_updates_mode.to_int().to_string());
		if let Some(device) = &client.sound.device {
			out.insert(String::from("soundDevice"), device.clone());
		}
	}
	out.insert(String::from("mouseSensitivity"), client.control.mouse_sensitivity.to_string());
	out.insert(String::from("fov"), client.video.fov.to_string());
	out.insert(String::from("screenEffectScale"), client.video.screen_effect_scale.to_string());
	out.insert(String::from("fovEffectScale"), client.video.fov_effect_scale.to_string());
	if after_22w15a {
		out.insert(String::from("darknessEffectScale"), client.video.darkness_effect_scale.to_string());
	}
	out.insert(String::from("gamma"), client.video.brightness.to_string());
	out.insert(String::from("renderDistance"), client.video.render_distance.to_string());
	if after_21w38a {
		out.insert(String::from("simulationDistance"), client.video.simulation_distance.to_string());
	}
	out.insert(String::from("entityDistanceScaling"), client.video.entity_distance_scaling.to_string());
	out.insert(String::from("guiScale"), client.video.gui_scale.to_string());
	out.insert(String::from("particles"), client.video.particles.to_int().to_string());
	out.insert(String::from("maxFps"), client.video.max_fps.to_string());
	out.insert(String::from("difficulty"), client.difficulty.to_int().to_string());
	if before_20w27a {
		out.insert(String::from("fancyGraphics"), match client.video.graphics_mode {
			EnumOrNumber::Enum(GraphicsMode::Fast) => false,
			EnumOrNumber::Enum(GraphicsMode::Fancy | GraphicsMode::Fabulous) => true,
			EnumOrNumber::Num(num) => num > 0,
		}.to_string());
	} else {
		out.insert(String::from("graphicsMode"), client.video.graphics_mode.to_int().to_string());
	}
	out.insert(String::from("ao"), client.video.smooth_lighting.to_string());
	if after_18w15a {
		out.insert(String::from("biomeBlendRadius"), client.video.biome_blend.to_string());
	}
	if after_14w25a {
		out.insert(String::from("renderClouds"), client.video.clouds.to_string());
	} else {
		out.insert(
			String::from("clouds"),
			matches!(client.video.clouds, CloudRenderMode::Fancy | CloudRenderMode::Fast).to_string()
		);
	}
	out.insert(String::from("resourcePacks"), write_resource_packs(&client.resource_packs));
	out.insert(String::from("incompatibleResourcePacks"), String::from("[]"));
	out.insert(String::from("lang"), client.language.clone());
	out.insert(String::from("chatVisibility"), client.chat.visibility.to_int().to_string());
	out.insert(String::from("chatOpacity"), client.chat.opacity.to_string());
	out.insert(String::from("chatLineSpacing"), client.chat.line_spacing.to_string());
	out.insert(String::from("textBackgroundOpacity"), client.chat.background_opacity.to_string());
	out.insert(String::from("backgroundForChatOnly"), client.chat.background_for_chat_only.to_string());
	out.insert(String::from("hideServerAddress"), client.hide_server_address.to_string());
	out.insert(String::from("advancedItemTooltips"), client.advanced_item_tooltips.to_string());
	out.insert(String::from("pauseOnLostFocus"), client.pause_on_lost_focus.to_string());
	out.insert(String::from("overrideWidth"), client.video.window_width.to_string());
	out.insert(String::from("overrideHeight"), client.video.window_height.to_string());
	if after_12w50a && before_1_19_4 {
		out.insert(String::from("heldItemTooltips"), client.held_item_tooltips.to_string());
	}
	out.insert(String::from("chatHeightFocused"), client.chat.focused_height.to_string());
	out.insert(String::from("chatDelay"), client.chat.delay.to_string());
	out.insert(String::from("chatHeightUnfocused"), client.chat.unfocused_height.to_string());
	out.insert(String::from("chatScale"), client.chat.scale.to_string());
	out.insert(String::from("chatWidth"), client.chat.width.to_string());
	out.insert(String::from("mipmapLevels"), client.video.mipmap_levels.to_string());
	out.insert(String::from("useNativeTransport"), client.use_native_transport.to_string());
	out.insert(String::from("mainHand"), client.main_hand.to_string());
	if after_17w06a {
		out.insert(String::from("narrator"), client.chat.narrator_mode.to_int().to_string());
		out.insert(String::from("tutorialStep"), client.tutorial_step.to_string());
	}
	if after_18w21a {
		out.insert(String::from("mouseWheelSensitivity"), client.control.mouse_wheel_sensitivity.to_string());
	}
	out.insert(String::from("rawMouseInput"), client.control.raw_mouse_input.to_string());
	if after_1_13_pre2 {
		out.insert(String::from("glDebugVerbosity"), client.log_level.to_int().to_string());
	}
	if after_1_15_2_pre1 {
		out.insert(String::from("skipMultiplayerWarning"), client.skip_multiplayer_warning.to_string());
	}
	if after_1_18_2_pre1 {
		out.insert(String::from("skipRealms32bitWarning"), client.skip_realms_32_bit_warning.to_string());
	}
	if after_1_16_4_rc1 {
		out.insert(String::from("hideMatchedNames"), client.hide_matched_names.to_string());
		out.insert(String::from("joinedFirstServer"), client.joined_server.to_string());
	}
	out.insert(String::from("hideBundleTutorial"), client.hide_bundle_tutorial.to_string());
	out.insert(String::from("syncChunkWrites"), client.sync_chunk_writes.to_string());
	if after_21w42a {
		out.insert(String::from("showAutosaveIndicator"), client.show_autosave_indicator.to_string());
	}
	if after_1_18_pre2 {
		out.insert(String::from("allowServerListing"), client.allow_server_listing.to_string());
	}
	// Keybinds
	out.insert(String::from("key_key.attack"), client.control.keys.attack.clone());
	out.insert(String::from("key_key.use"), client.control.keys.r#use.clone());
	out.insert(String::from("key_key.forward"), client.control.keys.forward.clone());
	out.insert(String::from("key_key.left"), client.control.keys.left.clone());
	out.insert(String::from("key_key.back"), client.control.keys.back.clone());
	out.insert(String::from("key_key.right"), client.control.keys.right.clone());
	out.insert(String::from("key_key.jump"), client.control.keys.jump.clone());
	out.insert(String::from("key_key.sneak"), client.control.keys.sneak.clone());
	out.insert(String::from("key_key.sprint"), client.control.keys.sprint.clone());
	out.insert(String::from("key_key.drop"), client.control.keys.drop.clone());
	out.insert(String::from("key_key.inventory"), client.control.keys.inventory.clone());
	out.insert(String::from("key_key.chat"), client.control.keys.chat.clone());
	out.insert(String::from("key_key.playerlist"), client.control.keys.playerlist.clone());
	out.insert(String::from("key_key.pickItem"), client.control.keys.pick_item.clone());
	out.insert(String::from("key_key.command"), client.control.keys.command.clone());
	out.insert(String::from("key_key.socialInteractions"), client.control.keys.social_interactions.clone());
	out.insert(String::from("key_key.screenshot"), client.control.keys.screenshot.clone());
	out.insert(String::from("key_key.togglePerspective"), client.control.keys.toggle_perspective.clone());
	out.insert(String::from("key_key.smoothCamera"), client.control.keys.smooth_camera.clone());
	out.insert(String::from("key_key.fullscreen"), client.control.keys.fullscreen.clone());
	out.insert(String::from("key_key.spectatorOutlines"), client.control.keys.spectator_outlines.clone());
	out.insert(String::from("key_key.swapOffhand"), client.control.keys.swap_offhand.clone());
	if after_17w06a {
		out.insert(String::from("key_key.saveToolbarActivator"), client.control.keys.save_toolbar.clone());
		out.insert(String::from("key_key.loadToolbarActivator"), client.control.keys.load_toolbar.clone());
		out.insert(String::from("key_key.advancements"), client.control.keys.advancements.clone());
	}
	out.insert(String::from("key_key.hotbar.1"), client.control.keys.hotbar_1.clone());
	out.insert(String::from("key_key.hotbar.2"), client.control.keys.hotbar_2.clone());
	out.insert(String::from("key_key.hotbar.3"), client.control.keys.hotbar_3.clone());
	out.insert(String::from("key_key.hotbar.4"), client.control.keys.hotbar_4.clone());
	out.insert(String::from("key_key.hotbar.5"), client.control.keys.hotbar_5.clone());
	out.insert(String::from("key_key.hotbar.6"), client.control.keys.hotbar_6.clone());
	out.insert(String::from("key_key.hotbar.7"), client.control.keys.hotbar_7.clone());
	out.insert(String::from("key_key.hotbar.8"), client.control.keys.hotbar_8.clone());
	out.insert(String::from("key_key.hotbar.9"), client.control.keys.hotbar_9.clone());
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
		out.insert(String::from("soundCategory_master"), client.sound.volume.master.to_string());
		out.insert(String::from("soundCategory_music"), client.sound.volume.music.to_string());
		out.insert(String::from(records_key), client.sound.volume.record.to_string());
		out.insert(String::from("soundCategory_weather"), client.sound.volume.weather.to_string());
		out.insert(String::from(blocks_key), client.sound.volume.block.to_string());
		out.insert(String::from(mobs_key), client.sound.volume.hostile.to_string());
		out.insert(String::from(animals_key), client.sound.volume.neutral.to_string());
		out.insert(String::from(players_key), client.sound.volume.player.to_string());
		out.insert(String::from("soundCategory_ambient"), client.sound.volume.ambient.to_string());
		out.insert(String::from("soundCategory_voice"), client.sound.volume.voice.to_string());
	} else {
		let volume_up = client.sound.volume.master > 0.0;
		out.insert(String::from("sound"), volume_up.to_string());
	}
	// Model parts
	out.insert(String::from("modelPart_cape"), client.skin.cape.to_string());
	out.insert(String::from("modelPart_jacket"), client.skin.jacket.to_string());
	out.insert(String::from("modelPart_left_sleeve"), client.skin.left_sleeve.to_string());
	out.insert(String::from("modelPart_right_sleeve"), client.skin.right_sleeve.to_string());
	out.insert(String::from("modelPart_left_pants_leg"), client.skin.left_pants.to_string());
	out.insert(String::from("modelPart_right_pants_leg"), client.skin.right_pants.to_string());
	out.insert(String::from("modelPart_hat"), client.skin.hat.to_string());
	if after_14w28a && before_15w31a {
		out.insert(String::from("allowBlockAlternatives"), client.video.allow_block_alternatives.to_string());
	}

	if let Some(resolution) = &client.video.fullscreen_resolution {
		out.insert(String::from("fullscreenResolution"), write_fullscreen_resolution(resolution));
	}

	let custom_clone = client.custom.clone();
	out.extend(custom_clone);

	Ok(out)
}

/// Write a client options key to a writer
pub fn write_key<W: Write>(key: &str, value: &str, writer: &mut W) -> anyhow::Result<()> {
	writeln!(writer, "{key}:{value}")?;
	
	Ok(())
}

#[cfg(test)]
mod tests {
	use crate::io::options::read::parse_options_str;
	use super::*;

	#[test]
	fn test_create_keys() {
		let options = parse_options_str(r#"{"client": {}, "server": {}}"#).unwrap();
		let versions = [String::from("1.18"), String::from("1.19.3")];
		create_keys(&options.client.unwrap(), "1.19.3", &versions).unwrap();
	}
}
