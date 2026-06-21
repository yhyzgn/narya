#![recursion_limit = "2048"]
pub mod assets;
pub mod ipc;
pub mod state;
pub mod ui_kit;
pub mod views;

use crate::assets::Assets;
use crate::views::app_shell::AppShell;
use gpui::*;

pub fn run() {
    gpui::Application::new()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            // Initialize System Tray (Skeleton)
            #[cfg(not(target_os = "linux"))]
            // Tray icon can be tricky on Linux in some environments
            let _tray = init_tray();

            liora::init_liora(cx);
            AppShell::open(cx);
            cx.activate(true);
        });
}

#[cfg(not(target_os = "linux"))]
fn init_tray() -> Option<tray_icon::TrayIcon> {
    use tray_icon::{menu::Menu, TrayIconBuilder};
    let menu = Menu::new();
    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Narya")
        .build()
        .ok()
}
