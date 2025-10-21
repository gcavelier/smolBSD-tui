use ratatui::style::Color;

mod render;
mod types;

const INFO_COLOR: Color = Color::Indexed(208);
const SELECTED_BUTTON_FG_COLOR: Color = Color::Black;
const SELECTED_BUTTON_BG_COLOR: Color = Color::Gray;
const UNSELECTED_BUTTON_FG_COLOR: Color = Color::Gray;
const UNSELECTED_BUTTON_BG_COLOR: Color = Color::Black;
const POPUP_BORDER_COLOR: Color = Color::Indexed(74);
pub const RUNNING_VM_FG: Color = Color::Indexed(74);
pub const STOPPED_VM_FG: Color = Color::Indexed(202);
pub const INVALID_CONF_VM_FG: Color = Color::Red;
pub const STARTING_VM_FG: Color = Color::LightGreen;
pub const STOPPING_VM_FG: Color = Color::Magenta;
const ACTION_COLOR: Color = Color::Magenta;
const DEFAULT_SPACING_PADDING: u16 = 1;
pub const LOGO: &[u8; 16255] = include_bytes!("../../assets/smolBSD.png");

pub use render::render;
pub use types::Screen;
