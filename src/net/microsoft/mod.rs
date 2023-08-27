pub mod auth;

use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
	call_api(
		"https://api.minecraftservices.com/minecraft/profile",
		access_token,
		client,
	)
	.await
}

/// Response from the player certificate endpoint
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftUserCertificate {
	pub key_pair: Keypair,
}

/// Keypair in player certificate
#[derive(Deserialize, Serialize, Debug)]
pub struct Keypair {
	// Yes this is stupid
	#[serde(rename(deserialize = "privateKey"))]
	pub private_key: String,
	#[serde(rename(deserialize = "publicKey"))]
	pub public_key: String,
}

/// Get a Minecraft user certificate
pub async fn get_user_certificate(
	access_token: &str,
	client: &Client,
) -> anyhow::Result<MinecraftUserCertificate> {
	let response = client
		.post("https://api.minecraftservices.com/player/certificates")
		.header("Authorization", format!("Bearer {access_token}"))
		.send()
		.await?
		.error_for_status()?
		.json()
		.await?;

	Ok(response)
}

/// Utility function to query the Minecraft Services API with correct authorization
async fn call_api<T: DeserializeOwned>(
	url: &str,
	access_token: &str,
	client: &Client,
) -> anyhow::Result<T> {
	let response = client
		.get(url)
		.header("Authorization", format!("Bearer {access_token}"))
		.send()
		.await?
		.error_for_status()?
		.json()
		.await?;

	Ok(response)
}
