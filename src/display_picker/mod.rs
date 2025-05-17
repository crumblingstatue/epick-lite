//! High level abstraction over display connection on multiple OS

pub mod x11;
pub use x11::DisplayPickerExt;

use crate::color::Color;
use anyhow::Result;
use std::{fmt::Debug, rc::Rc};

pub trait DisplayPicker: Debug {
    fn get_cursor_pos(&self) -> Result<(i32, i32)>;
    fn get_color_under_cursor(&self) -> Result<Color>;
}

pub fn init_display_picker() -> Option<Rc<dyn DisplayPickerExt>> {
    x11::X11Conn::new()
        .ok()
        .map(|conn| Rc::new(conn) as Rc<dyn DisplayPickerExt>)
}
