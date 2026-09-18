import type { ReactNode } from "react";

import type { ItemActionId } from "../../core";

/**
 * Item Action 的图标
 *
 * 行内动作是图标不是文字，所以每个动作配一张。
 * 图标是行内的一部分，所以尺寸与颜色都由 Item.css 决定（跟着 --item-h 缩放、currentColor 跟随整行状态）。
 */

const IconCopy = () => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <rect x="5.8" y="5.8" width="7.7" height="7.7" rx="1.6"></rect>
            <path d="M3.5 10.2V3.9a1.4 1.4 0 0 1 1.4-1.4h6.3"></path>
        </svg>
    );
}

/* 网页：地球，一条纬线加两条经线 */
const IconOpenUrl = () => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="8" cy="8" r="5.5"></circle>
            <path d="M2.5 8h11"></path>
            <path d="M8 2.5c1.7 1.7 2.5 3.5 2.5 5.5s-.8 3.8-2.5 5.5c-1.7-1.7-2.5-3.5-2.5-5.5S6.3 4.2 8 2.5z"></path>
        </svg>
    );
}

/* 文件与文件夹：文件夹 */
const IconOpenPath = () => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2.5 12.2V4.4a1.2 1.2 0 0 1 1.2-1.2h2.7l1.6 1.9h5.3a1.2 1.2 0 0 1 1.2 1.2v5.9a1.2 1.2 0 0 1-1.2 1.2H3.7a1.2 1.2 0 0 1-1.2-1.2z"></path>
        </svg>
    );
}

/* 在文件夹中选中：文件夹里一把放大镜 */
const IconReveal = () => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2.5 11.6V4.4a1.2 1.2 0 0 1 1.2-1.2h2.7l1.6 1.9h5.3a1.2 1.2 0 0 1 1.2 1.2v5.3a1.2 1.2 0 0 1-1.2 1.2H3.7a1.2 1.2 0 0 1-1.2-1.2z"></path>
            <circle cx="7.4" cy="9.3" r="2.1"></circle>
            <path d="M8.9 10.8l1.6 1.6"></path>
        </svg>
    );
}

/* 笔记：折角文档加两条正文线 */
const IconOpenNote = () => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <path d="M4.2 2.5h5.1l2.5 2.5v8.5H4.2z"></path>
            <path d="M9.3 2.5V5h2.5"></path>
            <path d="M6.2 8.2h3.6M6.2 10.5h3.6"></path>
        </svg>
    );
}

/** 动作图标表：按 `ItemActionId` 查 */
export const ACTION_ICONS: Record<ItemActionId, ReactNode> = {
    copy: <IconCopy></IconCopy>,
    open_url: <IconOpenUrl></IconOpenUrl>,
    open_path: <IconOpenPath></IconOpenPath>,
    reveal: <IconReveal></IconReveal>,
    open_note: <IconOpenNote></IconOpenNote>,
};
