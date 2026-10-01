import { convertFileSrc } from "@tauri-apps/api/core";

import type { PluginItemDisplay } from "../core";
import { registry, type PluginIcon as PluginIconValue } from "./registry.tsx";

interface Props {
    icon: PluginIconValue | null,
    /** 尺寸与颜色由它定：条目图标与动作图标各有一份 css 类（见 Item.css） */
    className: string,
}

/**
 * 画一张插件图标
 *
 * 内置插件交上来的是 JSX 元素，原样渲染；JS 插件交上来的只能是一段字符串
 * （图片地址，data URI 也算），画成 `<img>`。分成两条路是因为 JS 插件手上没有 React
 * （见 `registry.tsx` 的 `PluginIcon`）。
 *
 * `null` 是"认不出这个类型 / 这个动作"：整块不画，而不是画一个占位。
 */
const PluginIcon = ({ icon, className }: Props) => {
    if (icon === null) return <></>;

    if (typeof icon === "string") {
        return <img className={className} src={icon} alt="" draggable={false}></img>;
    }

    return <>{icon}</>;
};

export default PluginIcon;

/**
 * 一行的图标
 *
 * 条目自带图片时（`Plugin Package` 清单里的那一张）用它，否则退回按类型名查的那一张。
 * 自带图片是**绝对路径**（Rust 拼好的），过一道 `convertFileSrc` 才是 webview 能取的
 * asset URL——路径能不能取由 asset protocol 的 scope 决定，这里不额外判断。
 */
export const item_icon = (item: PluginItemDisplay): PluginIconValue | null =>
    item.icon === "" ? registry.type_icon(item.the_type) : convertFileSrc(item.icon);
