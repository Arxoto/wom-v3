import type { ReactNode } from "react";

export const IconSys = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="8" cy="8" r="3.3"></circle>
            <circle cx="8" cy="8" r="1.3"></circle>
            <path d="M12.2 8h1.3M8 12.2v1.3M3.8 8H2.5M8 3.8V2.5M11 11l.9.9M5 11l-.9.9M5 5 4.1 4.1M11 5l.9-.9"></path>
        </svg>
    );
}

export const IconCmd = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <rect x="2.4" y="3.4" width="11.2" height="9.2" rx="1.6"></rect>
            <path d="M5.2 6.4 7.1 8.3 5.2 10.2"></path>
            <path d="M8.6 10.4h2.6"></path>
        </svg>
    );
}

export const IconWeb = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="8" cy="8" r="5.6"></circle>
            <path d="M2.4 8h11.2"></path>
            <ellipse cx="8" cy="8" rx="2.6" ry="5.6"></ellipse>
        </svg>
    );
}

export const IconScan = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2.4 4.2h5.8M2.4 7.4h5.8M2.4 10.6h3.4"></path>
            <circle cx="11.4" cy="10.2" r="2.1"></circle>
            <path d="M12.9 11.7 14.3 13.1"></path>
        </svg>
    );
}
