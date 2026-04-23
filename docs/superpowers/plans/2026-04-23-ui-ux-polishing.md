# Narya Phase 3 Stage 6: UI 视觉重构与美学提升计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 深度重构 Narya 的 UI 布局与视觉风格，打造简洁大方、交互友好的消费级界面。

**Architecture:** 
- **Theme**: 采用 Midnight Dark 主题，结合 Narya Blue (#1677ff) 品牌色。
- **Layout**: 侧边栏改为更细窄的图标+文字模式，内容区增加内边距与大圆角容器。
- **Components**: 全面升级按钮、卡片、滚动条的样式。

---

### Task 1: 侧边栏与主框架视觉升级

**Files:**
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 调整全局配色与布局**
引入更专业的背景色：`#141414` (页面背景) 和 `#1d1d1d` (卡片背景)。

- [ ] **Step 2: 重构侧边栏项样式**
增加选中时的“左侧蓝条”指示器，调整 Hover 效果。

---

### Task 2: 仪表盘与流量图视觉精修

**Files:**
- Modify: `narya-ui/src/components/traffic_chart.rs`
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 优化流量图颜色与填充**
使用渐变填充（模拟）和更细的描边，增加纵坐标刻度线。

- [ ] **Step 2: Dashboard 头部状态卡片美化**
增加“总流量”、“运行时间”等精致的小卡片。

---

### Task 3: 节点列表与规则面板精致化

**Files:**
- Modify: `narya-ui/src/components/proxy_list.rs`
- Modify: `narya-ui/src/components/rule_panel.rs`

- [ ] **Step 1: 重构节点卡片**
左侧显示协议标识小标签，右侧延迟显示改为渐变色圆点。

- [ ] **Step 2: 拖拽区交互反馈增强**
当 App 悬停在 DropZone 上方时，该区域显示虚线高亮。

---

### Task 4: 编译、运行与最终审美校对

- [ ] **Step 1: 运行编译**
`cargo build`
- [ ] **Step 2: 视觉 Review**
确认整体间距、字体大小是否协调。
