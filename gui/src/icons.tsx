import PinIcon from "./assets/icons/pin.svg?component-solid";
import BoxIcon from "./assets/icons/box.svg?component-solid";
import FolderIcon from "./assets/icons/folder.svg?component-solid";
import PlayIcon from "./assets/icons/play.svg?component-solid";
import PropertiesIcon from "./assets/icons/properties.svg?component-solid";
import CopyIcon from "./assets/icons/copy.svg?component-solid";
import CheckIcon from "./assets/icons/check.svg?component-solid";
import GlobeIcon from "./assets/icons/globe.svg?component-solid";
import CrossIcon from "./assets/icons/cross.svg?component-solid";
import SpinnerIcon from "./assets/icons/spinner.svg?component-solid";
import AngleLeftIcon from "./assets/icons/angle_left.svg?component-solid";
import { HasWidthHeight } from "./components/Icon";

export function Pin({ width, height, viewBox }: HasWidthHeight) {
	return <PinIcon width={width} height={height} viewBox={viewBox} />;
}

export function Box({ width, height, viewBox }: HasWidthHeight) {
	return <BoxIcon width={width} height={height} viewBox={viewBox} />;
}

export function Folder({ width, height, viewBox }: HasWidthHeight) {
	return <FolderIcon width={width} height={height} viewBox={viewBox} />;
}

export function Play({ width, height, viewBox }: HasWidthHeight) {
	return <PlayIcon width={width} height={height} viewBox={viewBox} />;
}

export function Properties({ width, height, viewBox }: HasWidthHeight) {
	return <PropertiesIcon width={width} height={height} viewBox={viewBox} />;
}

export function Copy({ width, height, viewBox }: HasWidthHeight) {
	return <CopyIcon width={width} height={height} viewBox={viewBox} />;
}

export function Check({ width, height, viewBox }: HasWidthHeight) {
	return <CheckIcon width={width} height={height} viewBox={viewBox} />;
}

export function Globe({ width, height, viewBox }: HasWidthHeight) {
	return <GlobeIcon width={width} height={height} viewBox={viewBox} />;
}

export function Cross({ width, height, viewBox }: HasWidthHeight) {
	return <CrossIcon width={width} height={height} viewBox={viewBox} />;
}

export function Spinner({ width, height, viewBox }: HasWidthHeight) {
	return <SpinnerIcon width={width} height={height} viewBox={viewBox} />;
}

export function AnimatedSpinner({ width, height, viewBox }: HasWidthHeight) {
	return (
		<div class="rotating">
			<SpinnerIcon width={width} height={height} viewBox={viewBox} />
		</div>
	);
}

export function AngleLeft({ width, height, viewBox }: HasWidthHeight) {
	return <AngleLeftIcon width={width} height={height} viewBox={viewBox} />;
}
