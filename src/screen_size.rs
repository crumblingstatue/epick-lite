use egui::Rect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq)]
pub enum ScreenSize {
    Phone(f32, f32),
    Tablet(f32, f32),
    Laptop(f32, f32),
    Desktop(f32, f32),
}

impl From<Rect> for ScreenSize {
    fn from(screen: Rect) -> Self {
        match screen.width().round() as u32 {
            0..=480 => ScreenSize::Phone(screen.width(), screen.height()),
            481..=768 => ScreenSize::Tablet(screen.width(), screen.height()),
            769..=992 => ScreenSize::Laptop(screen.width(), screen.height()),
            _ => ScreenSize::Desktop(screen.width(), screen.height()),
        }
    }
}
