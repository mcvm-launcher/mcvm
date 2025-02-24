use mcvm_core::net::game_files::version_manifest::VersionEntry;
use mcvm_core::net::minecraft::MinecraftUserProfile;
use mcvm_core::util::versions::MinecraftVersionDeser;
use mcvm_pkg::script_eval::AddonInstructionData;
use mcvm_pkg::{RecommendedPackage, RequiredPackage};
use mcvm_shared::lang::translate::LanguageMap;
use mcvm_shared::modifications::{ClientType, ServerType};
use mcvm_shared::pkg::PackageID;
use mcvm_shared::{output::MCVMOutput, versions::VersionInfo, Side};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::hook_call::HookCallArg;
use crate::HookHandle;

/// Trait for a hook that can be called
pub trait Hook {
	/// The type for the argument that goes into the hook
	type Arg: Serialize + DeserializeOwned;
	/// The type for the result from the hook
	type Result: DeserializeOwned + Serialize + Default;

	/// Get the name of the hook
	fn get_name(&self) -> &'static str {
		Self::get_name_static()
	}

	/// Get the name of the hook statically
	fn get_name_static() -> &'static str;

	/// Get whether the hook should forward all output to the terminal
	fn get_takes_over() -> bool {
		false
	}

	/// Call the hook using the specified program
	fn call(
		&self,
		arg: HookCallArg<'_, Self>,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<HookHandle<Self>>
	where
		Self: Sized,
	{
		crate::hook_call::call(self, arg, o)
	}
}

macro_rules! def_hook {
	($struct:ident, $name:literal, $desc:literal, $arg:ty, $res:ty, $($extra:tt)*) => {
		#[doc = $desc]
		pub struct $struct;

		impl Hook for $struct {
			type Arg = $arg;
			type Result = $res;

			fn get_name_static() -> &'static str {
				$name
			}

			$(
				$extra
			)*
		}
	};
}

def_hook!(
	OnLoad,
	"on_load",
	"Hook for when a plugin is loaded",
	(),
	(),
);

def_hook!(
	Subcommand,
	"subcommand",
	"Hook for when a command's subcommands are run",
	Vec<String>,
	(),
	fn get_takes_over() -> bool {
		true
	}
);

def_hook!(
	ModifyInstanceConfig,
	"modify_instance_config",
	"Hook for modifying an instance's configuration",
	serde_json::Map<String, serde_json::Value>,
	ModifyInstanceConfigResult,
);

/// Result from the ModifyInstanceConfig hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ModifyInstanceConfigResult {
	/// Additional JVM args to add to the instance
	pub additional_jvm_args: Vec<String>,
}

def_hook!(
	AddVersions,
	"add_versions",
	"Hook for adding extra versions to the version manifest",
	(),
	Vec<VersionEntry>,
);

def_hook!(
	OnInstanceSetup,
	"on_instance_setup",
	"Hook for doing work when setting up an instance for update or launch",
	OnInstanceSetupArg,
	(),
);

/// Argument for the OnInstanceSetup hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OnInstanceSetupArg {
	/// The ID of the instance
	pub id: String,
	/// The side of the instance
	pub side: Option<Side>,
	/// Path to the instance's game dir
	pub game_dir: String,
	/// Version info for the instance
	pub version_info: VersionInfo,
	/// Custom config on the instance
	pub custom_config: serde_json::Map<String, serde_json::Value>,
}

def_hook!(
	OnInstanceLaunch,
	"on_instance_launch",
	"Hook for doing work before an instance is launched",
	InstanceLaunchArg,
	(),
);

def_hook!(
	WhileInstanceLaunch,
	"while_instance_launch",
	"Hook for running sibling processes with an instance when it is launched",
	InstanceLaunchArg,
	(),
);

def_hook!(
	OnInstanceStop,
	"on_instance_stop",
	"Hook for doing work when an instance is stopped gracefully",
	InstanceLaunchArg,
	(),
);

/// Argument for the OnInstanceLaunch and WhileInstanceLaunch hooks
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct InstanceLaunchArg {
	/// The ID of the instance
	pub id: String,
	/// The side of the instance
	pub side: Option<Side>,
	/// Path to the instance's dir
	pub dir: String,
	/// Path to the instance's game dir
	pub game_dir: String,
	/// Version info for the instance
	pub version_info: VersionInfo,
	/// Custom config on the instance
	pub custom_config: serde_json::Map<String, serde_json::Value>,
	/// The PID of the instance process
	pub pid: Option<u32>,
}

def_hook!(
	CustomPackageInstruction,
	"custom_package_instruction",
	"Hook for handling custom instructions in packages",
	CustomPackageInstructionArg,
	CustomPackageInstructionResult,
);

/// Argument for the CustomPackageInstruction hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CustomPackageInstructionArg {
	/// The ID of the package
	pub pkg_id: String,
}

/// Result from the CustomPackageInstruction hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CustomPackageInstructionResult {
	/// Whether the instruction was handled by this plugin
	pub handled: bool,
	/// The output of addon requests
	pub addon_reqs: Vec<AddonInstructionData>,
	/// The output dependencies
	pub deps: Vec<Vec<RequiredPackage>>,
	/// The output conflicts
	pub conflicts: Vec<PackageID>,
	/// The output recommendations
	pub recommendations: Vec<RecommendedPackage>,
	/// The output bundled packages
	pub bundled: Vec<PackageID>,
	/// The output compats
	pub compats: Vec<(PackageID, PackageID)>,
	/// The output package extensions
	pub extensions: Vec<PackageID>,
	/// The output notices
	pub notices: Vec<String>,
}

def_hook!(
	HandleAuth,
	"handle_auth",
	"Hook for handling authentication for custom user types",
	HandleAuthArg,
	HandleAuthResult,
);

/// Argument for the HandleAuth hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HandleAuthArg {
	/// The ID of the user
	pub user_id: String,
	/// The custom type of the user
	pub user_type: String,
}

/// Result from the HandleAuth hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HandleAuthResult {
	/// Whether the auth for this user type was handled by this plugin
	pub handled: bool,
	/// The resulting user profile
	pub profile: Option<MinecraftUserProfile>,
}

def_hook!(
	AddTranslations,
	"add_translations",
	"Hook for adding extra translations to MCVM",
	(),
	LanguageMap,
);

def_hook!(
	AddInstanceTransferFormats,
	"add_instance_transfer_formats",
	"Hook for adding information about instance transfer formats",
	(),
	Vec<InstanceTransferFormat>,
);

/// Information about an instance transfer format
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct InstanceTransferFormat {
	/// The ID for this format
	pub id: String,
	/// Info for the import side of this format
	pub import: Option<InstanceTransferFormatDirection>,
	/// Info for the export side of this format
	pub export: Option<InstanceTransferFormatDirection>,
}

/// Information about a side of an instance transfer format
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct InstanceTransferFormatDirection {
	/// Support status of the modloader
	pub modloader: InstanceTransferFeatureSupport,
	/// Support status of the mods
	pub mods: InstanceTransferFeatureSupport,
	/// Support status of the launch settings
	pub launch_settings: InstanceTransferFeatureSupport,
}

/// Support status of some feature in an instance transfer format
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum InstanceTransferFeatureSupport {
	/// This feature is supported by the transfer
	#[default]
	Supported,
	/// This feature is unsupported by the nature of the format
	FormatUnsupported,
	/// This feature is not yet supported by the plugin
	PluginUnsupported,
}

def_hook!(
	ExportInstance,
	"export_instance",
	"Hook for exporting an instance",
	ExportInstanceArg,
	(),
);

/// Argument provided to the export_instance hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ExportInstanceArg {
	/// The ID of the transfer format being used
	pub format: String,
	/// The ID of the instance
	pub id: String,
	/// The name of the instance
	pub name: Option<String>,
	/// The side of the instance
	pub side: Option<Side>,
	/// The directory where the instance game files are located
	pub game_dir: String,
	/// The desired path for the resulting instance, as a file path
	pub result_path: String,
	/// The Minecraft version of the instance
	pub minecraft_version: Option<MinecraftVersionDeser>,
	/// The client type of the new instance
	pub client_type: Option<ClientType>,
	/// The server type of the new instance
	pub server_type: Option<ServerType>,
}

def_hook!(
	ImportInstance,
	"import_instance",
	"Hook for importing an instance",
	ImportInstanceArg,
	ImportInstanceResult,
);

/// Argument provided to the import_instance hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ImportInstanceArg {
	/// The ID of the transfer format being used
	pub format: String,
	/// The ID of the new instance
	pub id: String,
	/// The path to the instance to import
	pub source_path: String,
	/// The desired directory for the resulting instance
	pub result_path: String,
}

/// Result from the ImportInstance hook giving information about the new instance
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ImportInstanceResult {
	/// The ID of the transfer format being used
	pub format: String,
	/// The name of the instance
	pub name: Option<String>,
	/// The side of the instance
	pub side: Option<Side>,
	/// The Minecraft version of the instance
	pub version: Option<MinecraftVersionDeser>,
	/// The client type of the new instance
	pub client_type: Option<ClientType>,
	/// The server type of the new instance
	pub server_type: Option<ServerType>,
}
