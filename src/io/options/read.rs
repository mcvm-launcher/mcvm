use anyhow::Context;
use serde::Deserialize;

use crate::util::mojang::TARGET_64_BIT;

#[derive(Deserialize)]
pub struct KeyOptions {
	#[serde(default = "default_key_attack")]
	attack: String,
	#[serde(default = "default_key_use")]
	r#use: String,
	#[serde(default = "default_key_forward")]
	forward: String,
	#[serde(default = "default_key_left")]
	left: String,
	#[serde(default = "default_key_back")]
	back: String,
	#[serde(default = "default_key_right")]
	right: String,
	#[serde(default = "default_key_jump")]
	jump: String,
	#[serde(default = "default_key_sneak")]
	sneak: String,
	#[serde(default = "default_key_sprint")]
	sprint: String,
	#[serde(default = "default_key_drop")]
	drop: String,
	#[serde(default = "default_key_inventory")]
	inventory: String,
	#[serde(default = "default_key_chat")]
	chat: String,
	#[serde(default = "default_key_playerlist")]
	playerlist: String,
	#[serde(default = "default_key_pick_item")]
	pick_item: String,
	#[serde(default = "default_key_command")]
	command: String,
	#[serde(default = "default_key_social_interactions")]
	social_interactions: String,
	#[serde(default = "default_key_screenshot")]
	screenshot: String,
	#[serde(default = "default_key_toggle_perspective")]
	toggle_perspective: String,
	#[serde(default = "default_key_smooth_camera")]
	smooth_camera: String,
	#[serde(default = "default_key_fullscreen")]
	fullscreen: String,
	#[serde(default = "default_key_spectator_outlines")]
	spectator_outlines: String,
	#[serde(default = "default_key_swap_offhand")]
	swap_offhand: String,
	#[serde(default = "default_key_save_toolbar")]
	save_toolbar: String,
	#[serde(default = "default_key_load_toolbar")]
	load_toolbar: String,
	#[serde(default = "default_key_advancements")]
	advancements: String,
	#[serde(default = "default_key_hotbar_1")]
	hotbar_1: String,
	#[serde(default = "default_key_hotbar_2")]
	hotbar_2: String,
	#[serde(default = "default_key_hotbar_3")]
	hotbar_3: String,
	#[serde(default = "default_key_hotbar_4")]
	hotbar_4: String,
	#[serde(default = "default_key_hotbar_5")]
	hotbar_5: String,
	#[serde(default = "default_key_hotbar_6")]
	hotbar_6: String,
	#[serde(default = "default_key_hotbar_7")]
	hotbar_7: String,
	#[serde(default = "default_key_hotbar_8")]
	hotbar_8: String,
	#[serde(default = "default_key_hotbar_9")]
	hotbar_9: String,
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

#[derive(Deserialize)]
pub struct ControlOptions {
	#[serde(default)]
	keys: KeyOptions,
	#[serde(default = "default_auto_jump")]
	auto_jump: bool,
	#[serde(default = "default_discrete_mouse_scroll")]
	discrete_mouse_scroll: bool,
	#[serde(default = "default_invert_mouse_y")]
	invert_mouse_y: bool,
	#[serde(default = "default_enable_touchscreen")]
	enable_touchscreen: bool,
	#[serde(default = "default_toggle_sprint")]
	toggle_sprint: bool,
	#[serde(default = "default_toggle_crouch")]
	toggle_crouch: bool,
	#[serde(default = "default_mouse_sensitivity")]
	mouse_sensitivity: f32,
	#[serde(default = "default_mouse_wheel_sensitivity")]
	mouse_wheel_sensitivity: f32,
	#[serde(default = "default_raw_mouse_input")]
	raw_mouse_input: bool,
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

#[derive(Deserialize)]
pub struct ChatOptions {
	#[serde(default = "default_auto_command_suggestions")]
	auto_command_suggestions: bool,
	#[serde(default = "default_enable_chat_colors")]
	enable_colors: bool,
	#[serde(default = "default_enable_chat_links")]
	enable_links: bool,
	#[serde(default = "default_prompt_links")]
	prompt_links: bool,
	#[serde(default = "default_force_unicode")]
	force_unicode: bool,
	#[serde(default = "default_chat_visibility")]
	visibility: ChatVisibility,
	#[serde(default = "default_chat_opacity")]
	opacity: f32,
	#[serde(default = "default_chat_line_spacing")]
	line_spacing: f32,
	#[serde(default = "default_text_background_opacity")]
	background_opacity: f32,
	#[serde(default = "default_background_for_chat_only")]
	background_for_chat_only: bool,
	#[serde(default = "default_chat_focused_height")]
	focused_height: f32,
	#[serde(default = "default_chat_unfocused_height")]
	unfocused_height: f32,
	#[serde(default = "default_chat_delay")]
	delay: f32,
	#[serde(default = "default_chat_scale")]
	scale: f32,
	#[serde(default = "default_chat_width")]
	width: f32,
	#[serde(default = "default_narrator_mode")]
	narrator_mode: OptionsEnum<NarratorMode>,
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

#[derive(Deserialize)]
pub struct VideoOptions {
	#[serde(default = "default_vsync")]
	vsync: bool,
	#[serde(default = "default_entity_shadows")]
	entity_shadows: bool,
	#[serde(default = "default_fullscreen")]
	fullscreen: bool,
	#[serde(default = "default_view_bobbing")]
	view_bobbing: bool,
	#[serde(default = "default_dark_mojang_background")]
	dark_mojang_background: bool,
	#[serde(default = "default_hide_lightning_flashes")]
	hide_lightning_flashes: bool,
	#[serde(default = "default_fov")]
	fov: u8,
	#[serde(default = "default_screen_effect_scale")]
	screen_effect_scale: f32,
	#[serde(default = "default_fov_effect_scale")]
	fov_effect_scale: f32,
	#[serde(default = "default_darkness_effect_scale")]
	darkness_effect_scale: f32,
	#[serde(default = "default_brightness")]
	brightness: f32,
	#[serde(default = "default_render_distance")]
	render_distance: u8,
	#[serde(default = "default_simulation_distance")]
	simulation_distance: u8,
	#[serde(default = "default_entity_distance_scaling")]
	entity_distance_scaling: f32,
	#[serde(default = "default_gui_scale")]
	gui_scale: u8,
	#[serde(default = "default_particles")]
	particles: OptionsEnum<ParticlesMode>,
	#[serde(default = "default_max_fps")]
	max_fps: u8,
	#[serde(default = "default_graphics_mode")]
	graphics_mode: OptionsEnum<GraphicsMode>,
	#[serde(default = "default_smooth_lighting")]
	smooth_lighting: bool,
	#[serde(default = "default_chunk_updates_mode")]
	chunk_updates_mode: OptionsEnum<ChunkUpdatesMode>,
	#[serde(default = "default_biome_blend")]
	biome_blend: u8,
	#[serde(default = "default_clouds")]
	clouds: CloudRenderMode,
	#[serde(default = "default_mipmap_levels")]
	mipmap_levels: u8,
	#[serde(default = "default_window_width")]
	window_width: u16,
	#[serde(default = "default_window_height")]
	window_height: u16,
	#[serde(default = "default_attack_indicator")]
	attack_indicator: OptionsEnum<AttackIndicatorMode>,
	#[serde(default = "default_fullscreen_resolution")]
	fullscreen_resolution: Option<FullscreenResolution>,
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
		}
	}
}

#[derive(Deserialize)]
pub struct VolumeOptions {
	#[serde(default = "default_sound_volume")]
	master: f32,
	#[serde(default = "default_sound_volume")]
	music: f32,
	#[serde(default = "default_sound_volume")]
	record: f32,
	#[serde(default = "default_sound_volume")]
	weather: f32,
	#[serde(default = "default_sound_volume")]
	block: f32,
	#[serde(default = "default_sound_volume")]
	hostile: f32,
	#[serde(default = "default_sound_volume")]
	neutral: f32,
	#[serde(default = "default_sound_volume")]
	player: f32,
	#[serde(default = "default_sound_volume")]
	ambient: f32,
	#[serde(default = "default_sound_volume")]
	voice: f32,
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

#[derive(Deserialize)]
pub struct SoundOptions {
	#[serde(default)]
	volume: VolumeOptions,
	#[serde(default = "default_show_subtitles")]
	show_subtitles: bool,
	#[serde(default = "default_directional_audio")]
	directional_audio: bool,
	#[serde(default = "default_sound_device")]
	device: Option<String>,
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

#[derive(Deserialize)]
pub struct SkinOptions {
	#[serde(default = "default_skin_part")]
	cape: bool,
	#[serde(default = "default_skin_part")]
	jacket: bool,
	#[serde(default = "default_skin_part")]
	left_sleeve: bool,
	#[serde(default = "default_skin_part")]
	right_sleeve: bool,
	#[serde(default = "default_skin_part")]
	left_pants: bool,
	#[serde(default = "default_skin_part")]
	right_pants: bool,
	#[serde(default = "default_skin_part")]
	hat: bool,
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

#[derive(Deserialize)]
pub struct ClientOptions {
	#[serde(default = "default_data_version")]
	data_version: i16,
	#[serde(default)]
	video: VideoOptions,
	#[serde(default)]
	control: ControlOptions,
	#[serde(default)]
	chat: ChatOptions,
	#[serde(default)]
	sound: SoundOptions,
	#[serde(default)]
	skin: SkinOptions,
	#[serde(default = "default_realms_notifications")]
	realms_notifications: bool,
	#[serde(default = "default_reduced_debug_info")]
	reduced_debug_info: bool,
	#[serde(default = "default_difficulty")]
	difficulty: OptionsEnum<Difficulty>,
	#[serde(default = "default_resource_packs")]
	resource_packs: Vec<String>,
	#[serde(default = "default_language")]
	language: String,
	#[serde(default = "default_tutorial_step")]
	tutorial_step: TutorialStep,
	#[serde(default = "default_skip_multiplayer_warning")]
	skip_multiplayer_warning: bool,
	#[serde(default = "default_skip_realms_32_bit_warning")]
	skip_realms_32_bit_warning: bool,
	#[serde(default = "default_hide_bundle_tutorial")]
	hide_bundle_tutorial: bool,
	#[serde(default = "default_joined_server")]
	joined_server: bool,
	#[serde(default = "default_sync_chunk_writes")]
	sync_chunk_writes: bool,
	#[serde(default = "default_use_native_transport")]
	use_native_transport: bool,
	#[serde(default = "default_held_item_tooltips")]
	held_item_tooltips: bool,
	#[serde(default = "default_advanced_item_tooltips")]
	advanced_item_tooltips: bool,
	#[serde(default = "default_log_level")]
	log_level: OptionsEnum<LogLevel>,
	#[serde(default = "default_hide_matched_names")]
	hide_matched_names: bool,
	#[serde(default = "default_pause_on_lost_focus")]
	pause_on_lost_focus: bool,
	#[serde(default = "default_main_hand")]
	main_hand: MainHand,
	#[serde(default = "default_hide_server_address")]
	hide_server_address: bool,
	#[serde(default = "default_show_autosave_indicator")]
	show_autosave_indicator: bool,
	#[serde(default = "default_allow_server_listing")]
	allow_server_listing: bool,
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

/// General options structure used to produce options for both client and server
#[derive(Deserialize)]
pub struct Options {
	#[serde(default)]
	client: ClientOptions,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphicsMode {
	Fast,
	Fancy,
	Fabulous,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticlesMode {
	All,
	Decreased,
	Minimal,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
	Peaceful,
	Easy,
	Normal,
	Hard,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkUpdatesMode {
	Threaded,
	SemiBlocking,
	FullyBlocking,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudRenderMode {
	Fancy,
	Off,
	Fast,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatVisibility {
	Shown,
	CommandsOnly,
	Hidden,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MainHand {
	Left,
	Right,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackIndicatorMode {
	Off,
	Crosshair,
	Hotbar,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NarratorMode {
	Off,
	All,
	Chat,
	System,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TutorialStep {
	Movement,
	FindTree,
	PunchTree,
	OpenInventory,
	CraftPlanks,
	None,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
	None,
	High,
	Medium,
	Low,
	Notification,
}

#[derive(Deserialize)]
pub struct FullscreenResolution {
	width: u32,
	height: u32,
	refresh_rate: u32,
	color_bits: u32,
}

/// Used for values that can be string representations or custom numbers
#[derive(Deserialize)]
#[serde(untagged)]
pub enum OptionsEnum<T> {
	Mode(T),
	Number(i32),
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
fn default_particles() -> OptionsEnum<ParticlesMode> { OptionsEnum::Mode(ParticlesMode::All) }
fn default_max_fps() -> u8 { 120 }
fn default_difficulty() -> OptionsEnum<Difficulty> { OptionsEnum::Mode(Difficulty::Normal) }
fn default_graphics_mode() -> OptionsEnum<GraphicsMode> { OptionsEnum::Mode(GraphicsMode::Fancy) }
fn default_smooth_lighting() -> bool { true }
fn default_chunk_updates_mode() -> OptionsEnum<ChunkUpdatesMode> { OptionsEnum::Mode(ChunkUpdatesMode::Threaded) }
fn default_biome_blend() -> u8 { 2 }
fn default_clouds() -> CloudRenderMode { CloudRenderMode::Fancy }
fn default_resource_packs() -> Vec<String> { vec![] }
fn default_language() -> String { String::from("en_us") }
fn default_sound_device() -> Option<String> { None }
fn default_chat_visibility() -> ChatVisibility { ChatVisibility::Shown }
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
fn default_attack_indicator() -> OptionsEnum<AttackIndicatorMode> { OptionsEnum::Mode(AttackIndicatorMode::Crosshair) }
fn default_narrator_mode() -> OptionsEnum<NarratorMode> { OptionsEnum::Mode(NarratorMode::Off) }
fn default_tutorial_step() -> TutorialStep { TutorialStep::None }
fn default_mouse_wheel_sensitivity() -> f32 { 1.0 }
fn default_raw_mouse_input() -> bool { true }
fn default_log_level() -> OptionsEnum<LogLevel> { OptionsEnum::Mode(LogLevel::High) }
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

pub fn parse_options(string: &str) -> anyhow::Result<Options> {
	serde_json::from_str(string).context("Failed to parse options")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default() {
		let options = parse_options("{}").unwrap();

		assert_eq!(options.client.data_version, default_data_version());
	}
}
