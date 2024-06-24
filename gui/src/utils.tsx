import { convertFileSrc } from "@tauri-apps/api/tauri";
import { InstanceIcon } from "./types";

export function getIconSrc(icon: InstanceIcon | null): string {
	if (icon === null) {
		return "icons/default_instance.png";
	} else {
		return convertFileSrc(icon);
	}
}
