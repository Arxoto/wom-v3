import { memo } from "react";
import type { ItemActionId, ItemDisplay } from "../../core";
import { ACTION_ICONS, ACTION_SWITCH_MARKS } from "./action_icons";
import { ITEM_ICONS } from "./item_icons";
import "./Item.css";

interface Props {
    item: ItemDisplay,
    /** 该行显示的动作（每个条目自己记着，选中不改变谁来画）；条目没有动作时传 null，动作栏整块不渲染 */
    action_id: ItemActionId | null,
    /** 该条目这一侧还有别的动作可切时，动作图标旁的三角才露出来；没动作可切的侧只留白 */
    can_switch_prev: boolean,
    can_switch_next: boolean,
    is_selected: boolean,
}

/**
 * 一行条目：左图标、中间名称与描述、右侧动作图标
 *
 * 左侧是 ItemType 的线条图标（见 item_icons.tsx）。
 * 动作与 Item 数据无关（来自动作表），所以由外部传入；该行显示哪个动作也由外部决定。
 * is_selected 是键盘选中的那一行，与鼠标悬停共用同一套高亮样式（见 Item.css）。
 */
const Item = ({ item, action_id, can_switch_prev, can_switch_next, is_selected }: Props) => {
    return (
        <div className={is_selected ? "item-box is-selected" : "item-box"}>
            {ITEM_ICONS[item.the_type]}
            <div className="item-text">
                <div className="item-name">{item.name}</div>
                <div className="item-desc">{item.desc}</div>
            </div>
            {action_id !== null
                ? <div className="item-action"
                    data-switch-prev={can_switch_prev}
                    data-switch-next={can_switch_next}>
                    {ACTION_SWITCH_MARKS.prev}
                    {ACTION_ICONS[action_id]}
                    {ACTION_SWITCH_MARKS.next}
                </div>
                : <></>}
        </div>
    );
}

export default memo(Item);
