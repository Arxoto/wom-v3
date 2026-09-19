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
    /** 输入是不是空的：空输入不检索、列表已清空，占位文案换成提示输入 */
    input_empty: boolean,
}

/**
 * 列表主体：左侧条目列表，右侧预览
 *
 * 可见条数是配置里的 main_item_n；滚动偏移是 Selection 的派生值，不占状态：
 * 高亮走到可见区往下 60%（`item_n * 0.6` 取整）那一行时列表整体上移一行、高亮停在同一屏幕行，
 * 下面确实没有更多结果时才继续下移到可见区最后一行（见 spec §5）。
 * 已加载结果整份留在 `item_list` 里，但只有 `offset` 起的那 main_item_n 行进 DOM——
 * 翻页是往末尾追加，越翻越长，整份铺出来是白渲染，而滚动位置已经由 `offset` 表达完了。
 * 预览是否打开由外部传入（AppMain），这样它与 Tail 的提示是同一个状态。
 */
const Body = ({ item_list, selection, item_n, show_preview, type_actions, input_empty }: Props) => {
    const preview_item = item_list[selection];

    // 触发线：可见区往下 60% 那一行。行号得是整数（小数行没法拿去比高亮），
    // 也不能落在可见区之外——一行高的窗口 round 完是 1，夹回 0（那一档本来也无处可滚）
    const trigger = Math.min(Math.round(item_n * 0.6), item_n - 1);
    // 上下都夹住：触发线以上不滚（0），下面到头钉在最后一屏；已加载不足一屏时
    // 右端是负数，被 0 兜住，整份列表都露着
    const offset = Math.max(0, Math.min(selection - trigger, item_list.length - item_n));

    // 露在可见区里的那几行：窗口的第一行就是 offset，末尾不足一屏时自然短一截
    const item_show_list = item_list.slice(offset, offset + item_n);

    return (
        <div className="body-box">
            <div className="body-items">
                {item_list.length === 0
                    ? <div className="body-empty">
                        {input_empty ? "输入关键字开始搜索" : "没有匹配的条目"}
                    </div>
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
