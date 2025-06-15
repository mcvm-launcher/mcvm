import { For } from "solid-js";
import {
	PackageCategory,
	packageCategoryDisplayName,
	packageCategoryIcon,
} from "../../package";
import Icon from "../Icon";
import "./PackageLabels.css";

export default function PackageLabels(props: PackageLabelsProps) {
	let small = props.small == undefined ? false : props.small;

	return (
		<div class={`cont package-labels ${small ? "small" : ""}`}>
			<For each={props.categories}>
				{(category, i) => {
					if (props.limit != undefined && i() >= props.limit) {
						return undefined;
					} else {
						return (
							<div class={`cont package-category ${small ? "small" : ""}`}>
								<div class="cont package-category-icon">
									<Icon icon={packageCategoryIcon(category)} size="1rem" />
								</div>
								<div class="cont package-category-label">
									{packageCategoryDisplayName(category)}
								</div>
							</div>
						);
					}
				}}
			</For>
		</div>
	);
}

export interface PackageLabelsProps {
	categories: PackageCategory[];
	// The maximum number of labels to include
	limit?: number;
	small?: boolean;
}
