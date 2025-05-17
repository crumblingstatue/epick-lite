use crate::{color::Color, display_picker::DisplayPicker};
use anyhow::{Context, Result};
use egui::Color32;
use x11rb::{
    connection::Connection,
    image::Image,
    protocol::xproto::{ConnectionExt, Screen, Window},
    rust_connection::RustConnection,
};

pub trait DisplayPickerExt: DisplayPicker {
    fn flush(&self) -> Result<()>;
    fn get_image(&self, window: Window, x: i16, y: i16, width: u16, height: u16) -> Result<Image>;
    fn screen(&self) -> &Screen;
}

#[derive(Debug)]
pub struct X11Conn {
    conn: RustConnection,
    screen_num: usize,
}

impl X11Conn {
    pub fn new() -> Result<Self> {
        let (conn, screen_num) = x11rb::connect(None).context("failed to connect to x11 server")?;

        Ok(Self { conn, screen_num })
    }

    pub fn screen(&self) -> &Screen {
        &self.conn.setup().roots[self.screen_num]
    }

    pub fn flush(&self) -> Result<()> {
        self.conn
            .flush()
            .context("failed to flush connection")
            .map(|_| ())
    }

    pub fn get_image(
        &self,
        window: Window,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Result<Image> {
        Image::get(&self.conn, window, x, y, width, height)
            .map(|tup| tup.0)
            .context("failed to get image")
    }

    pub fn get_cursor_xy(&self, window: Window) -> Result<(i16, i16)> {
        self.conn
            .query_pointer(window)
            .context("connection failed")?
            .reply()
            .context("failed to query pointer")
            .map(|reply| (reply.root_x, reply.root_y))
    }

    pub fn get_color(&self, window: Window, x: i16, y: i16) -> Result<(u8, u8, u8)> {
        let img = self.get_image(window, x, y, 1, 1)?;
        let pixel = img.get_pixel(0, 0);

        let (red, green, blue);
        match img.byte_order() {
            x11rb::image::ImageOrder::LsbFirst => {
                red = (pixel >> 16) & 0xff;
                green = (pixel >> 8) & 0xff;
                blue = pixel & 0xff;
            }
            x11rb::image::ImageOrder::MsbFirst => {
                red = (pixel >> 8) & 0xff;
                green = (pixel >> 16) & 0xff;
                blue = (pixel >> 24) & 0xff;
            }
        }

        Ok((red as u8, green as u8, blue as u8))
    }

    pub fn get_color_for_screen(&self, screen: &Screen) -> Result<(u8, u8, u8)> {
        let (x, y) = self.get_cursor_xy(screen.root)?;
        self.get_color(screen.root, x, y)
    }

    pub fn get_color_for_conn(&self) -> Result<(u8, u8, u8)> {
        self.get_color_for_screen(self.screen())
    }
}

impl DisplayPicker for X11Conn {
    fn get_cursor_pos(&self) -> Result<(i32, i32)> {
        self.get_cursor_xy(self.screen().root)
            .map(|(x, y)| (x as i32, y as i32))
    }

    fn get_color_under_cursor(&self) -> Result<Color> {
        self.get_color_for_conn()
            .map(|color| Color32::from_rgb(color.0, color.1, color.2).into())
    }
}

impl DisplayPickerExt for X11Conn {
    fn flush(&self) -> Result<()> {
        self.flush()
    }
    fn get_image(&self, window: Window, x: i16, y: i16, width: u16, height: u16) -> Result<Image> {
        self.get_image(window, x, y, width, height)
    }
    fn screen(&self) -> &Screen {
        self.screen()
    }
}
