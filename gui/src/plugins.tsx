import { invoke } from "@tauri-apps/api";
import { WebviewWindow } from "@tauri-apps/api/window";

// Loads plugins on a page
export function loadPagePlugins(page: string, object?: string) {
	invoke("get_page_inject_script", { page: page, object: object }).then(
		(script) => {
			let script2 = script as string;
			eval(script2);
			console.log("Page plugins loaded successfully");
		},
		(e) => {
			console.error("Failed to load page plugins: " + e);
		}
	);
	setupPluginFunctions();
}

export function setupPluginFunctions() {
	let global = window as any;
	global.tauriInvoke = async (command: any, args: any) => {
		await invoke(command, args);
	};
	global.TauriWindow = WebviewWindow;
}
