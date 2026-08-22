# 常用命令记录

当前工程已初始化为 Bun + Vite + Vue + Tauri v2 + Rust workspace。

## 根目录快捷命令

```bash
bun run lint
bun run typecheck
bun run test
bun run web:build
```

## 前端 / Web UI

```bash
cd apps/desktop
bun install
bun run dev
bun run lint
bun run typecheck
bun run test
bun run build
```

## 桌面 / Tauri

```bash
cd apps/desktop
bun run tauri dev
bun run tauri build
```

> 注：如当前环境没有图形会话或系统 WebKit/WebView 依赖，`tauri dev/build` 可能需要额外系统依赖。不能伪造结果；失败时记录错误摘要。

## Rust workspace

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

## 路由 smoke（可选）

```bash
cd apps/desktop
bun run dev -- --host 127.0.0.1
# 另开终端用 Playwright / Chrome 访问 http://127.0.0.1:1420/<route>
```

## Git

```bash
git status --short
git diff --stat
git diff --check
git add -A
git commit -m "..."
git push
```
