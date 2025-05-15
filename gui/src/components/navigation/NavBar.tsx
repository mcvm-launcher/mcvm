import { AngleLeft, AngleRight } from "../../icons";
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
					<div
						class="cont navbar-item"
						onclick={() => (window.location.href = "/packages/0")}
					>
						Packages
					</div>
					<div class="cont navbar-item"></div>
				</div>
			</div>
		</>
	);
}
