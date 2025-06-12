export type Side = "client" | "server";
export type InstanceIcon = string;

export interface InstanceInfo {
	id: string;
	name: string | null;
	side: Side;
	icon: InstanceIcon | null;
	pinned: boolean;
}

export type InstanceMap = {
	[id: string]: InstanceInfo;
};

export interface GroupInfo {
	id: string;
	contents: string[];
}

export interface RunningInstanceInfo {
	info: InstanceInfo;
	state: RunState;
}

export type RunState = "not_started" | "preparing" | "running";

export interface UpdateRunStateEvent {
	instance: string;
	state: RunState;
}

export interface AuthDisplayEvent {
	url: string;
	device_code: string;
}

export interface PackageMeta {
	name?: string;
	description?: string;
	long_description?: string;
	banner?: string;
	icon?: string;
	gallery?: string[];
}

export interface PackageProperties {
	supported_versions?: string[];
	supported_modloaders?: string[];
	supported_plugin_loaders?: string[];
	supported_sides?: Side[];
}
