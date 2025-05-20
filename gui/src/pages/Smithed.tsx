export default function Smithed() {
	return (
		<div id="smithed">
			<iframe
				onLoad={(e) => {
					let target = e.target as unknown as HasContentWindow;
				}}
				src="https://smithed.net/"
				style="width: 100vw;height:100vh;border:none;margin-left:-0.47rem;margin-top:-0.47rem"
			></iframe>
		</div>
	);
}

interface HasContentWindow {
	contentWindow: Window;
}
