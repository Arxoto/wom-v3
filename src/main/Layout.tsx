import "./Layout.css";

interface Props {
    children: React.ReactNode
}

export const Box = ({ children }: Props) => {
    return <div className="layout-box">{children}</div>
}

export const Static = ({ children }: Props) => {
    return <div className="static">{children}</div>
}

export const Elastic = ({ children }: Props) => {
    return <div className="elastic">{children}</div>
}

export const DividerTop = () => {
    return <div className="divider-top"></div>
}

export const DividerBottom = () => {
    return <div className="divider-bottom"></div>
}
