import "./Layout.css";

interface ReactDomWithChildren {
    children: any
}

const Box = ({ children }: ReactDomWithChildren) => {
    return <div className="layout-box">{children}</div>
}

const Static = ({ children }: ReactDomWithChildren) => {
    return <div className="static">{children}</div>
}

const Elastic = ({ children }: ReactDomWithChildren) => {
    return <div className="elastic">{children}</div>
}

const Divider = () => {
    return <div className="divider"></div>
}

export { Box, Static, Elastic, Divider };
