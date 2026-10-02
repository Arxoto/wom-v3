import type { ReactNode } from "react";

export const IconJsPlugin = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M6 2.6v3M10 2.6v3"></path>
            <path d="M4 5.6h8v2.6a4 4 0 0 1-8 0z"></path>
            <path d="M8 12.2v1.4"></path>
        </svg>
    );
}

export const IconOpenSearch = (): ReactNode => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="7" cy="7" r="4.2"></circle>
            <path d="M10.1 10.1 13.4 13.4"></path>
        </svg>
    );
}
