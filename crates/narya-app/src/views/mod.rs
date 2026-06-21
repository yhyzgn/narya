pub mod app_shell;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActiveView {
    Dashboard,
    Nodes,
    Connections,
    Rules,
    Subscriptions,
    Config,
    Logs,
    Tools,
    Settings,
    About,
}
