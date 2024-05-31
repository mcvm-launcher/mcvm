/// Downloading essential files for launching the game
pub mod game_files;
/// Downloading different flavors of the JRE
pub mod java;
/// Interacting with the Minecraft / Microsoft / Mojang APIs
pub mod minecraft;

// Re-export
pub use mcvm_net::download;
