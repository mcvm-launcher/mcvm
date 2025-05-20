import { AngleLeft, AngleRight, Logo, Menu } from "../../icons";
import IconButton from "../input/IconButton";
import "./NavBar.css";

export default function NavBar(props: NavBarProps) {
	return (
		<>
			{/* Gap used to move page content down so that it starts below the navbar */}
			<div id="navbar-gap"></div>
			<div id="navbar" class="border">
				<div id="navbar-container">
					<div class="cont navbar-item" id="navbar-left">
						<div id="sidebar-button">
							<IconButton
								icon={Menu}
								size="28px"
								color="var(--bg3)"
								selectedColor="var(--accent)"
								onClick={props.onSidebarToggle}
								selected={false}
							/>
						</div>
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
						<a href="/" class="cont link bold" title="Return to the homepage">
							<div style="margin-top:-0.45rem">
								<Logo width="25px" />
							</div>
							MCVM
						</a>
					</h2>
					<div class="cont navbar-item"></div>
					<div class="cont navbar-item"></div>
				</div>
			</div>
		</>
	);
}

export interface NavBarProps {
	onSidebarToggle: () => void;
}
