import { memo, useState } from "react";
import type { PluginItemDisplay } from "../core";
import Item from "./item/Item";
import PluginIcon, { item_icon } from "../plugins/plugin_icon";
import { registry } from "../plugins/registry.tsx";
import { current_item } from "./interaction/reducer";
import "./Body.css";

interface BodyPreviewProps {
    item: PluginItemDisplay,
}

/**
 * 预览：当前条目的图标、名称与描述
 */
const BodyPreview = ({ item }: BodyPreviewProps) => {
    return <>
        <div className="body-divider"></div>
        <div className="body-preview">
            <div className="body-preview-icon">
                <div className="body-preview-icon-block">
                    <PluginIcon icon={item_icon(item)} className="item-icon"></PluginIcon>
                </div>
            </div>
            <div className="body-preview-title">{item.name}</div>
            <div className="body-preview-divider"></div>
            <div className="body-preview-details">{item.desc}</div>
        </div>
    </>;
}

interface Props {
    item_list: PluginItemDisplay[],
    selection: number,
    item_n: number,
    show_preview: boolean,
    /** 每个条目记住的动作下标（按 Item Index）：行内动作与两侧三角都按它画 */
    action_indices: Record<number, number>,
    /** 空态：还没有结论（没输入过、输入为空，或查询还没回来）——列表区留白 */
    empty: boolean,
    /** 插件搜索页是否打开：那一页为空时也留白（这一段没有渲染"为什么"的地方，spec §4.4） */
    search_open: boolean,
}

type Layer = "main" | "search";

interface LayerScroll {
    selection: number,
    offset: number,
}

/**
 * 列表主体：左侧条目列表，右侧预览
 *
 * 可见条数是配置里的 main_item_n；滚动偏移是这里自己的状态，不是 Selection 的纯派生值。
 * 
 * 已加载结果整份留在 `item_list` 里，但只有 `offset` 起的那 main_item_n 行进 DOM。
 * 
 * 预览是否打开由外部传入（AppMain），这样它与 Tail 的提示是同一个状态。
 */
const Body = ({ item_list, selection, item_n, show_preview, action_indices, empty, search_open }: Props) => {
    const preview_item = current_item(item_list, selection);

    const empty_text = empty || search_open ? "" : "没有匹配的条目";

    /**
     * 窗口的滚动位置。Selection 一起存着，只在它真的变了之后才看窗口要不要挪。
     *
     * `which` 是"这一份列表是哪一层"（主列表 / 某个插件的搜索页）：两层各记一份，
     * 换层时另一层原样留着——从结果页回到主列表，看到的是离开前那一屏。
     */
    const which: Layer = search_open ? "search" : "main";
    const [scrolls, set_scrolls] = useState<Record<Layer, LayerScroll>>({
        main: { selection, offset: 0 },
        search: { selection: 0, offset: 0 },
    });

    const layer = scrolls[which];
    let offset = layer.offset;

    if (layer.selection !== selection) {
        // 高亮上下各留的余量（行数），近似黄金分割
        const margin = Math.floor(item_n * 0.4);
        // 高亮现在落在屏幕第几行
        const row_index = selection - offset;

        if (row_index < margin - 1) offset = selection - margin + 1;
        else if (row_index > item_n - margin) offset = selection + margin - item_n;
        // 两端夹住
        offset = Math.max(0, Math.min(offset, item_list.length - item_n));
        set_scrolls({ ...scrolls, [which]: { selection, offset } });
    }

    // 露在可见区里的那几行：窗口的第一行就是 offset，末尾不足一屏时自然短一截
    const item_show_list = item_list.slice(offset, offset + item_n);

    return (
        <div className="body-box">
            <div className="body-items">
                {item_list.length === 0
                    ? <div className="body-empty">{empty_text}</div>
                    : item_show_list.map((item, index) => {
                        const is_selected = offset + index === selection;
                        const actions = registry.actions_of(item);
                        const action_index = action_indices[item.item_index] ?? 0;
                        const action = registry.current_action(item, action_index);
                        return (
                            // 用窗口内的下标作 key：翻页时同一槽位的 DOM 保持复用
                            <Item
                                key={index}
                                item={item}
                                action_id={action?.id ?? null}
                                can_switch_prev={action_index > 0}
                                can_switch_next={action_index < actions.length - 1}
                                is_selected={is_selected}>
                            </Item>
                        );
                    })}
            </div>
            {show_preview && preview_item ? <BodyPreview item={preview_item}></BodyPreview> : <></>}
        </div>
    );
}

export default memo(Body);
