import { invoke } from "@tauri-apps/api";

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
}
