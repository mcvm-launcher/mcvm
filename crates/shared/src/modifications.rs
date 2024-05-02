use std::fmt::Display;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A loader for Minecraft mods
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Modloader {
	/// No loader, just the default game
	#[default]
	Vanilla,
	/// MinecraftForge
	Forge,
	/// NeoForged
	NeoForged,
	/// Fabric Loader
	Fabric,
	/// Quilt Loader
	Quilt,
	/// LiteLoader
	LiteLoader,
	/// Risugami's Modloader
	Risugamis,
	/// Rift
	Rift,
	/// An unknown modloader
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl Display for Modloader {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Vanilla => write!(f, "None"),
			Self::Forge => write!(f, "Forge"),
			Self::NeoForged => write!(f, "NeoForged"),
			Self::Fabric => write!(f, "Fabric"),
			Self::Quilt => write!(f, "Quilt"),
			Self::LiteLoader => write!(f, "LiteLoader"),
			Self::Risugamis => write!(f, "Risugami's"),
			Self::Rift => write!(f, "Rift"),
			Self::Unknown(other) => write!(f, "{other}"),
		}
	}
}

/// Matcher for different types of loader
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ModloaderMatch {
	/// Matches vanilla
	Vanilla,
	/// Matches MinecraftForge
	Forge,
	/// Matches NeoForged
	NeoForged,
	/// Matches any loader that supports loading Forge mods
	ForgeLike,
	/// Matches Fabric Loader
	Fabric,
	/// Matches Quilt Loader
	Quilt,
	/// Matches any loader that supports loading Fabric mods
	FabricLike,
	/// Matches LiteLoader
	LiteLoader,
	/// Matches Risugami's Modloader
	Risugamis,
	/// Matches Rift
	Rift,
	/// An unknown modloader match
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl ModloaderMatch {
	/// Parse a ModloaderMatch from a string
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"forge" => Some(Self::Forge),
			"neoforged" => Some(Self::NeoForged),
			"forgelike" => Some(Self::ForgeLike),
			"fabric" => Some(Self::Fabric),
			"quilt" => Some(Self::Quilt),
			"fabriclike" => Some(Self::FabricLike),
			"liteloader" => Some(Self::LiteLoader),
			"risugamis" => Some(Self::Risugamis),
			"rift" => Some(Self::Rift),
			other => Some(Self::Unknown(other.into())),
		}
	}

	/// Checks if a modloader matches
	pub fn matches(&self, other: &Modloader) -> bool {
		match self {
			Self::Vanilla => matches!(other, Modloader::Vanilla),
			Self::Forge => matches!(other, Modloader::Forge),
			Self::NeoForged => matches!(other, Modloader::NeoForged),
			Self::ForgeLike => matches!(other, Modloader::Forge | Modloader::NeoForged),
			Self::Fabric => matches!(other, Modloader::Fabric),
			Self::Quilt => matches!(other, Modloader::Quilt),
			Self::FabricLike => matches!(other, Modloader::Fabric | Modloader::Quilt),
			Self::LiteLoader => matches!(other, Modloader::LiteLoader),
			Self::Risugamis => matches!(other, Modloader::Risugamis),
			Self::Rift => matches!(other, Modloader::Rift),
			Self::Unknown(this) => matches!(other, Modloader::Unknown(other) if this == other),
		}
	}
}

/// Different types of server changes. These are mostly mutually exclusive.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ServerType {
	/// Unspecified. Usually inherits from something else
	#[default]
	None,
	/// No modifications, just the default game
	Vanilla,
	/// Paper server
	Paper,
	/// SpongeVanilla
	Sponge,
	/// SpongeForge
	SpongeForge,
	/// CraftBukkit
	CraftBukkit,
	/// Spigot
	Spigot,
	/// Glowstone
	Glowstone,
	/// Pufferfish
	Pufferfish,
	/// Purpur
	Purpur,
	/// Folia
	Folia,
	/// MinecraftForge
	Forge,
	/// NeoForged
	NeoForged,
	/// Fabric Loader
	Fabric,
	/// Quilt Loader
	Quilt,
	/// Risugami's Modloader
	Risugamis,
	/// Rift
	Rift,
	/// An unknown server type
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl Display for ServerType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None => write!(f, "None"),
			Self::Vanilla => write!(f, "Vanilla"),
			Self::Paper => write!(f, "Paper"),
			Self::Sponge => write!(f, "Sponge"),
			Self::SpongeForge => write!(f, "SpongeForge"),
			Self::CraftBukkit => write!(f, "CraftBukkit"),
			Self::Spigot => write!(f, "Spigot"),
			Self::Glowstone => write!(f, "Glowstone"),
			Self::Pufferfish => write!(f, "Pufferfish"),
			Self::Purpur => write!(f, "Purpur"),
			Self::Folia => write!(f, "Folia"),
			Self::Forge => write!(f, "Forge"),
			Self::NeoForged => write!(f, "NeoForged"),
			Self::Fabric => write!(f, "Fabric"),
			Self::Quilt => write!(f, "Quilt"),
			Self::Risugamis => write!(f, "Risugami's"),
			Self::Rift => write!(f, "Rift"),
			Self::Unknown(other) => write!(f, "{other}"),
		}
	}
}

/// Matcher for different types of server plugin loaders
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum PluginLoaderMatch {
	/// The default game with no plugin support
	Vanilla,
	/// Matches any server that can load Bukkit plugins
	Bukkit,
	/// Matches Paper server
	Paper,
	/// Matches Sponge
	Sponge,
	/// Matches CraftBukkit
	CraftBukkit,
	/// Matches Spigot
	Spigot,
	/// Matches Glowstone
	Glowstone,
	/// Matches Pufferfish
	Pufferfish,
	/// Matches Purpur
	Purpur,
	/// Matches Folia
	Folia,
	/// An unknown plugin loader match
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl PluginLoaderMatch {
	/// Parse a PluginLoaderMatch from a string
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"bukkit" => Some(Self::Bukkit),
			"paper" => Some(Self::Paper),
			"sponge" => Some(Self::Sponge),
			"craftbukkit" => Some(Self::CraftBukkit),
			"spigot" => Some(Self::Spigot),
			"glowstone" => Some(Self::Glowstone),
			"pufferfish" => Some(Self::Pufferfish),
			"purpur" => Some(Self::Purpur),
			"folia" => Some(Self::Folia),
			other => Some(Self::Unknown(other.into())),
		}
	}

	/// Checks if a plugin loader matches
	pub fn matches(&self, other: &ServerType) -> bool {
		match self {
			Self::Vanilla => matches!(other, ServerType::Vanilla),
			Self::Bukkit => matches!(
				other,
				ServerType::Paper
					| ServerType::CraftBukkit
					| ServerType::Spigot | ServerType::Glowstone
					| ServerType::Pufferfish
					| ServerType::Purpur
			),
			Self::Paper => matches!(other, ServerType::Paper),
			Self::Sponge => matches!(other, ServerType::Sponge | ServerType::SpongeForge),
			Self::CraftBukkit => matches!(other, ServerType::CraftBukkit),
			Self::Spigot => matches!(other, ServerType::Spigot),
			Self::Glowstone => matches!(other, ServerType::Glowstone),
			Self::Pufferfish => matches!(other, ServerType::Pufferfish),
			Self::Purpur => matches!(other, ServerType::Purpur),
			Self::Folia => matches!(other, ServerType::Folia),
			Self::Unknown(this) => matches!(other, ServerType::Unknown(other) if this == other),
		}
	}
}

/// Different modifications for the client. Mostly mututally exclusive
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ClientType {
	/// Unspecified. Usually inherits from something else
	#[default]
	None,
	/// No modifications, just the default game
	Vanilla,
	/// MinecraftForge
	Forge,
	/// NeoForged
	NeoForged,
	/// Fabric Loader
	Fabric,
	/// Quilt Loader
	Quilt,
	/// LiteLoader
	LiteLoader,
	/// Risugami's Modloader
	Risugamis,
	/// Rift
	Rift,
	/// An unknown client type
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl Display for ClientType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None => write!(f, "None"),
			Self::Vanilla => write!(f, "Vanilla"),
			Self::Forge => write!(f, "Forge"),
			Self::NeoForged => write!(f, "NeoForged"),
			Self::Fabric => write!(f, "Fabric"),
			Self::Quilt => write!(f, "Quilt"),
			Self::LiteLoader => write!(f, "LiteLoader"),
			Self::Risugamis => write!(f, "Risugami's"),
			Self::Rift => write!(f, "Rift"),
			Self::Unknown(other) => write!(f, "{other}"),
		}
	}
}

/// Different proxies
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Proxy {
	/// No proxy
	#[default]
	None,
	/// The BungeeCord proxy
	BungeeCord,
	/// The Waterfall proxy
	Waterfall,
	/// The Velocity proxy
	Velocity,
	/// An unknown proxy
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl Display for Proxy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None => write!(f, "None"),
			Self::BungeeCord => write!(f, "BungeeCord"),
			Self::Waterfall => write!(f, "Waterfall"),
			Self::Velocity => write!(f, "Velocity"),
			Self::Unknown(other) => write!(f, "{other}"),
		}
	}
}

/// Matcher for different types of server proxies
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ProxyMatch {
	/// No proxy
	None,
	/// The BungeeCord proxy
	BungeeCord,
	/// The Waterfall proxy
	Waterfall,
	/// The Velocity proxy
	Velocity,
	/// Matches any proxy that can load BungeeCord plugins
	BungeeCordLike,
	/// An unknown proxy match
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl ProxyMatch {
	/// Parse a ProxyMatch from a string
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"none" => Some(Self::None),
			"bungeecord" => Some(Self::BungeeCord),
			"waterfall" => Some(Self::Waterfall),
			"velocity" => Some(Self::Velocity),
			"bungeecordlike" => Some(Self::BungeeCordLike),
			other => Some(Self::Unknown(other.into())),
		}
	}

	/// Checks if a proxy matches
	pub fn matches(&self, other: &Proxy) -> bool {
		match self {
			Self::None => matches!(other, Proxy::None),
			Self::BungeeCord => matches!(other, Proxy::BungeeCord),
			Self::Waterfall => matches!(other, Proxy::Waterfall),
			Self::Velocity => matches!(other, Proxy::Velocity),
			Self::BungeeCordLike => matches!(other, Proxy::BungeeCord | Proxy::Waterfall),
			Self::Unknown(this) => matches!(other, Proxy::Unknown(other) if this == other),
		}
	}
}
