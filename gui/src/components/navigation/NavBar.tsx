import "./NavBar.css";

export default function NavBar() {
	return (
		<>
			{/* Gap used to move page content down so that it starts below the navbar */}
			<div id="navbar-gap"></div>
			<div id="navbar" class="border">
				<div id="navbar-container">
					<div class="cont navbar-item"></div>
					<h2 class="cont navbar-item">
						<a href="/" class="link bold" title="Return to the homepage">
							MCVM
						</a>
					</h2>
					<div class="cont navbar-item" onclick={() => window.location.href = "/packages/0"}>Packages</div>
				</div>
			</div>
		</>
	);
}
