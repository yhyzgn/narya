# Narya Phase 3 Stage 3: 智能分流拖拽交互实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现应用在“直连”与“代理”分类间的可视化拖拽交互，并同步更新底层过滤规则。

**Architecture:** 
- **Model**: `RuleStore` 存储 App 标识与其归属组的映射。
- **View**: `RulesView` 包含：
    - `AppPool`: 左侧待选应用池。
    - `DirectZone`: 右侧直连目标区。
    - `ProxyZone`: 右侧代理目标区。
- **Interaction**: 使用 GPUI 的 `Draggable` 和 `Drag` 事件模型。

**Tech Stack:** Rust, GPUI Drag&Drop API, IPC.

---

### Task 1: 建立 Rule 数据模型

**Files:**
- Create: `narya-ui/src/models/rule.rs`
- Modify: `narya-ui/src/models/mod.rs`
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 定义 `AppRule` 和 `RuleStore`**

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppInfo {
    pub id: String, // PID or Name
    pub name: String,
}

pub struct RuleStore {
    pub unassigned: Vec<AppInfo>,
    pub direct: Vec<AppInfo>,
    pub proxy: Vec<AppInfo>,
}
```

- [ ] **Step 2: 在 `Workspace` 中初始化 `RuleStore`**

---

### Task 2: 实现应用拉取逻辑 (Mock -> Real)

**Files:**
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 模拟从系统抓取活跃应用**
调用 `ProcessTracker` 的 Mock 逻辑填充 `unassigned` 列表。

---

### Task 3: 构建拖拽交互界面

**Files:**
- Create: `narya-ui/src/components/rule_panel.rs`
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 实现可拖拽的应用卡片组件**
- [ ] **Step 2: 实现两个 Drop 接收区组件**
- [ ] **Step 3: 实现拖拽释放后的数据交换逻辑**
利用 `cx.new_view` 创建拖拽中的“影子”视图。

---

### Task 4: 规则持久化与 IPC 联动 (预览)

- [ ] **Step 1: 当数据发生变化时，打印规则更新日志**
为下一步对接真正内核拦截做准备。
