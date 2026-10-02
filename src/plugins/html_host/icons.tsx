import type { ReactNode } from "react";

export const IconHtmlPlugin = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M3.6 1.8h6l3 3v9.4H3.6z"></path>
            <path d="M9.6 1.8v3h3"></path>
            <path d="M6 8.6h4M6 11h2.6"></path>
        </svg>
    );
}

export const IconOpenPage = (): ReactNode => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
            <path d="M9.4 2.6h4v4"></path>
            <path d="M13.4 2.6 7.6 8.4"></path>
            <path d="M12 9.4v3.2a.8.8 0 0 1-.8.8H3.4a.8.8 0 0 1-.8-.8V4.8a.8.8 0 0 1 .8-.8h3.2"></path>
        </svg>
    );
}
