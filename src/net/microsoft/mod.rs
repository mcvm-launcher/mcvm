pub mod auth;

use reqwest::Client;
use serde::Deserialize;

/// Struct for a Minecraft Profile from the Minecraft Services API
#[derive(Deserialize, Debug)]
pub struct MinecraftUserProfile {
	#[serde(rename = "id")]
	pub uuid: String,
	pub skins: Vec<Skin>,
	pub capes: Vec<Cape>,
}

/// A skin for a Minecraft user
#[derive(Deserialize, Debug)]
pub struct Skin {
	#[serde(flatten)]
	pub cosmetic: Cosmetic,
	pub variant: SkinVariant,
}

/// Variant for a skin
#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SkinVariant {
	Classic,
	Slim,
}

/// A cape for a Minecraft user
#[derive(Deserialize, Debug)]
pub struct Cape {
	#[serde(flatten)]
	pub cosmetic: Cosmetic,
	pub alias: String,
}

/// Common structure used for a user cosmetic
#[derive(Deserialize, Debug)]
pub struct Cosmetic {
	pub id: String,
	pub url: String,
	pub state: CosmeticState,
}

/// State for a cosmetic
#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CosmeticState {
	Active,
	Inactive,
}

/// Get a Minecraft user profile
pub async fn get_user_profile(
	access_token: &str,
	client: &Client,
) -> anyhow::Result<MinecraftUserProfile> {
	let response = client
		.get("https://api.minecraftservices.com/minecraft/profile")
		.header("Authorization", format!("Bearer {access_token}"))
		.send()
		.await?
		.error_for_status()?
		.json()
		.await?;

	Ok(response)
}
