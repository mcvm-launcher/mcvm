import { AngleLeft, AngleRight, Box } from "../../icons";
import IconButton from "../input/IconButton";
import "./NavBar.css";

export default function NavBar() {
	return (
		<>
			{/* Gap used to move page content down so that it starts below the navbar */}
			<div id="navbar-gap"></div>
			<div id="navbar" class="border">
				<div id="navbar-container">
					<div class="cont navbar-item">
						<IconButton
							icon={AngleLeft}
							size="28px"
							color="var(--bg3)"
							selectedColor="var(--accent)"
							onClick={() => {
								history.back();
							}}
							selected={false}
						/>
						<IconButton
							icon={AngleRight}
							size="28px"
							color="var(--bg3)"
							selectedColor="var(--accent)"
							onClick={() => {
								history.forward();
							}}
							selected={false}
						/>
					</div>
					<div class="cont navbar-item"></div>
					<h2 class="cont navbar-item">
						<a href="/" class="link bold" title="Return to the homepage">
							MCVM
						</a>
					</h2>
					<a class="cont link navbar-item" href="/packages/0">
						<div style="margin-top:0.3rem;margin-right:-0.2rem;color:var(--package)">
							<Box />
						</div>
						<div>Packages</div>
					</a>
					<div class="cont navbar-item"></div>
				</div>
			</div>
		</>
	);
}
