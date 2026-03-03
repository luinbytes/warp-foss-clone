//! Configuration management

pub mod keybindings;
pub mod settings;
pub mod theme;

// Re-exports are kept for future use when config system is fully integrated
#[allow(unused_imports)]
pub use keybindings::{Action, KeyCombo, Keybindings, Modifier};
#[allow(unused_imports)]
pub use settings::{Config, FontConfig, TerminalConfig, WindowConfig};
#[allow(unused_imports)]
pub use theme::{AnsiColors, Color, Theme, ThemeConfig};
