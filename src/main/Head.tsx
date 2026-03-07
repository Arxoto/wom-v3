import "./Head.css";

const Head = () => {
    let typed_value = "hello-aaaaaaaaaaaaaaaa-000000000000000000000000";
    let ghost_value = "_world-yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy";
    return (
        <div className="head-box">
            <div className="head-space"></div>
            <div className="input-container head-elastic">
                <div className="suggestion-layer input-base">
                    <span className="typed-text">{typed_value}</span>
                    <span className="ghost-text">{ghost_value}</span>
                </div>
                <input type="text" className="real-input input-base" value={typed_value} onChange={_e => { }} spellCheck="false" />
            </div>
            <div className="input-tag head-static">N/A</div>
            <img className='wom-icon head-static' src={undefined} alt="" data-tauri-drag-region />
        </div>
    );
}

export { Head };

