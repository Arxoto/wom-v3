import { Box, Static, Elastic, DividerTop, DividerBottom } from "./main/Layout";
import Head from "./main/Head";
import Body from "./main/Body";
import Tail from "./main/Tail";
import { set_page_main } from "./core";

import "./core.css";

const App = () => {
  set_page_main();
  return (
    <Box>
      <Static>
        <Head></Head>
      </Static>
      <DividerTop></DividerTop>
      <Elastic>
        <Body></Body>
      </Elastic>
      <DividerBottom></DividerBottom>
      <Static>
        <Tail></Tail>
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
- 翻页交互：一行一行下翻，并使用边距预览、倒数第二条触发滚动
  - 下滑操作有限流 100-120 ms
  - 注意 Accent Color 动画拉长回缩，在持续下滑的时候表现出弹力
  - 注意 React 的 key 不变防止销毁 DOM
  - 下翻时给予 list 一个向上的 transform&opacity transition
*/


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
