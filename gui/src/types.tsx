export type Side = "client" | "server";

export interface InstanceInfo {
	id: string;
	name?: string;
	side: Side;
}
