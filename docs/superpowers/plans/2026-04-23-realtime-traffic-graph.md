# Narya Phase 3 Stage 2: 实时流量图实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 基于 GPUI Canvas 实现 60FPS 的上行/下行实时流量波形图。

**Architecture:** 
- **Model**: `TrafficStore` 维护一个固定长度的环形队列（Ring Buffer），存储最近 60 个秒级的流量数据点。
- **View**: `TrafficChart` 组件，在每一帧渲染时从 `TrafficStore` 读取数据并生成 Path。

**Tech Stack:** Rust, GPUI Canvas API.

---

### Task 1: 建立流量数据存储逻辑

**Files:**
- Create: `narya-ui/src/models/traffic.rs`
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 定义 `TrafficData` 和 `TrafficStore`**
存储 `up` 和 `down` 的速率。

```rust
pub struct TrafficData {
    pub up: f32,
    pub down: f32,
}

pub struct TrafficStore {
    pub history: Vec<TrafficData>,
    pub max_samples: usize,
}
```

- [ ] **Step 2: 实现定时模拟数据更新**
在 UI 启动时开启一个 `cx.spawn` 定时器，每秒向 Store 注入一个随机数据点并通知 UI 更新。

---

### Task 2: 实现自定义绘图组件 (TrafficChart)

**Files:**
- Create: `narya-ui/src/components/traffic_chart.rs`

- [ ] **Step 1: 编写 Canvas 绘制逻辑**
- 计算坐标系：将速率映射到高度，将时间映射到宽度。
- 绘制下行流量：使用蓝色填充。
- 绘制上行流量：使用绿色描边。

---

### Task 3: 集成到 Dashboard 首页

**Files:**
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 在 Dashboard 选项卡中展示 `TrafficChart`**
将占位文本替换为真实的流量组件。
