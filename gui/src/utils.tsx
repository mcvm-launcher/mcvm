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

// Sort comparison function for strings
export function stringCompare(a: string, b: string) {
	return a > b ? 1 : a < b ? -1 : 0;
}

// Makes a string pretty by capitalizing first letters and replacing -,_, and . with spaces
export function beautifyString(string: string) {
	string = string.replace(/\./g, " ");
	string = string.replace(/\-/g, " ");
	string = string.replace(/\_/g, " ");

	// Capitalize
	let last = "";
	for (let i = 0; i < string.length; i++) {
		if (last == "" || last == " ") {
			let left = string.slice(0, i);
			let right = string.slice(i + 1);
			string = left.concat(string[i].toLocaleUpperCase()).concat(right);
		}
		last = string[i];
	}

	return string;
}
