# Narya Phase 3 Stage 4: 系统应用感知与 IPC 联动实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 UI 能够显示系统中真实的运行应用，并实现拖拽规则与后端分流引擎的实时同步。

**Architecture:** 
- **Daemon (IPC Server)**: 
    - 增加 `GET_APPS` 指令：返回当前活跃的进程列表。
    - 增加 `UPDATE_RULES` 指令：接收 UI 下发的白名单/黑名单。
- **UI (IPC Client)**: 
    - 使用 `tokio::net::UnixStream` 与 `/tmp/narya.sock` 通信。
    - 在 `Workspace` 切换到 Rules 标签时触发拉取。

**Tech Stack:** Rust, IPC (Unix Domain Socket), Serde, Sysinfo.

---

### Task 1: 扩展后端 IPC 协议

**Files:**
- Modify: `daemon/src/ipc.rs`

- [ ] **Step 1: 增强 IPC 请求处理逻辑**
支持 `get_apps` 和 `update_rules <json>` 指令。

```rust
// 逻辑伪代码
"get_apps" => {
    let apps = daemon.tracker.list_network_apps().await?;
    serde_json::to_string(&apps)?
}
```

---

### Task 2: 在 UI 中实现 IPC 客户端

**Files:**
- Create: `narya-ui/src/ipc_client.rs`
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 编写简单的异步 IPC 请求函数**
使用 `utils::TOKIO_RUNTIME` 来处理 UDS 连接。

- [ ] **Step 2: 联动 Rules 面板**
当进入 Rules 标签时，自动调用后端接口更新 `RuleStore` 中的 `unassigned` 列表。

---

### Task 3: 实现规则双向同步

**Files:**
- Modify: `narya-ui/src/components/rule_panel.rs`
- Modify: `daemon/src/controller.rs`

- [ ] **Step 1: 拖拽触发 IPC 发送**
当应用在 UI 上被分类后，立即将新的规则表发送给 Daemon。

- [ ] **Step 2: 后端热更新 RuleEngine**
Daemon 收到规则后，调用 `engine.write()` 更新内存中的匹配规则。
