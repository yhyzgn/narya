# Narya Project Memories

## Architecture
- **Workspace Mode**: Cargo workspace with crates: `api`, `config`, `narya-core`, `daemon`, `platform`, and `utils`.
- **Main Binary**: Located in `src/main.rs`.
- **FFI Layer**: `narya-core` defines the C-ABI interface for Sing-box integration.

## Performance & Optimization
- **Allocator**: `MiMalloc` is used as the global allocator.
- **Runtime**: Single-threaded Tokio runtime (`flavor = "current_thread"`) to minimize thread stack overhead.
- **Process Tracking**: `sysinfo` is configured for lazy, targeted refreshes of process paths only (`ProcessRefreshKind`) to keep RSS under 10MB in release mode (current baseline: ~8.8MB).

## Progress & Status
- **Phase 1 & 2 complete**: Infrastructure, IPC Server, Config Diffing, and Subscription Parser (Clash/Base64) are implemented and tested.
- **Mocking**: Sing-box core is currently mocked in `narya-core` for development.
