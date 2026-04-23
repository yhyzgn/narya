use gpui::*;
use gpui_platform::application;

pub struct Workspace {
    selected_tab: usize,
}

impl Workspace {
    pub fn new() -> Self {
        Self { selected_tab: 0 }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .bg(rgb(0x1e1e1e))
            .child(
                // 侧边栏
                div()
                    .w_64()
                    .h_full()
                    .bg(rgb(0x252526))
                    .flex()
                    .flex_col()
                    .child(self.render_tab(0, "Dashboard"))
                    .child(self.render_tab(1, "Proxies"))
                    .child(self.render_tab(2, "Profiles"))
                    .child(self.render_tab(3, "Settings"))
            )
            .child(
                // 主内容区
                div()
                    .flex_1()
                    .h_full()
                    .p_4()
                    .text_color(rgb(0xffffff))
                    .child(match self.selected_tab {
                        0 => "Welcome to Narya Dashboard",
                        1 => "Proxy Nodes List",
                        2 => "Profile Management",
                        3 => "App Settings",
                        _ => "Under Construction",
                    })
            )
    }
}

impl Workspace {
    fn render_tab(&self, index: usize, label: &'static str) -> impl IntoElement {
        let is_selected = self.selected_tab == index;
        
        div()
            .p_2()
            .m_1()
            .rounded_md()
            .bg(if is_selected { rgb(0x37373d) } else { rgb(0x1e1e1e) })
            .text_color(if is_selected { rgb(0xffffff) } else { rgb(0xcccccc) })
            .child(label)
    }
}

pub fn run_app() {
    application().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions::default(),
            |_, cx| cx.new(|_| Workspace::new()),
        ).unwrap();
        
        cx.activate(true);
    });
}
