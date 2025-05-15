import { convertFileSrc } from "@tauri-apps/api/tauri";
import { InstanceIcon } from "./types";

export function getIconSrc(icon: InstanceIcon | null): string {
	if (icon === null) {
		return "icons/default_instance.png";
	} else {
		return convertFileSrc(icon);
	}
}

export function parseQueryString(string: string): QueryStringResult {
	if (!string.startsWith("?")) {
		return {};
	}

	string = string.substring(1);
	let entries = string.split("&");
	let out: QueryStringResult = {};
	for (let entry of entries) {
		let items = entry.split("=");
		if (items.length < 2) {
			continue;
		}
		let key = items[0];
		let value = items[1];
		out[key] = value;
	}

	return out;
}

export interface QueryStringResult {
	[key: string]: string | undefined;
}
