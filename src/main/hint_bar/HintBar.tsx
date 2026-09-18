import { memo } from "react";

import { IconArrowDown, IconArrowUp, IconCursor, IconEnter, IconShift } from "./hint_icon";
import "./HintBar.css";

/**
 * 提示条里的子组件
 *
 * 一个片段管一条提示，图标见 [`./hint_icon`](./hint_icon.tsx)；
 * 一个片段只做一件事，文案与数据都由骨架传进来，它们不认形态、也不拿主窗口状态。
 */

/** 「触发动作」那条提示的输入：只关心这一步有没有事可做，不关心动作叫什么 */
interface ActionHintProps {
    has_action: boolean,
}

/** 开关预览那条提示的输入：说的是这一次按下会做什么（打开还是关闭），不是当前状态名 */
interface PreviewToggleProps {
    label: string,
}

/** ESC 出口那条提示的输入：文案由用它的形态给——两种形态按下去的结果不一样 */
interface EscHintProps {
    label: string,
}

/** 「上下键 / 鼠标选中条目」：两种形态共用 */
const SelectionHint = () => {
    return <>
        <IconArrowUp></IconArrowUp>
        <IconArrowDown></IconArrowDown>
        <IconCursor></IconCursor>
        <span className="hint-text">选中条目</span>
    </>;
}

/**
 * 「回车触发当前动作」这条提示
 *
 * 只说按键与「会触发」这件事：具体动作名由 `ActionName` 在正中说，所以文案写死不跟着动作变。
 * 收的是「有没有动作」而不是动作名，所以换条目时只有有无动作翻转才会重新渲染它；
 * 没有可跑的动作时整块不渲染，与「Enter 对它确实什么都不做」一致。
 */
const ActionHint = ({ has_action }: ActionHintProps) => {
    if (!has_action) return <></>;

    return <>
        <IconEnter></IconEnter>
        <span className="hint-text">触发动作</span>
    </>;
}

/** ESC 没有通用的符号，用文字键帽代替（样式见 [HintBar.css](./HintBar.css) 的 .hint-key） */
const KeyEsc = () => {
    return (
        <span className="hint-key">ESC</span>
    );
}

/**
 * ESC 出口这条提示
 *
 * 两种形态都有（列表模式关界面、预览模式关预览），所以只说「按 ESC」，后果由形态给文案。
 */
const EscHint = ({ label }: EscHintProps) => {
    return <>
        <KeyEsc></KeyEsc>
        <span className="hint-text">{label}</span>
    </>;
}

/** 「Shift+Enter 开关预览」：两种形态共用，文案由骨架给 */
const PreviewToggleHint = ({ label }: PreviewToggleProps) => {
    return <>
        <span className="hint-combo">
            <IconShift></IconShift>
            <IconEnter></IconEnter>
        </span>
        <span className="hint-text">{label}</span>
    </>;
}

/**
 * 导出定义
 */

/** 正中动作名的输入：null 表示没有条目、或条目没有动作 */
interface ActionNameProps {
    action_desc: string | null,
}

/**
 * 正中的动作名：当前 `ItemType` + 动作解析出来的文案
 *
 * 绝对定位挂在 `.tail-box` 上（见 [HintBar.css](./HintBar.css) 的 `.hint-action-name`）：
 * 它不占流，左右两组的长度因此动不了它的居中。没有动作时什么都不显示。
 */
export const ActionName = ({ action_desc }: ActionNameProps) => {
    if (action_desc === null) return <></>;

    return (
        <span className="hint-action-name">{action_desc}</span>
    );
}

interface Props {
    /** ESC 出口的后果：列表模式是关界面，预览模式是关预览 */
    esc_label: string,
    /** 开关预览那一步做什么：打开还是关闭 */
    preview_toggle_label: string,
    /** 当前条目有没有可跑的动作：决定「触发动作」那条提示显不显示 */
    has_action: boolean,
}

/**
 * 提示条的骨架
 *
 * 两套形态的图标顺序完全一致，差别只在文案，所以骨架只有一份；文案由 Tail 按形态传进来
 * （见 `../Tail.tsx`），这里不认形态、也不拿主窗口状态。
 * 左组是退场与选中，右组是触发动作与开关预览。
 *
 * 动作名不在这里：它随 `Selection` 每次上下都变，交给外壳单独渲染（见 `../Tail.tsx`），
 * 这里收的三个输入只有形态或有无动作翻转时才变——配 `memo` 之后，切条目不会重渲染提示条。
 */
export const HintBar = memo(({ esc_label, preview_toggle_label, has_action }: Props) => {
    return <>
        <div className="hint-group">
            <EscHint label={esc_label}></EscHint>
            <SelectionHint></SelectionHint>
        </div>
        <div className="hint-group">
            <ActionHint has_action={has_action}></ActionHint>
            <PreviewToggleHint label={preview_toggle_label}></PreviewToggleHint>
        </div>
    </>;
});
