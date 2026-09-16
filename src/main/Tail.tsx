import "./Tail.css";

/**
 * 底部提示条显示哪一套提示
 *
 * ItemList：预览没打开，上下键与鼠标选中条目，回车触发动作，shift+回车打开预览
 * Preview：预览打开，ESC 与 shift+回车都能关掉预览
 */
export enum TailHint {
    ItemList = "ItemList",
    Preview = "Preview",
}

/* 图标都是装饰性的，尺寸交给 css 的 1em（跟着 --tail-h 缩放），颜色用 currentColor 跟随文字 */

const IconArrowUp = () => {
    return (
        <svg className="tail-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M8 13V3M3 8l5-5 5 5"></path>
        </svg>
    );
}

const IconArrowDown = () => {
    return (
        <svg className="tail-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M8 3v10M3 8l5 5 5-5"></path>
        </svg>
    );
}

const IconCursor = () => {
    return (
        <svg className="tail-icon" viewBox="0 0 16 16" aria-hidden="true" fill="currentColor">
            <path d="M4 2v11.2l2.7-2.9 1.8 3.9 2-.9-1.8-3.8 4-.6z"></path>
        </svg>
    );
}

/* 回车键：一条折线加箭头，形状取自常见的 ↵ */
const IconEnter = () => {
    return (
        <svg className="tail-icon tail-icon-enter" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12.5 3.5v5H3.5M6.5 5.5l-3 3 3 3"></path>
        </svg>
    );
}

/* Shift 键：空心上箭头 */
const IconShift = () => {
    return (
        <svg className="tail-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinejoin="round">
            <path d="M8 2.5 14 9.5h-3.2v4h-5.6v-4H2z"></path>
        </svg>
    );
}

/* ESC 没有通用的符号，用文字键帽代替 */
const KeyEsc = () => {
    return (
        <span className="tail-key">ESC</span>
    );
}

interface Props {
    hint: TailHint,
    /** 当前条目的默认动作文案；没有条目或条目没有动作时为 null，动作栏整块不渲染 */
    action_desc: string | null,
}

/**
 * 底部提示条：左边是选中方式，右边是当前动作与预览
 *
 * 提示文字只看传入的 hint 与 action_desc，不接别的数据；
 * 真正按键时是否显示（比如预览不可用）留给后续功能。
 */
const Tail = ({ hint, action_desc }: Props) => {
    const in_preview = hint === TailHint.Preview;
    return (
        <div className="tail-box">
            <div className="tail-group">
                {in_preview
                    ? <>
                        <KeyEsc></KeyEsc>
                        <span className="tail-text">关闭预览</span>
                    </>
                    : <>
                        <IconArrowUp></IconArrowUp>
                        <IconArrowDown></IconArrowDown>
                        <IconCursor></IconCursor>
                        <span className="tail-text">选中条目</span>
                    </>}
            </div>
            <div className="tail-group">
                {/* 动作栏：条目没有动作时整块不渲染，Enter 对它也确实什么都不做 */}
                {action_desc !== null
                    ? <>
                        <IconEnter></IconEnter>
                        <span className="tail-text">{action_desc}</span>
                    </>
                    : <></>}
                <span className="tail-combo">
                    <IconShift></IconShift>
                    <IconEnter></IconEnter>
                </span>
                <span className="tail-text">{in_preview ? "关闭预览" : "打开预览"}</span>
            </div>
        </div>
    );
}

export default Tail;
