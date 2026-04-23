# Narya Phase 3 Stage 8: 搜索功能与视觉细节精修计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Rules 页面的实时搜索功能，并对全平台 UI 进行最终的审美与交互细节微调。

**Architecture:** 
- **View**: 引入 `SearchHeader` 组件，利用 GPUI 的 `on_key_down` 或者是标准的输入框模式。
- **Theme**: 优化全局 CSS-like 属性，定义统一的 `ROUNDED_CORNER` 和 `BORDER_COLOR` 常量。

---

### Task 1: 实现 Rules 实时搜索交互

**Files:**
- Modify: `narya-ui/src/lib.rs`
- Modify: `narya-ui/src/components/rule_panel.rs`

- [ ] **Step 1: 建立键盘输入监听器**
在 `Workspace` 根视图中拦截非控制键输入，更新 `rule_store.search_query`。
- [ ] **Step 2: 搜索框视觉反馈**
搜索框在有内容时显示蓝色的“清除”图标，并实时高亮匹配的应用。

---

### Task 2: UI 视觉层次感深度优化

**Files:**
- Modify: `narya-ui/src/lib.rs`
- Modify: `narya-ui/src/components/proxy_list.rs`

- [ ] **Step 1: 引入卡片悬浮动效**
所有的卡片（节点卡片、App 卡片）在 Hover 时增加 `offset_y(-1)` 和边框亮度提升。
- [ ] **Step 2: 细化状态标识**
使用更专业的点状图标表示“运行中”、“已连接”状态。

---

### Task 3: 仪表盘 Dashboard 布局对齐

**Files:**
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 流量图背景网格**
在 Canvas 中增加 Subtle 的水平参考线（1MB, 512KB 等）。
- [ ] **Step 2: 运行时间展示**
在 Dashboard 顶部增加实时运行时间计数器。

---

### Task 4: 编译、运行与最终交付

- [ ] **Step 1: 运行全项目编译**
`cargo build`
- [ ] **Step 2: 交互 Review**
确认拖拽、搜索、切换、测速各环节的流畅度与美感。
