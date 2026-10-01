import type { ReactNode } from "react";

/**
 * launcher 插件的动作图标
 *
 * 行内动作是图标不是文字，所以每个动作配一张。图标是行内的一部分，
 * 所以尺寸与颜色都由 Item.css 决定（跟着 --item-h 缩放、currentColor 跟随整行状态）。
 *
 * 图标由 launcher 自己提供、注册进前端注册表（见 `launcher.tsx`）：动作 id 是插件定义的，
 * 「这个动作长什么样」自然也是插件的事。
 *
 * `open_note` 那张图标在接入这一轮**删掉了**：note 还不是插件，留着就是没有调用方的
 * 死图标；等它成为插件（spec §2.1 理由 1）时，从 git 历史里取回来即可。
 */

export const IconCopy = (): ReactNode => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <rect x="5.8" y="5.8" width="7.7" height="7.7" rx="1.6"></rect>
            <path d="M3.5 10.2V3.9a1.4 1.4 0 0 1 1.4-1.4h6.3"></path>
        </svg>
    );
}

/* 网页：地球，一条纬线加两条经线 */
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

/* 文件与文件夹：文件夹 */
export const IconOpenPath = (): ReactNode => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2.5 12.2V4.4a1.2 1.2 0 0 1 1.2-1.2h2.7l1.6 1.9h5.3a1.2 1.2 0 0 1 1.2 1.2v5.9a1.2 1.2 0 0 1-1.2 1.2H3.7a1.2 1.2 0 0 1-1.2-1.2z"></path>
        </svg>
    );
}

/* 在文件夹中选中：文件夹里一把放大镜 */
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

/* 两侧的切换三角：哪一侧还有动作就露哪一侧，样式见 Item.css 的 .item-action-arrow */
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

/** 动作图标两侧的切换三角：`prev` 朝左、`next` 朝右，位置常驻、由 Item 决定露不露 */
export const ACTION_SWITCH_MARKS: Record<"prev" | "next", ReactNode> = {
    prev: <IconSwitchPrev></IconSwitchPrev>,
    next: <IconSwitchNext></IconSwitchNext>,
};
