import { Box, Static, Elastic, DividerTop, DividerBottom } from "./main/Layout";
import Head from "./main/Head";
import Body from "./main/Body";
import Tail from "./main/Tail";
import { set_page_main } from "./core";
import { useMainInteraction } from "./main/interaction/useMainInteraction";
import { default_action_label } from "./main/interaction/action_labels";

import "./core.css";

/**
 * 补全建议的占位串
 *
 * 本 effort 不实现补全，先沿用样式阶段那条假建议。
 */
const GHOST_VALUE = "World-yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy";

const App = () => {
  set_page_main();
  const { state, item_n, type_actions, input_ref } = useMainInteraction();

  return (
    <Box>
      <Static>
        <Head
          value={state.input}
          ghost={GHOST_VALUE}
          input_ref={input_ref}
          read_only={state.preview_open}>
        </Head>
      </Static>
      <DividerTop></DividerTop>
      <Elastic>
        <Body
          item_list={state.item_list}
          selection={state.selection}
          item_n={item_n}
          show_preview={state.preview_open}
          type_actions={type_actions}
          input_empty={state.input === ""}>
        </Body>
      </Elastic>
      <DividerBottom></DividerBottom>
      <Static>
        <Tail
          preview_open={state.preview_open}
          action_desc={default_action_label(type_actions, state.item_list[state.selection])}>
        </Tail>
      </Static>
    </Box>
  );
}

export default App;

/*
todo list
- 主动作（Primary Action）—— Enter
- 动作指示器（Hint Bar）—— 底部常驻
- 交互动作面板（Action Menu）—— 侧边挤占/覆盖
- 列表项高亮时左侧一个 2-4px 的 Accent Color 条、可延迟动画，（右侧显示常用图标，悬停才显示）
  - 动画用 absolute 定位的元素去实现
- 交互动作符号化语言
- 高亮优化：选中时增加 .is-selected css 类效果，复杂实现使用 Data Attributes
  - HTML <div className="list-item" data-selected={isSelected} data-action-type={item.type} />
  - CSS .list-item[data-selected="true"][data-action-type="copy"] { }
- 使用 React.memo 确保只有选中的行和刚刚失去选中的行发生变化
- （可选）高频触发、复杂的位移或缩放动画，使用 css will-change: background-color; 避免滥用，一般 transform 和 opacity 是值得使用的
- 列表数量多时，手动实现分页渲染， Rust 搜索（初级分页） + JS 分页
- 输入时及时搜索，异步搜索，使用随机数（自增id）作为标识，获得结果时若标识一致才显示
- 显示匹配分割线（国际化）：完全匹配、前缀匹配、关键词匹配、不完全匹配，匹配模式名称标签显示在右侧，不占空间、不改变布局（位移）
- 翻页交互：一行一行下翻，并使用边距预览、倒数第二条触发滚动
  - 下滑操作有限流 100-120 ms
  - 注意 Accent Color 动画拉长回缩，在持续下滑的时候表现出弹力
  - 注意 React 的 key 不变防止销毁 DOM
  - 下翻时给予 list 一个向上的 transform&opacity transition
*/

// =================================
// todo Item 实现
// =================================

// 下翻 list 过渡动画，使用 rAF 绕过 React 渲染任务队列、不写明 will-change 让其自动优化
// /* 基础槽位 */
// .item-slot {
//   transform: translateY(0);
//   opacity: 1;
//   /* 回弹阶段的动画：缓动函数决定了“弹力感” */
//   transition: transform 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275),
//               opacity 0.2s ease;
// }
// /* 瞬间触发态 */
// .item-slot.is-bumping {
//   /* 关键：关闭过渡，让位移瞬间发生 */
//   transition: none !important;
//   transform: translateY(2px);
//   opacity: 0.7;
// }

// const ListItemSlot = ({ data, isActive }) => {
//   const domRef = useRef<HTMLDivElement>(null);
//   const isFirstRender = useRef(true);

//   useLayoutEffect(() => {
//     if (isFirstRender.current) {
//       isFirstRender.current = false;
//       return;
//     }

//     const el = domRef.current;
//     if (!el) return;

//     // 第一帧：瞬间加上类名，让元素跳到 2px 位置，由于在 useLayoutEffect 中，这一步对用户是不可见的（发生在 Paint 之前）
//     el.classList.add('is-bumping');
//     // 第二帧：移除类名，CSS 的 transition 此时接管，让它弹回 0
//     const rafId = requestAnimationFrame(() => {
//       el.classList.remove('is-bumping');
//     });


//     // 强制重置，防止 React 复用槽位时带着旧的样式
//     return () => {
//       cancelAnimationFrame(rafId);
//       if (el) {
//         el.classList.remove('is-bumping');
//       }
//     };
//   }, [data?.id]); // 仅在数据更新时触发

//   return (
//     <div ref={domRef} className={`item-slot ${isActive ? 'active' : ''}`}>
//       {/* 内容... */}
//     </div>
//   );
// };

// =================================
// todo Note 实现
// =================================

// import { useMemo } from 'react';
// import { convertFileSrc } from '@tauri-apps/api/core';

// const useSafeHtml = (rawHtml: string) => {
//   return useMemo(() => {
//     // 1. 在内存中创建一个虚拟文档
//     const parser = new DOMParser();
//     const doc = parser.parseFromString(rawHtml, 'text/html');

//     // 2. 精准查找所有 img 标签
//     const imgs = doc.querySelectorAll('img');

//     imgs.forEach(img => {
//       const src = img.getAttribute('src');
//       // 3. 只有当它看起来像本地路径时才转换
//       if (src && !src.startsWith('http') && !src.startsWith('data:') && !src.startsWith('asset:')) {
//         img.setAttribute('src', convertFileSrc(src));
//       }

//       // 顺便可以在这里做一些“非暴力”的预处理
//       img.setAttribute('loading', 'lazy'); // 自动开启延迟加载
//       img.setAttribute('draggable', 'false'); // 禁止拖拽
//     });

//     // 4. 返回处理后的 HTML 字符串
//     return doc.body.innerHTML;
//   }, [rawHtml]);
// };

// const StaticRichText = ({ htmlContent }: { htmlContent: string }) => {
//   const containerRef = useRef<HTMLDivElement>(null);

//   useEffect(() => {
//     if (!containerRef.current) return;

//     // 1. 强制对所有图片绑定事件
//     const images = containerRef.current.querySelectorAll('img');
//     const handleClick = (e: Event) => {
//       const target = e.target as HTMLImageElement;
//       console.log('图片被点击了:', target.src);
//       // 这里可以调用 Tauri 的 API 弹出大图预览
//     };

//     images.forEach(img => {
//       img.addEventListener('click', handleClick);
//       // 顺手解决静态 HTML 的图片加载失败显示问题
//       img.style.cursor = 'pointer';
//     });

//     // 2. 清理函数（防止 React 严格模式下重复绑定）
//     return () => {
//       images.forEach(img => img.removeEventListener('click', handleClick));
//     };
//   }, [htmlContent]); // 当内容更新时重新绑定

//   return (
//     <div
//       ref={containerRef}
//       className="prose max-w-none"
//       dangerouslySetInnerHTML={{ __html: htmlContent }}
//     />
//   );
// };
