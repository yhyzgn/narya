# Narya Phase 3 Stage 5: 真实内核规则联动实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现从 UI 拖拽到 eBPF 内核态过滤规则的实时同步与生效。

**Architecture:** 
- **BPF Map**: 使用 `HashMap<u32, ProcessConfig>`。Key 是进程的 PID 或 cgroup ID，Value 是动作（Direct/Proxy/Reject）。
- **Kernel (eBPF)**: `narya_egress` 程序在发包前查询 Map，决定是否返回 `SK_DROP` (Reject) 或允许通过。
- **User (Daemon)**: `EbpfProcessTracker` 监听 IPC 下发的规则，负责清理并重新填充 BPF Map。

**Tech Stack:** Rust, Aya (eBPF), IPC.

---

### Task 1: 扩展共享数据结构

**Files:**
- Modify: `narya-ebpf-common/src/lib.rs`

- [ ] **Step 1: 完善 `ProcessConfig` 定义**
支持多种过滤动作。

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessConfig {
    pub action: u32, // 0: Pass, 1: Drop, 2: Redirect (Future)
}
```

---

### Task 2: 实现内核态实时过滤逻辑

**Files:**
- Modify: `narya-ebpf/src/main.rs`

- [ ] **Step 1: 定义 `RULES_MAP`**
- [ ] **Step 2: 在 `narya_egress` 中加入查表逻辑**
利用 `bpf_get_current_pid_tgid` 获取当前 PID 并匹配规则。

---

### Task 3: 实现用户态 Map 更新器

**Files:**
- Modify: `daemon/src/tracker.rs`

- [ ] **Step 1: 在 `EbpfProcessTracker` 中持有 Map 的 Handle**
- [ ] **Step 2: 实现 `update_bypass_rules` 写入逻辑**
将 UI 传来的进程名/PID 映射转换并写入内核。

---

### Task 4: 闭环冒烟测试

- [ ] **Step 1: 验证编译**
`cargo build`
- [ ] **Step 2: 日志确认**
确认拖拽后后端打印 `Map updated successfully`。
