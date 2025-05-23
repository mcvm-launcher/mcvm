// Adds an error shake animation
export function inputError(element: string, message?: string) {
	let elem = document.getElementById(element);
	if (elem != null) {
		elem.classList.remove("error-shake");
		// Trigger the animation to restart
		elem.offsetHeight;
		elem.classList.add("error-shake");
	}
}
