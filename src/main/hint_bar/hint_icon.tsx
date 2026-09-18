/**
 * 提示条的按键图标
 *
 * 只有提示条自己用的图标；列表行的动作图标属于 Item，见 [action_icons.tsx](../item/action_icons.tsx)——
 * 图标跟着它的消费者走，不集中到一个"全app图标库"里。
 * 图标只出几何——尺寸见 [HintBar.css](./HintBar.css) 的 `.hint-icon`，颜色一律 `currentColor` 跟随文字。
 */

/** 向上键 */
export const IconArrowUp = () => {
    return (
        <svg className="hint-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M8 13V3M3 8l5-5 5 5"></path>
        </svg>
    );
}

/** 向下键 */
export const IconArrowDown = () => {
    return (
        <svg className="hint-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M8 3v10M3 8l5 5 5-5"></path>
        </svg>
    );
}

/** 鼠标指针 */
export const IconCursor = () => {
    return (
        <svg className="hint-icon" viewBox="0 0 16 16" aria-hidden="true" fill="currentColor">
            <path d="M4 2v11.2l2.7-2.9 1.8 3.9 2-.9-1.8-3.8 4-.6z"></path>
        </svg>
    );
}

/** 回车键：一条折线加箭头，形状取自常见的 ↵ */
export const IconEnter = () => {
    return (
        <svg className="hint-icon hint-icon-enter" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12.5 3.5v5H3.5M6.5 5.5l-3 3 3 3"></path>
        </svg>
    );
}

/** Shift 键：空心上箭头 */
export const IconShift = () => {
    return (
        <svg className="hint-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinejoin="round">
            <path d="M8 2.5 14 9.5h-3.2v4h-5.6v-4H2z"></path>
        </svg>
    );
}
