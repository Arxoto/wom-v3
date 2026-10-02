import type { ReactNode } from "react";

export const IconCopy = (): ReactNode => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <rect x="5.8" y="5.8" width="7.7" height="7.7" rx="1.6"></rect>
            <path d="M3.5 10.2V3.9a1.4 1.4 0 0 1 1.4-1.4h6.3"></path>
        </svg>
    );
}

export const IconOpenUrl = (): ReactNode => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="8" cy="8" r="5.5"></circle>
            <path d="M2.5 8h11"></path>
            <path d="M8 2.5c1.7 1.7 2.5 3.5 2.5 5.5s-.8 3.8-2.5 5.5c-1.7-1.7-2.5-3.5-2.5-5.5S6.3 4.2 8 2.5z"></path>
        </svg>
    );
}

export const IconOpenPath = (): ReactNode => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2.5 12.2V4.4a1.2 1.2 0 0 1 1.2-1.2h2.7l1.6 1.9h5.3a1.2 1.2 0 0 1 1.2 1.2v5.9a1.2 1.2 0 0 1-1.2 1.2H3.7a1.2 1.2 0 0 1-1.2-1.2z"></path>
        </svg>
    );
}

export const IconReveal = (): ReactNode => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2.5 11.6V4.4a1.2 1.2 0 0 1 1.2-1.2h2.7l1.6 1.9h5.3a1.2 1.2 0 0 1 1.2 1.2v5.3a1.2 1.2 0 0 1-1.2 1.2H3.7a1.2 1.2 0 0 1-1.2-1.2z"></path>
            <circle cx="7.4" cy="9.3" r="2.1"></circle>
            <path d="M8.9 10.8l1.6 1.6"></path>
        </svg>
    );
}

const IconSwitchPrev = (): ReactNode => {
    return (
        <svg className="item-action-arrow" viewBox="0 0 16 16" aria-hidden="true" fill="currentColor">
            <path d="M11 3.2 5.2 8 11 12.8z"></path>
        </svg>
    );
}

const IconSwitchNext = (): ReactNode => {
    return (
        <svg className="item-action-arrow" viewBox="0 0 16 16" aria-hidden="true" fill="currentColor">
            <path d="M5 3.2 10.8 8 5 12.8z"></path>
        </svg>
    );
}

export const ACTION_SWITCH_MARKS: Record<"prev" | "next", ReactNode> = {
    prev: <IconSwitchPrev></IconSwitchPrev>,
    next: <IconSwitchNext></IconSwitchNext>,
};
