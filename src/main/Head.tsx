import type { RefObject } from "react";

import "./Head.css";

interface Props {
    value: string,
    ghost: string,
    input_ref: RefObject<HTMLInputElement | null>,
    read_only: boolean,
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
 * 顶部输入框：真实 input 收字符，下面同宽的一层只负责显示 ghost 建议
 *
 * 本组件只负责显示与受控回写：输入与合成（IME）事件都由接线层挂在同一个 input 上
 * （见 useMainInteraction），值往外转交后由受控的 `value` 落回来。
 * 预览打开时只读：焦点不动（主动 blur 会打断可能正在进行的合成），输入内容保留。
 */
const Head = ({ value, ghost, input_ref, read_only }: Props) => {
    return (
        <div className="head-box">
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
                />
            </div>
            <div className="input-tag head-static">N/A</div>
            <img className='wom-icon head-static' src={undefined} alt="" data-tauri-drag-region />
        </div>
    );
}

export default Head;
