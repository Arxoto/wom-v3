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
 * 高亮走到倒数第二行时列表整体上移一行、高亮停在同一屏幕行，
 * 下面确实没有更多结果时才继续下移到可见区最后一行（见 spec §5）。
 * 预览是否打开由外部传入（AppMain），这样它与 Tail 的提示是同一个状态。
 */
const Body = ({ item_list, selection, item_n, show_preview, type_actions, input_empty }: Props) => {
    const preview_item = item_list[selection];

    // 倒数第二行是滚动触发线；可见行数不足两行时没有这条线，退化成高亮到哪滚到哪
    const offset = Math.min(
        Math.max(selection - Math.max(item_n - 2, 0), 0),
        Math.max(item_list.length - item_n, 0),
    );

    return (
        <div className="body-box">
            <div className="body-items">
                {item_list.length === 0
                    ? <div className="body-empty">
                        {input_empty ? "输入关键字开始搜索" : "没有匹配的条目"}
                    </div>
                    // 整条轨道一起位移做滚动，可见区还是外面那层 overflow: hidden；
                    // 位移挂在行高变量上，px 不在这里算
                    : <div
                        className="body-items-track"
                        style={{ transform: `translateY(calc(var(--item-h) * ${-offset}))` }}>
                        {item_list.map((item, index) => (
                            // 用下标作 key：翻页时同一槽位的 DOM 保持复用（见 AppMain.tsx 的 todo）
                            <Item
                                key={index}
                                item={item}
                                action_id={default_action(type_actions, item.the_type)?.id ?? null}
                                is_selected={index === selection}>
                            </Item>
                        ))}
                    </div>}
            </div>
            {show_preview && preview_item ? <BodyPreview item={preview_item}></BodyPreview> : <></>}
        </div>
    );
}

export default Body;
