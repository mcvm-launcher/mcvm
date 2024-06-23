import "./NavBar.css";

export default function NavBar() {
	return (
		<>
			{/* Gap used to move page content down so that it starts below the navbar */}
			<div id="navbar-gap"></div>
			<div id="navbar" class="border">
				<h1>
					<a href="/" class="link bold" title="Return to the homepage">
						MCVM
					</a>
				</h1>
			</div>
		</>
	);
}
