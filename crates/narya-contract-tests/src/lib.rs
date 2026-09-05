pub fn workspace_root() -> &'static std::path::Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
}

pub fn workspace_file(path: &str) -> String {
    let root = workspace_root();
    std::fs::read_to_string(root.join(path))
        .unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{workspace_file, workspace_root};

    #[test]
    fn ui_contract_launches_main_window_without_splash() {
        let lib = workspace_file("crates/narya-app/src/lib.rs");
        assert!(
            lib.contains("liora::init_liora_with_options(cx, options)")
                && lib.contains("liora::FontConfig::system()"),
            "narya-app must initialize Liora with the platform system font policy during GPUI startup"
        );
        let ui_kit = workspace_file("crates/narya-app/src/ui_kit.rs");
        assert!(
            ui_kit.contains("family: \"Consolas\".into()")
                && ui_kit.contains("\"LXGW WenKai\".to_string()"),
            "the UI font chain must preserve the configured Consolas/LXGW WenKai pairing"
        );
        let assets = workspace_file("crates/narya-app/src/assets.rs");
        assert!(
            assets.contains("IconAssetSource"),
            "application assets must delegate virtual Liora icon paths to IconAssetSource"
        );
        assert!(
            !lib.contains("views::splash::Splash") && !lib.contains("Splash::new"),
            "startup must not open the legacy splash window"
        );
        assert!(
            lib.contains("AppShell::open(cx)"),
            "startup must open the main AppShell directly"
        );
        let ipc = workspace_file("crates/narya-app/src/ipc.rs");
        assert!(
            ipc.contains("ensure_daemon") && ipc.contains("narya-daemon"),
            "desktop startup must provision the co-located daemon instead of requiring a silent manual prerequisite"
        );
        let views_mod = workspace_file("crates/narya-app/src/views/mod.rs");
        assert!(
            !views_mod.contains("pub mod splash"),
            "legacy splash module should not be part of the compiled app surface"
        );
    }

    #[test]
    fn ui_contract_declares_liora_dependency_and_registry_gpui() {
        let app_cargo = workspace_file("crates/narya-app/Cargo.toml");
        let root_cargo = workspace_file("Cargo.toml");
        assert!(
            app_cargo.contains("liora"),
            "narya-app must depend on the Liora facade crate"
        );
        assert!(
            app_cargo.contains("default-features = false") || app_cargo.contains("liora = \""),
            "Liora should be declared intentionally instead of hidden through transitive dependencies"
        );
        assert!(
            root_cargo.contains("gpui = { version = \"0.2.2\"")
                && !root_cargo.contains("gpui = { git"),
            "workspace GPUI must be aligned with Liora's registry gpui 0.2.2"
        );
        assert!(
            !root_cargo.contains("gpui_platform"),
            "Liora/GPUI 0.2.2 apps should use gpui::Application instead of gpui_platform"
        );
    }

    #[test]
    fn ui_contract_has_project_component_boundary() {
        let lib = workspace_file("crates/narya-app/src/lib.rs");
        assert!(
            lib.contains("pub mod ui_kit;"),
            "narya-app must expose a project-local Liora wrapper layer"
        );
        let ui_kit = workspace_file("crates/narya-app/src/ui_kit.rs");
        for symbol in ["NaryaCard", "NaryaButton", "NaryaMetric", "NaryaPage"] {
            assert!(ui_kit.contains(symbol), "ui_kit.rs must define {symbol}");
        }
        assert!(
            !ui_kit.contains(".flex()"),
            "ui_kit layout must be delegated to Liora components instead of raw GPUI flex calls"
        );
        assert!(
            ui_kit.contains("liora::components") || ui_kit.contains("liora_components"),
            "project components must wrap Liora components, not bypass the component library"
        );
        let app_shell = workspace_file("crates/narya-app/src/views/app_shell.rs");
        for symbol in ["NaryaCard", "NaryaButton", "NaryaMetric", "NaryaPage"] {
            assert!(
                app_shell.contains(symbol),
                "main shell must compose through {symbol}"
            );
        }
    }

    #[test]
    fn ui_contract_keeps_primary_controls_interactive() {
        let ui_kit = workspace_file("crates/narya-app/src/ui_kit.rs");
        let app_shell = workspace_file("crates/narya-app/src/views/app_shell.rs");
        assert!(
            !ui_kit.contains("readonly_shell") && !ui_kit.contains("interaction_blocker"),
            "primary Liora controls must not be wrapped in a mouse-blocking readonly layer"
        );
        assert!(
            ui_kit.contains(".on_click(move |_, _, cx| on_nav(target, cx))"),
            "sidebar navigation must dispatch through a direct stable row click handler"
        );
        assert!(
            ui_kit.contains(".when(active, |style| style.bg(color(0xEEF4FF)))"),
            "sidebar inactive rows must remain transparent instead of using opaque black RGB"
        );
        assert!(
            ui_kit.contains(".bg(color(0xFBFDFF))"),
            "sidebar rows must paint an explicit light base to avoid black compositor surfaces"
        );
        assert!(
            ui_kit.contains("WindowDecorations::Server"),
            "window controls must remain owned by the native window manager"
        );
        assert!(
            !ui_kit.contains("fn interaction_blocker()"),
            "the application must not retain a full-size interaction blocker"
        );
        assert!(
            ui_kit.contains("category_menu_with_change")
                && ui_kit.contains("setting_row_with_change")
                && ui_kit.contains("on_select"),
            "settings navigation and switches must expose real Liora callbacks"
        );
        assert!(
            app_shell.contains("settings_category")
                && app_shell.contains("set_setting_value")
                && app_shell.contains("overflow_y_scroll"),
            "settings page must retain state and bound the kernel column"
        );
        assert!(
            app_shell.contains("category_menu: Entity<NavigationMenu>")
                && app_shell.contains("autostart: Entity<Switch>")
                && app_shell.contains("appearance: Entity<Segmented>")
                && app_shell.contains("SettingsPage::new")
                && app_shell.contains("settings.category_menu.clone()"),
            "settings controls must use persistent Liora entities and SettingsPage layout"
        );
    }

    #[test]
    fn integration_contract_red_line_fixes_are_locked() {
        let app_shell = workspace_file("crates/narya-app/src/views/app_shell.rs");
        for callback in ["AppState::toggle_proxy", "AppState::connect_node"] {
            assert!(
                app_shell.contains(callback),
                "main Liora shell must wire visible connection actions through {callback}"
            );
        }
        let daemon = workspace_file("crates/narya-daemon/src/main.rs");
        assert!(
            daemon.contains("InstallOfficialKernel")
                && daemon.contains("UpgradeOfficialKernel")
                && daemon.contains("UninstallKernel")
                && workspace_file("crates/narya-daemon/src/installer.rs")
                    .contains("KernelArtifactRequest")
                && workspace_file("crates/narya-daemon/src/installer.rs")
                    .contains("managed storage")
                && workspace_file("crates/narya-daemon/src/kernel.rs")
                    .contains("cannot uninstall the active kernel"),
            "kernel install and upgrade must require a verified artifact request"
        );
        assert!(
            daemon.contains("read_frame")
                && daemon.contains("write_frame")
                && daemon.contains("SetRoutingMode")
                && daemon.contains("GetRoutingStatus")
                && daemon.contains("preflight_tun")
                && workspace_file("crates/narya-daemon/src/kernel.rs")
                    .contains("wait_for_configured_listeners")
                && workspace_file("crates/narya-daemon/src/kernel.rs")
                    .contains("reachable == targets.len()"),
            "daemon IPC must use framing, validated routing state, and complete kernel listener readiness"
        );

        let config_gen = workspace_file("crates/narya-daemon/src/config_gen.rs");
        assert!(
            config_gen.contains("unsupported proxy protocol")
                && config_gen.contains("split_shadowsocks_credentials"),
            "config generation must fail closed and split Shadowsocks method/password"
        );
        assert!(
            !config_gen.contains("\"password\": \"password\"")
                && !config_gen
                    .contains("\"type\": \"direct\",\n                    \"tag\": \"proxy\""),
            "config generation must not use placeholder passwords or direct proxy fallback"
        );

        let app_state = workspace_file("crates/narya-app/src/state.rs");
        assert!(
            app_state.contains("response.error.is_none()")
                && app_state.contains("StartKernel")
                && app_state.contains("SetRoutingMode 失败"),
            "app state must inspect daemon IpcResponse.error before reporting connected state"
        );
        assert!(
            app_state.contains("InstallOfficialKernel")
                && app_state.contains("UpgradeOfficialKernel")
                && app_state.contains("UninstallKernel")
                && !app_state.contains("Kernel installation is not implemented"),
            "kernel settings must submit real install/upgrade IPC requests"
        );
        assert!(
            app_state.contains("export_rules")
                && app_state.contains("import_rules")
                && app_state.contains("set_rule_condition")
                && app_state.contains("set_group_strategy")
                && app_state.contains("set_rule_set_enabled"),
            "routing workbench must support validated import/export, AND conditions, group strategies, and ruleset lifecycle"
        );
        assert!(
            app_state.contains("仍被规则引用") && app_state.contains("仍被规则引用，请先修改"),
            "deleting referenced rule sets or groups must fail closed instead of silently retargeting"
        );
        let installer = workspace_file("crates/narya-daemon/src/installer.rs");
        assert!(
            installer.contains("Ed25519") && installer.contains("signature verification"),
            "HTTPS kernel artifacts must be authenticated, not checksum-only"
        );
        let ruleset_cache = workspace_file("crates/narya-daemon/src/ruleset_cache.rs");
        assert!(
            ruleset_cache.contains("verify_bytes")
                && ruleset_cache.contains("fs::rename")
                && ruleset_cache.contains("Signature"),
            "remote rulesets must be verified and atomically cached by the daemon"
        );
        let rules = workspace_file("crates/narya-rules/src/lib.rs");
        let app_shell = workspace_file("crates/narya-app/src/views/app_shell.rs");
        assert!(
            rules.contains("default_rule_set_enabled")
                && app_shell.contains("RuleSetToggle")
                && app_shell.contains("Switch")
                && rules.contains("RuleSetFormat")
                && workspace_file("crates/narya-daemon/src/config_gen/mihomo.rs")
                    .contains("rule_providers"),
            "ruleset lifecycle and cross-kernel provider formats must be explicit and exposed through Liora"
        );
        let catalog = workspace_file("crates/narya-daemon/src/kernel_catalog.rs");
        let daemon = workspace_file("crates/narya-daemon/src/main.rs");
        assert!(
            catalog.contains("canonical_payload")
                && catalog.contains("local trust root")
                && catalog.contains("find_entry")
                && daemon.contains("RefreshKernelCatalog")
                && daemon.contains("GetKernelCatalog"),
            "kernel HTTPS installs must be constrained by a verified signed catalog"
        );
        assert!(
            workspace_file("crates/narya-app/src/state.rs")
                .contains("InstallOfficialKernel")
                && !workspace_file("crates/narya-app/src/views/app_shell.rs")
                    .contains("KernelArtifactForm")
                && !workspace_file("crates/narya-app/src/views/app_shell.rs")
                    .contains("KernelCatalogForm"),
            "settings page must use official catalog actions without exposing artifact or catalog forms"
        );
        for action in ["install", "upgrade", "uninstall", "start"] {
            assert!(
                app_shell.contains(&format!("narya-kernel-{{name}}-{action}")),
                "each kernel row must assign a unique stable id to its {action} button"
            );
        }
        let official_release = workspace_file("crates/narya-daemon/src/official_release.rs");
        for repository in ["SagerNet", "MetaCubeX", "XTLS", "v2fly"] {
            assert!(
                official_release.contains(repository),
                "official installer must pin the {repository} release repository"
            );
        }
        assert!(
            official_release.contains("release-assets.githubusercontent.com")
                && official_release.contains("extract_binary")
                && official_release.contains("MAX_DOWNLOAD_SIZE"),
            "official installer must constrain redirects, extraction, and download size"
        );

        let ipc = workspace_file("crates/narya-ipc/src/lib.rs");
        assert!(
            ipc.contains("XDG_RUNTIME_DIR")
                && ipc.contains("socket_path()")
                && ipc.contains("FrameDecoder")
                && ipc.contains("MAX_FRAME_SIZE")
                && !ipc.contains("/tmp/narya.sock"),
            "IPC paths must use a per-user runtime directory and framed messages"
        );
        let kernel = workspace_file("crates/narya-kernel/src/lib.rs");
        assert!(
            kernel.contains("probe_managed")
                && !kernel.contains("split_paths")
                && !kernel.contains("find_on_path"),
            "kernel discovery must be limited to Narya managed storage"
        );
    }

    #[test]
    fn ui_specs_are_image_only_and_page_layer_has_no_raw_gpui_layout() {
        let root = workspace_root();
        let ui_root = root.join("ui");
        if ui_root.exists() {
            for entry in walkdir::WalkDir::new(ui_root) {
                let entry = entry.expect("walk ui");
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                let path_string = path.strip_prefix(root).unwrap().display().to_string();
                let lower = path_string.to_ascii_lowercase();
                let is_image = [".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg"]
                    .iter()
                    .any(|suffix| lower.ends_with(suffix));
                assert!(
                    !lower.contains("spec") || is_image,
                    "spec-related UI artifacts must be removed unless they are images: {path_string}"
                );
            }
        }

        for page in ["crates/narya-app/src/views/app_shell.rs"] {
            let source = workspace_file(page);
            for forbidden in [
                "use gpui::",
                "gpui::{",
                "div()",
                ".flex()",
                ".bg(",
                ".border_color(",
                ".text_color(",
                ".padding_",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "page layer {page} must not use raw GPUI/layout styling token `{forbidden}`; use Liora or narya_ui wrappers"
                );
            }
            assert!(
                source.contains("narya_ui") || source.contains("crate::ui_kit"),
                "page layer {page} must compose through the local reusable Narya UI layer"
            );
            assert!(
                source.contains("toggle_routing_mode") && source.contains("switch_routing_mode"),
                "dashboard routing cards must dispatch mode changes through the daemon transaction"
            );
        }
    }
}
