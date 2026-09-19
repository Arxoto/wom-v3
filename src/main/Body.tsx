import { useState } from "react";
import type { ItemDisplay, ItemTypeActions } from "../core";
import Item from "./item/Item";
import { default_action } from "./interaction/action_labels";
import "./Body.css";

interface BodyPreviewProps {
    item: ItemDisplay,
}

/**
 * 预览：当前条目的图标、名称与描述
 */
const BodyPreview = ({ item }: BodyPreviewProps) => {
    return <>
        <div className="body-divider"></div>
        <div className="body-preview">
            <div className="body-preview-icon">
                <div className="body-preview-icon-block"></div>
            </div>
            <div className="body-preview-title">{item.name}</div>
            <div className="body-preview-divider"></div>
            <div className="body-preview-details">{item.desc}</div>
        </div>
    </>;
}

interface Props {
    item_list: ItemDisplay[],
    selection: number,
    item_n: number,
    show_preview: boolean,
    type_actions: ItemTypeActions,
    /** 空态：还没输入，或结果还在路上——列表区留白 */
    empty: boolean,
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
const Body = ({ item_list, selection, item_n, show_preview, type_actions, empty }: Props) => {
    const preview_item = item_list[selection];

    const empty_text = empty ? "" : "没有匹配的条目";

    // 窗口的滚动位置。Selection 一起存着，只在它真的变了之后才看窗口要不要挪
    const [scroll, set_scroll] = useState({ selection, offset: 0 });

    let offset = scroll.offset;
    if (scroll.selection !== selection) {
        // 高亮上下各留的余量（行数），近似黄金分割
        const margin = Math.floor(item_n * 0.4);
        // 高亮现在落在屏幕第几行
        const row_index = selection - offset;

        if (row_index < margin - 1) offset = selection - margin + 1;
        else if (row_index > item_n - margin) offset = selection + margin - item_n;
        // 两端夹住
        offset = Math.max(0, Math.min(offset, item_list.length - item_n));
        set_scroll({ selection, offset });
    }

    // 露在可见区里的那几行：窗口的第一行就是 offset，末尾不足一屏时自然短一截
    const item_show_list = item_list.slice(offset, offset + item_n);

    return (
        <div className="body-box">
            <div className="body-items">
                {item_list.length === 0
                    ? <div className="body-empty">{empty_text}</div>
                    : item_show_list.map((item, index) => (
                        // 用窗口内的下标作 key：翻页时同一槽位的 DOM 保持复用（见 AppMain.tsx 的 todo）
                        <Item
                            key={index}
                            item={item}
                            action_id={default_action(type_actions, item.the_type)?.id ?? null}
                            is_selected={offset + index === selection}>
                        </Item>
                    ))}
            </div>
            {show_preview && preview_item ? <BodyPreview item={preview_item}></BodyPreview> : <></>}
        </div>
    );
}

export default Body;
