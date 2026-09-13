import type { ItemDisplay } from "../core";
import Item from "./item/Item";
import "./Body.css";

/**
 * 列表条数占位
 *
 * 条数就是 Config::main_item_n（默认 10，窗口高度也按它算好），
 * 接功能时换成 get_config().main_item_n，样式阶段先写常量。
 */
const ITEM_N = 10;

/**
 * 假数据：接功能前先用它看排版
 *
 * 名称与描述分短、中、超长三档，用来验证中间列的伸缩与渐变消失。
 */
const MOCK_ITEM_LIST: ItemDisplay[] = [
    { the_type: "sys", name: "Window Effect", desc: "切换窗口效果" },
    { the_type: "cmd", name: "Scan Base", desc: "扫描根路径" },
    { the_type: "note", name: "Item", desc: "一条可被检索到的内置内容" },
    { the_type: "web", name: "Config", desc: "配置只有一份，既是文件也是运行时依据" },
    { the_type: "file", name: "托盘", desc: "常驻系统托盘" },
    {
        the_type: "scan",
        name: "ItemDisplayWithAnAbsurdlyLongNameThatNeverEnds",
        desc: "超长英文名称，用来验证中间列的渐变消失",
    },
    {
        the_type: "snip",
        name: "全局快捷键",
        desc: "Ctrl+Space 唤出主面板 常驻托盘 内置插件 检索 分页 预览",
    },
    { the_type: "sys", name: "边缘发丝线", desc: "Edge Border" },
    {
        the_type: "cmd",
        name: "Window Effect Downgrade",
        desc: "选中的效果在当前平台不可用时替换成下一个支持的效果，保留用户的选择不动",
    },
    { the_type: "note", name: "短", desc: "短" },
];

interface BodyPreviewProps {
    item: ItemDisplay,
}

/**
 * 预览：当前条目的图标、名称与描述
 *
 * 选中条目的逻辑还没接，先用列表第一条占位。
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

const Body = () => {
    let show_preview = true;
    // show_preview = false;
    return (
        <div className="body-box">
            <div className="body-items">
                {MOCK_ITEM_LIST.slice(0, ITEM_N).map((item, index) => (
                    // 用下标作 key：翻页时同一槽位的 DOM 保持复用（见 AppMain.tsx 的 todo）
                    <Item key={index} item={item} action="action"></Item>
                ))}
            </div>
            {show_preview ? <BodyPreview item={MOCK_ITEM_LIST[8]}></BodyPreview> : <></>}
        </div>
    );
}

export default Body;
