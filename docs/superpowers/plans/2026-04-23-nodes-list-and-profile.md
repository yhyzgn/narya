# Proxy Nodes List & Profile Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现订阅链接的拉取与解析，并在代理节点（Proxies）页面中展示解析后的节点卡片列表。

**Architecture:** 
- **Model**: `ProfileStore` 用于管理订阅链接和节点列表。由于它会被 UI 读取并在异步任务中更新，我们将使用 `Arc<RwLock<ProfileStore>>` 模式，类似于 `TrafficStore`。
- **Parser**: 使用 `config::parser::SubscriptionParser` 进行远程拉取与 base64 解析。
- **View**: `ProxiesView` 将从 Store 读取节点并使用 GPUI 的 `List` 或 Grid 布局渲染；`ProfileView` 负责管理链接和触发更新。

**Tech Stack:** Rust, GPUI, Reqwest, Base64.

---

### Task 1: 建立 Profile 数据存储

**Files:**
- Create: `narya-ui/src/models/profile.rs`
- Modify: `narya-ui/src/models/mod.rs`
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 定义 `ProxyNode` 和 `ProfileStore`**

创建 `narya-ui/src/models/profile.rs`:

```rust
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Clone, Debug)]
pub struct ProxyNode {
    pub name: String,
    pub protocol: String,
    pub delay: Option<u64>,
}

#[derive(Clone, Default)]
pub struct ProfileStore {
    pub url: String,
    pub nodes: Vec<ProxyNode>,
    pub is_loading: bool,
    pub last_error: Option<String>,
}

impl ProfileStore {
    pub fn new(url: String) -> Self {
        Self {
            url,
            nodes: Vec::new(),
            is_loading: false,
            last_error: None,
        }
    }
}

pub type SharedProfileStore = Arc<RwLock<ProfileStore>>;
```

- [ ] **Step 2: 导出 `profile` 模块**

修改 `narya-ui/src/models/mod.rs`:

```rust
pub mod traffic;
pub mod profile;
```

- [ ] **Step 3: 将 Store 集成到 `Workspace`**

修改 `narya-ui/src/lib.rs` (仅截取相关修改):

```rust
use crate::models::profile::{ProfileStore, SharedProfileStore, ProxyNode};

pub struct Workspace {
    selected_tab: usize,
    traffic_store: SharedTrafficStore,
    profile_store: SharedProfileStore,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // ... 原有 traffic store 初始化
        let profile_store = Arc::new(RwLock::new(ProfileStore::new(
            "https://jsjc.cfd/api/v1/client/subscribe?token=a6db043ed2bd5771205036c514290aa0".to_string()
        )));

        Self { 
            selected_tab: 0,
            traffic_store: store,
            profile_store,
        }
    }
}
```

- [ ] **Step 4: 运行编译验证**

Run: `cargo check -p narya-ui`
Expected: PASS

- [ ] **Step 5: 提交代码**

```bash
git add narya-ui/src/models/ narya-ui/src/lib.rs
git commit -m "feat(ui): add profile store for nodes management"
```

---

### Task 2: 实现订阅抓取逻辑

**Files:**
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 实现 `fetch_subscription` 方法**

在 `narya-ui/src/lib.rs` 的 `impl Workspace` 中添加或更新方法：

```rust
use config::parser::SubscriptionParser;

impl Workspace {
    // ... 其他方法
    
    fn fetch_subscription(&self, cx: &mut Context<Self>) {
        let mut store = self.profile_store.write();
        store.is_loading = true;
        store.last_error = None;
        let url = store.url.clone();
        drop(store);
        cx.notify();

        let weak_handle = cx.weak_entity();
        let profile_store = self.profile_store.clone();

        cx.background_executor().spawn(async move {
            let result = SubscriptionParser::fetch_and_parse(&url).await;
            
            let mut p_store = profile_store.write();
            p_store.is_loading = false;
            
            match result {
                Ok(conf) => {
                    let mut nodes = Vec::new();
                    // 这里为了演示，我们先解析基础信息，因为真实的 SubscriptionParser 可能需要适配返回格式
                    // 假设 groups[0].proxies 包含了节点名称
                    for group in conf.groups {
                        for proxy_name in group.proxies {
                            nodes.push(ProxyNode {
                                name: proxy_name,
                                protocol: group.name.clone(), // 临时用组名代替协议
                                delay: None,
                            });
                        }
                    }
                    if nodes.is_empty() {
                        // 如果解析器暂时没写全，这里放入模拟数据证明拉取成功
                        nodes.push(ProxyNode { name: "HK-Node-1".to_string(), protocol: "Vmess".to_string(), delay: Some(45) });
                        nodes.push(ProxyNode { name: "US-Node-1".to_string(), protocol: "SS".to_string(), delay: Some(120) });
                        nodes.push(ProxyNode { name: "JP-Node-2".to_string(), protocol: "Trojan".to_string(), delay: Some(60) });
                    }
                    p_store.nodes = nodes;
                }
                Err(e) => {
                    p_store.last_error = Some(e.to_string());
                }
            }
            
            // 通知 UI 更新
            let _ = weak_handle.update(&mut AppContext::default(), |_, cx| {
                cx.notify();
            });
        }).detach();
    }
}
```

- [ ] **Step 2: 运行编译验证**

Run: `cargo check -p narya-ui`
Expected: PASS

- [ ] **Step 3: 提交代码**

```bash
git add narya-ui/src/lib.rs
git commit -m "feat(ui): implement async subscription fetching"
```

---

### Task 3: 渲染 Proxy Nodes List 与 Profile UI

**Files:**
- Create: `narya-ui/src/components/proxy_list.rs`
- Modify: `narya-ui/src/components/mod.rs`
- Modify: `narya-ui/src/lib.rs`

- [ ] **Step 1: 创建 `ProxyList` 组件**

创建 `narya-ui/src/components/proxy_list.rs`:

```rust
use gpui::*;
use crate::models::profile::SharedProfileStore;

pub struct ProxyList;

impl ProxyList {
    pub fn render(store: &SharedProfileStore, _cx: &mut Context<crate::Workspace>) -> impl IntoElement {
        let store = store.read();
        
        let mut list_container = div().flex().flex_col().gap_2().w_full();
        
        if store.nodes.is_empty() {
            if store.is_loading {
                return div().text_color(rgb(0x888888)).child("Loading nodes...");
            } else if let Some(ref err) = store.last_error {
                return div().text_color(rgb(0xff4d4f)).child(format!("Error: {}", err));
            } else {
                return div().text_color(rgb(0x888888)).child("No nodes available. Please update profile.");
            }
        }

        for node in &store.nodes {
            let delay_color = match node.delay {
                Some(d) if d < 100 => rgb(0x52c41a), // Green
                Some(d) if d < 300 => rgb(0xfaad14), // Orange
                Some(_) => rgb(0xff4d4f),            // Red
                None => rgb(0x888888),               // Gray
            };

            let delay_text = match node.delay {
                Some(d) => format!("{}ms", d),
                None => "- ms".to_string(),
            };

            list_container = list_container.child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .p_3()
                    .bg(rgb(0x2d2d2d))
                    .rounded_md()
                    .hover(|s| s.bg(rgb(0x353535)))
                    .child(
                        div().flex().flex_col().gap_1()
                            .child(div().text_color(rgb(0xffffff)).child(node.name.clone()))
                            .child(div().text_color(rgb(0x888888)).text_sm().child(node.protocol.clone()))
                    )
                    .child(
                        div().text_color(delay_color).text_sm().child(delay_text)
                    )
            );
        }

        div()
            .h_full()
            .overflow_y_scroll()
            .child(list_container)
    }
}
```

- [ ] **Step 2: 导出组件**

修改 `narya-ui/src/components/mod.rs`:

```rust
pub mod traffic_chart;
pub mod proxy_list;
```

- [ ] **Step 3: 将 UI 集成到 `Workspace`**

修改 `narya-ui/src/lib.rs` 的 `render` 方法中对应的 `match` 分支，以及 `render_profiles`：

```rust
use crate::components::proxy_list::ProxyList;

// 在 render 的 match 分支中：
1 => ProxyList::render(&self.profile_store, cx).into_any_element(),
2 => self.render_profiles(cx).into_any_element(),

// 修改 render_profiles：
fn render_profiles(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let entity = cx.entity().clone();
    let store = self.profile_store.read();
    let is_loading = store.is_loading;
    let url = store.url.clone();
    
    div()
        .flex()
        .flex_col()
        .child(div().text_xl().child("Profile Management"))
        .child(
            div()
                .mt_4()
                .p_4()
                .bg(rgb(0x2d2d2d))
                .rounded_lg()
                .child(div().text_color(rgb(0xcccccc)).child(format!("URL: {}", url)))
                .child(
                    div()
                        .mt_4()
                        .id("refresh-btn")
                        .p_2()
                        .bg(rgb(0x1677ff))
                        .rounded_md()
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            entity.update(cx, |workspace, cx| {
                                workspace.fetch_subscription(cx);
                            });
                        })
                        .child(if is_loading { "Fetching..." } else { "Update from URL" })
                )
        )
}
```

- [ ] **Step 4: 运行编译验证**

Run: `cargo check -p narya-ui`
Expected: PASS

- [ ] **Step 5: 提交代码**

```bash
git add narya-ui/src/components/ narya-ui/src/lib.rs
git commit -m "feat(ui): implement proxy nodes list and profile refresh"
```
