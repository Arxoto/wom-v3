import "./Body.css";

const BodyPreview = () => {
    return <>
        <div className="body-divider"></div>
        <div className="body-preview">
            <div className="body-preview-icon">ICON</div>
            <div className="body-preview-title">TITLE-ssssssssssssssssssssssssssssssssssssssssssssssssssssssssss</div>
            <div className="body-preview-divider"></div>
            <div className="body-preview-details">DETAILS</div>
        </div>
    </>;
}

const Body = () => {
    let show_preview = true;
    return (
        <div className="body-box">
            <div className="body-items">
                asdsdadadadasdasd
                <br />
                ssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss
                <br />
                sssssssssssssssss
            </div>
            {show_preview ? <BodyPreview></BodyPreview> : <></>}
        </div>
    );
}

export default Body;
