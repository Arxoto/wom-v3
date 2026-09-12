import type { ItemDisplay } from "../../core";
import "./Item.css";

interface Props {
    item: ItemDisplay,
    action: string,
}

/**
 * 一行条目：左图标、中间名称与描述、右侧动作名称
 *
 * 图标先用纯色块占位，后期换成 svg / ico。
 * 动作名称与 Item 数据无关（来自动作系统），所以由外部传入，当前传占位值。
 */
const Item = ({ item, action }: Props) => {
    return (
        <div className="item-box">
            <div className="item-icon"></div>
            <div className="item-text">
                <div className="item-name">{item.name}</div>
                <div className="item-desc">{item.desc}</div>
            </div>
            <div className="item-action">{action}</div>
        </div>
    );
}

export default Item;
