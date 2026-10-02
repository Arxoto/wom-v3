import { convertFileSrc } from "@tauri-apps/api/core";

import type { PluginItemDisplay } from "../core";
import { registry, type PluginIcon as PluginIconValue } from "./registry.tsx";

interface Props {
    icon: PluginIconValue | null,
    className: string,
}

const PluginIcon = ({ icon, className }: Props) => {
    if (icon === null) return <></>;

    if (typeof icon === "string") {
        return <img className={className} src={icon} alt="" draggable={false}></img>;
    }

    return <>{icon}</>;
};

export default PluginIcon;

export const item_icon = (item: PluginItemDisplay): PluginIconValue | null =>
    item.icon === "" ? registry.type_icon(item.the_type) : convertFileSrc(item.icon);
