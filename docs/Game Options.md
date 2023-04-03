# Game Options

mcvm is capable of applying global options for both client and server
that are inherited across all of your profiles. The options are agnostic
to the Minecraft version and automatically converted to the correct format. Options are supplied in a JSON
format in the file `options.json` in your config directory. This file may not exist so you might
have to create it yourself.

Format:
```json
{
	"client": {
		...
	},
	"server": {
		...
	}
}
```

A description will not be provided for every option as they mirror the options in-game and inside the server.properties and should be somewhat self-explanatory. Any options that need an explanation will have a note.

## Client

```json
{
	"data_version": integer,
	"video": {
		"vsync": bool,
		"entity_shadows": bool,
		"fullscreen": bool,
		"view_bobbing": bool,
		"dark_mojang_background": bool,
		"hide_lightning_flashes": bool,
		"fov": integer,
		"screen_effect_scale": number,
		"fov_effect_scale": number,
		"darkness_effect_scale": number,
		"brightness": number,
		"render_distance": integer,
		"simulation_distance": integer,
		"entity_distance_scaling": number,
		"gui_scale": number,
		"particles": "all" | "decreased" | "minimal" | integer,
		"max_fps": number,
		"graphics_mode": "fast" | "fancy" | "fabulous" | integer,
		"smooth_lighting": bool,
		"chunk_updates_mode": "threaded" | "semi_blocking" | "fully_blocking" | integer,
		"biome_blend": integer,
		"clouds": "fancy" | "fast" | "off" | integer,
		"mipmap_levels": integer,
		"window_width": integer,
		"window_height": integer,
		"attack_indicator": "off" | "crosshair" | "hotbar" | integer,
		"fullscreen_resolution"?: {
			"width": integer,
			"height": integer,
			"refresh_rate": integer,
			"color_bits": integer
		},
		"allow_block_alternatives": bool
	},
	"control": {
		"keys": {
			"attack": string,
			"use": string,
			"forward": string,
			"left": string,
			"back": string,
			"right": string,
			"jump": string,
			"sneak": string,
			"sprint": string,
			"drop": string,
			"inventory": string,
			"chat": string,
			"playerlist": string,
			"pick_item": string,
			"command": string,
			"social_interactions": string,
			"screenshot": string,
			"toggle_perspective": string,
			"smooth_camera": string,
			"fullscreen": string,
			"spectator_outlines": string,
			"swap_offhand": string,
			"save_toolbar": string,
			"load_toolbar": string,
			"advancements": string,
			"hotbar_1": string,
			"hotbar_2": string,
			"hotbar_3": string,
			"hotbar_4": string,
			"hotbar_5": string,
			"hotbar_6": string,
			"hotbar_7": string,
			"hotbar_8": string,
			"hotbar_9": string
		},
		"auto_jump": bool,
		"invert_mouse_y": bool,
		"enable_touchscreen": bool,
		"toggle_sprint": bool,
		"toggle_crouch": bool,
		"mouse_sensitivity": number,
		"mouse_wheel_sensitivity": number,
		"raw_mouse_input": bool
	},
	"chat": {
		"auto_command_suggestions": bool,
		"enable_colors": bool,
		"enable_links": bool,
		"prompt_links": bool,
		"force_unicode": bool,
		"visibility": "shown" | "commands_only" | "hidden" | integer,
		"opacity": number,
		"line_spacing": number,
		"background_opacity": number,
		"background_for_chat_only": bool,
		"focused_height": number,
		"unfocused_height": number,
		"delay": number,
		"scale": number,
		"width": number,
		"narrator_mode": "off" | "all" | "chat" | "system" | integer
	},
	"sound": {
		"volume": {
			"master": number,
			"music": number,
			"record": number,
			"weather": number,
			"block": number,
			"hostile": number,
			"neutral": number,
			"player": number,
			"ambient": number,
			"voice": number
		},
		"show_subtitles": bool,
		"directional_audio": bool,
		"device"?: string
	},
	"skin": {
		"cape": bool,
		"jacket": bool,
		"left_sleeve": bool,
		"right_sleeve": bool,
		"left_pants": bool,
		"right_pants": bool,
		"hat": bool
	},
	"custom": {
		...
	},
	"realms_notifications": bool,
	"reduced_debug_info": bool,
	"difficulty": "peaceful" | "easy" | "normal" | "hard" | integer,
	"resource_packs": [string],
	"language": string,
	"tutorial_step": "movement" | "find_tree" | "punch_tree" | "open_inventory" | "craft_planks" | "none",
	"skip_multiplayer_warning": bool,
	"skip_realms_32_bit_warning": bool,
	"hide_bundle_tutorial": bool,
	"joined_server": bool,
	"sync_chunk_writes": bool,
	"use_native_transport": bool,
	"held_item_tooltips": bool,
	"advanced_item_tooltips": bool,
	"log_level": "none" | "high" | "medium" | "low" | "notification" | integer,
	"hide_matched_names": bool,
	"pause_on_lost_focus": bool,
	"main_hand": "left" | "right",
	"hide_server_address": bool,
	"show_autosave_indicator": bool,
	"allow_server_listing": bool
}
```

## Server

```json
{
	"rcon": {
		"enable": bool,
		"port": integer,
		"password"?: string
	},
	"query": {
		"enable": bool,
		"port": integer
	},
	"whitelist": {
		"enable": bool,
		"enforce": bool
	},
	"gamemode": {
		"default": "survival" | "creative" | "adventure" | "spectator" | integer,
		"force": bool
	},
	"datapacks": {
		"function_permission_level": integer,
		"initial_enabled": [String],
		"initial_disabled": [String]
	},
	"world": {
		"name": string,
		"seed"?: string,
		"type": "normal" | "flat" | "large_biomes" | "amplified" | "single_biome" | "buffet" | "custom" | string,
		"structures": bool,
		"generator_settings": {},
		"max_size": integer,
		"max_build_height": integer,
		"allow_nether": bool
	},
	"resource_pack": {
		"uri"?: String,
		"prompt"?: String,
		"sha1"?: String,
		"required": bool
	},
	"allow_flight": bool,
	"broadcast_console_to_ops": bool,
	"broadcast_rcon_to_ops": bool,
	"difficulty": "peaceful" | "easy" | "normal" | "hard" | integer,
	"allow_command_blocks": bool,
	"jmx_monitoring": bool,
	"enable_status": bool,
	"enforce_secure_profile": bool,
	"entity_broadcast_range": integer,
	"max_chained_neighbor_updates": integer,
	"max_players": integer,
	"max_tick_time": integer,
	"motd": string,
	"network_compression_threshold": number | "disabled" | "all",
	"offline_mode": bool,
	"op_permission_level": integer,
	"player_idle_timeout": integer,
	"prevent_proxy_connections": bool,
	"enable_chat_preview": bool,
	"enable_pvp": bool,
	"rate_limit": integer,
	"ip"?: string,
	"port": integer,
	"simulation_distance": integer,
	"enable_snooper": bool,
	"spawn_animals": bool,
	"spawn_monsters": bool,
	"spawn_npcs": bool,
	"spawn_protection": integer,
	"sync_chunk_writes": bool,
	"use_native_transport": bool,
	"view_distance": integer
}
```

### Notes:
1. `offline_mode` is the opposite of the usual server.properties option `online_mode`
