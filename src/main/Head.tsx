import type { RefObject } from "react";

import "./Head.css";

interface Props {
    value: string,
    /** 真实输入为空时显示的提示文案，有输入时传空串（见 AppMain） */
    ghost: string,
    input_ref: RefObject<HTMLInputElement | null>,
    read_only: boolean,
    /** 空态：还没有结论（没输入过、输入为空，或查询还没回来）——标签留白 */
    empty: boolean,
    /** 当前结果集的总数（不是已加载条数） */
    total: number,
    /** List Position：`Selection` 落在已加载列表的第几行 */
    selection: number,
}

/**
 * 受控输入框上的占位处理器
 *
 * `value` 存在时 React 要求输入框挂一个 change 处理器，否则按只读渲染（并在 dev 报警，
 * 事件结束后会把 DOM 值刷回 `value`）。真实的输入事件由接线层挂在同一个 input 上
 * （见 useMainInteraction），所以这里只需要让 React 认可它是可编辑的。
 */
const keep_editable = () => { };

/**
 * 右上角标签显示什么
 */
const tag_text = (empty: boolean, total: number, selection: number) => {
    // 空态（还没有结论）
    if (empty) return "";
    // 这一份结论是「没有结果」
    if (total === 0) return "N/A";
    // 有结果显示 「当前选中项的序号 / 结果总数」
    return `${selection + 1}/${total}`;
}

/**
 * 顶部输入框：真实 input 收字符，下面同宽的一层只负责显示 ghost 提示
 *
 * 本组件只负责显示与受控回写：输入事件等由接线层处理后通过 `value` 回落（ useMainInteraction ）。
 * 预览打开时只读：焦点不动，输入内容保留。
 */
const Head = ({ value, ghost, input_ref, read_only, empty, total, selection }: Props) => {
    return (
        <div className="head-box" data-tauri-drag-region="deep">
            <div className="head-space"></div>
            <div className="input-container head-elastic">
                <div className="suggestion-layer input-base">
                    <span className="typed-text">{value}</span>
                    <span className="ghost-text">{ghost}</span>
                </div>
                <input
                    ref={input_ref}
                    type="text"
                    className="real-input input-base"
                    value={value}
                    onChange={keep_editable}
                    readOnly={read_only}
                    spellCheck="false"
                    autoComplete="off"
                    autoCorrect="off"
                    autoCapitalize="off"
                />
            </div>
            <div className="input-tag head-static">{tag_text(empty, total, selection)}</div>
            <img className='wom-icon head-static' src={undefined} alt="" />
        </div>
    );
}

export default Head;
