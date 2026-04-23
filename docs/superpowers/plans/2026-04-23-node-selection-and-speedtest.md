# Narya Phase 3 Stage 7: 节点选择与延迟测速实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现代理节点的交互选择功能，并提供真实的并发延迟测速能力。

**Architecture:** 
- **Model**: 在 `ProfileStore` 中增加 `active_node` 字段。
- **Logic**: 
    - **Selection**: UI 点击触发 IPC 指令给 Daemon，通知 Sing-box 切换 Outbound。
    - **Latency**: 使用 `utils::TOKIO_RUNTIME` 并发对所有节点执行 `reqwest` 请求（针对 Google 或百度），计算从握手到首包的时间。

**Tech Stack:** Rust, GPUI, Reqwest, Tokio.

---

### Task 1: 完善 Profile 模型与 UI 选择逻辑

**Files:**
- Modify: `narya-ui/src/models/profile.rs`
- Modify: `narya-ui/src/components/proxy_list.rs`
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 在 `ProfileStore` 中增加 `active_node: Option<String>`**
- [ ] **Step 2: 修改 `ProxyList` 渲染**
选中节点显示蓝色边框或背景，点击触发 `Workspace::select_proxy`。

---

### Task 2: 实现真实并发测速

**Files:**
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 编写 `test_latency` 异步方法**
使用全局 `TOKIO_RUNTIME`。对每个节点，模拟一个 HTTP 请求（目前先 Mock 请求过程，但保留异步计时逻辑，为 Phase 4 真实转发做准备）。

---

### Task 3: 闭环与 IPC 联动

**Files:**
- Modify: `daemon/src/ipc.rs`
- Modify: `daemon/src/controller.rs`

- [ ] **Step 1: 支持 `select_proxy <name>` 指令**
后端收到后打印日志，并准备调用 Sing-box reload。
