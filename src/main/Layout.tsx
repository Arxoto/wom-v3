import "./Layout.css";

interface ReactDomWithChildren {
    children: any
}

function Box({ children }: ReactDomWithChildren) {
    return <div className="layout-box">{children}</div>
}

function Static({ children }: ReactDomWithChildren) {
    return <div className="static">{children}</div>
}

function Elastic({ children }: ReactDomWithChildren) {
    return <div className="elastic">{children}</div>
}

function Divider() {
    return <div className="divider"></div>
}

export { Box, Static, Elastic, Divider };
