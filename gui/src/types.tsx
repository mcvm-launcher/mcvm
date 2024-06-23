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
