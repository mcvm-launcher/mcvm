import { createEffect, JSX, Show } from "solid-js";
import "./Sidebar.css";
import { Box, Home, Jigsaw, Menu } from "../../icons";
import { Location } from "@solidjs/router";

export default function Sidebar(props: SidebarProps) {
	// Close the sidebar when clicking outside of it
	createEffect(() => {
		document.addEventListener("click", (e) => {
			let sidebar = document.getElementById("sidebar");
			let sidebarButton = document.getElementById("sidebar-button");
			// Walk up the tree
			let target = e.target as Element;
			while (target != null && target != sidebar && target != sidebarButton) {
				target = target.parentNode as Element;
			}

			if (target == null) {
				if (props.visible) {
					props.setVisible(false);
				}
			}
		});
	});

	return (
		<Show when={props.visible}>
			<div id="sidebar">
				<div id="sidebar-items">
					<SidebarItem
						href="/"
						location={props.location}
						selectedPath="/"
						color="var(--fg3)"
						closeSidebar={() => props.setVisible(false)}
					>
						<div style="margin-top:0.15rem;margin-right:-0.2rem;color:var(--fg2)">
							<Home />
						</div>
						<div>Home</div>
					</SidebarItem>
					<SidebarItem
						href="/packages/0"
						location={props.location}
						selectedPathStart="/packages"
						color="var(--package)"
						closeSidebar={() => props.setVisible(false)}
					>
						<div style="margin-top:0.3rem;margin-right:-0.2rem;color:var(--package)">
							<Box />
						</div>
						<div>Packages</div>
					</SidebarItem>
					<SidebarItem
						href="/plugins"
						location={props.location}
						selectedPathStart="/plugins"
						color="var(--plugin)"
						closeSidebar={() => props.setVisible(false)}
					>
						<div style="margin-top:0.1rem;margin-right:-0.2rem;color:var(--plugin)">
							<Jigsaw />
						</div>
						<div>Plugins</div>
					</SidebarItem>
					<SidebarItem
						href="/docs"
						location={props.location}
						selectedPathStart="/docs"
						color="var(--profile)"
						closeSidebar={() => props.setVisible(false)}
					>
						<div style="margin-top:0.3rem;margin-right:-0.2rem;color:var(--profile)">
							<Menu />
						</div>
						<div>Documentation</div>
					</SidebarItem>
					<SidebarItem
						href="/smithed"
						location={props.location}
						selectedPathStart="/smithed"
						color="#1b48c4"
						closeSidebar={() => props.setVisible(false)}
					>
						<div style="margin-top:0.2rem;margin-right:-0.2rem;color:var(--plugin)">
							<img src="/smithed.png" width="16px" style="width: 16px" />
						</div>
						<div>Smithed</div>
					</SidebarItem>
				</div>
			</div>
		</Show>
	);
}

export interface SidebarProps {
	visible: boolean;
	setVisible: (visible: boolean) => void;
	location: Location;
}

function SidebarItem(props: SidebarItemProps) {
	const selected = () => {
		if (props.selectedPath != undefined) {
			return props.location.pathname == props.selectedPath;
		}
		if (props.selectedPathStart != undefined) {
			return props.location.pathname.startsWith(props.selectedPathStart);
		}

		return false;
	};
	return (
		<a
			class={`cont link sidebar-item ${selected() ? "selected" : ""}`}
			href={props.href}
			style={`border-right-color:${props.color}`}
			onclick={() => props.closeSidebar()}
		>
			{props.children}
		</a>
	);
}

interface SidebarItemProps {
	children: JSX.Element;
	href: string;
	location: Location;
	// What the current URL should equal to select this item
	selectedPath?: string;
	// What the current URL should start with to select this item
	selectedPathStart?: string;
	color: string;
	closeSidebar: () => void;
}
