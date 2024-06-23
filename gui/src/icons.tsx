import PinIcon from "./assets/icons/pin.svg?component-solid";
import BoxIcon from "./assets/icons/box.svg?component-solid";
import FolderIcon from "./assets/icons/folder.svg?component-solid";
import PlayIcon from "./assets/icons/play.svg?component-solid";
import PropertiesIcon from "./assets/icons/properties.svg?component-solid";
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
