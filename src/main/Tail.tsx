import { ActionName, HintBar } from "./hint_bar/HintBar";
import "./Tail.css";

interface Props {
    /** 预览是否打开：提示条的形态由它现算，不另存一份状态 */
    preview_open: boolean,
    /** 当前条目的当前动作文案；没有条目或条目没有动作时为 null，动作栏整块不渲染 */
    action_desc: string | null,
}

/**
 * 底部区块的外壳：与 Head / Body 同层，只管布局（见 Tail.css）
 *
 * 提示条的骨架只有一份（见 `hint_bar/HintBar.tsx`），形态带来的差异只有文案，
 * 所以这里把形态翻成文案再交出去；图标与顺序都不经过这里。
 *
 * 动作名也在这里渲染、不进提示条：它随 `Selection` 每次上下都变，放在外壳这一层，
 * 提示条就不跟着一起重渲染（见 HintBar 的 memo）。它绝对定位在正中，DOM 里在哪一段无所谓。
 */
const Tail = ({ preview_open, action_desc }: Props) => {
    const texts = HINT_BAR_TEXTS[preview_open ? HintBarKind.Preview : HintBarKind.ItemList];

    return (
        <div className="tail-box">
            <HintBar
                esc_label={texts.esc_label}
                preview_toggle_label={texts.preview_toggle_label}
                has_action={action_desc !== null}>
            </HintBar>
            <ActionName action_desc={action_desc}></ActionName>
        </div>
    );
}

/** 提示条的形态：不挑组件（骨架只有一份），只挑一套说辞 */
enum HintBarKind {
    ItemList = "ItemList",
    Preview = "Preview",
}

/** 形态的说辞 */
interface HintBarTexts {
    esc_label: string,
    preview_toggle_label: string,
}

/** 表格写全所有形态，枚举加值时这里会编译不过 */
const HINT_BAR_TEXTS: Record<HintBarKind, HintBarTexts> = {
    [HintBarKind.ItemList]: {
        esc_label: "关闭界面",
        preview_toggle_label: "打开预览",
    },
    [HintBarKind.Preview]: {
        esc_label: "关闭预览",
        preview_toggle_label: "关闭预览",
    },
}

export default Tail;
