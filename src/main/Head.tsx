import type { RefObject } from "react";

import "./Head.css";

interface Props {
    value: string,
    ghost: string,
    on_change: (value: string) => void,
    input_ref: RefObject<HTMLInputElement | null>,
}

/**
 * 顶部输入框：真实 input 收字符，下面同宽的一层只负责显示 ghost 建议
 *
 * 值的变化往外转交（受控），合成事件由接线层挂在 input 上（见 useMainInteraction）。
 */
const Head = ({ value, ghost, on_change, input_ref }: Props) => {
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
                    onChange={event => on_change(event.target.value)}
                    spellCheck="false"
                />
            </div>
            <div className="input-tag head-static">N/A</div>
            <img className='wom-icon head-static' src={undefined} alt="" data-tauri-drag-region />
        </div>
    );
}

export default Head;
