//! Bugbee terminal IDE — UX modeled on OpenCode (opencode.ai).
//!
//! Layout: Home (logo + prompt) → Session (transcript + prompt + footer + sidebar).

mod app;
mod logo;
mod security_panel;
mod theme;

pub use app::run_workspace;
pub use security_panel::SecurityPanel;
