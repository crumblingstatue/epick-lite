use crate::color::NamedPalette;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Palettes {
    pub palettes: Vec<NamedPalette>,
    current_idx: usize,
}

impl Default for Palettes {
    fn default() -> Self {
        Self::new(NamedPalette::default())
    }
}

impl Palettes {
    pub const FILE_NAME: &'static str = "palettes.ron";

    pub fn new(palette: NamedPalette) -> Self {
        Self {
            palettes: vec![palette],
            current_idx: 0,
        }
    }

    pub fn current_idx(&self) -> usize {
        self.current_idx
    }

    pub fn current(&self) -> &NamedPalette {
        self.palettes.get(self.current_idx).unwrap()
    }

    pub fn current_mut(&mut self) -> &mut NamedPalette {
        self.palettes.get_mut(self.current_idx).unwrap()
    }

    pub fn len(&self) -> usize {
        self.palettes.len()
    }

    /// Moves current index to the previous palette
    pub fn prev(&mut self) {
        if self.current_idx > 0 {
            self.current_idx -= 1;
        }
    }

    pub fn move_to_name(&mut self, name: impl AsRef<str>) {
        let name = name.as_ref();
        if let Some(idx) = self.palettes.iter().position(|p| p.name == name) {
            self.current_idx = idx;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &NamedPalette> {
        self.palettes.iter()
    }

    pub fn append_empty(&mut self) {
        use std::fmt::Write as _;
        let mut palette = NamedPalette::default();
        let _ = write!(palette.name, "{}", self.len() - 1);
        self.add(palette);
    }

    pub fn add(&mut self, palette: NamedPalette) -> bool {
        if !self.palettes.iter().any(|p| p.name == palette.name) {
            self.palettes.push(palette);
            return true;
        }
        false
    }

    pub fn remove_pos(&mut self, i: usize) -> Option<NamedPalette> {
        if i < self.palettes.len() {
            let removed = self.palettes.remove(i);
            if self.palettes.is_empty() {
                self.palettes.push(NamedPalette::default());
                self.current_idx = 0;
            }
            if i <= self.current_idx {
                self.prev();
            }
            Some(removed)
        } else {
            None
        }
    }

    pub fn remove(&mut self, palette: &NamedPalette) -> Option<NamedPalette> {
        self.palettes
            .iter()
            .position(|p| p == palette)
            .and_then(|i| self.remove_pos(i))
    }

    /// Loads the saved colors from the specified file located at `path`. The file is expected to
    /// be a valid ron file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let data = fs::read_to_string(path).context("failed to read palette file")?;
        ron::from_str(&data).context("failed to deserialize palette file")
    }

    /// Saves this colors as ron file in the provided `path`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut data = String::with_capacity(128);
        ron::ser::to_writer(&mut data, &self).context("failed to serialize palette file")?;
        fs::write(path, &data).context("failed to write palette to file")
    }

    /// Returns system directory where saved colors should be placed joined by the `name` parameter.
    pub fn dir(name: impl AsRef<str>) -> Option<PathBuf> {
        let name = name.as_ref();

        if let Some(dir) = dirs::config_dir() {
            return Some(dir.join(name));
        }

        if let Some(dir) = dirs::home_dir() {
            return Some(dir.join(name));
        }

        None
    }
}

impl std::ops::Index<usize> for Palettes {
    type Output = NamedPalette;

    fn index(&self, index: usize) -> &Self::Output {
        &self.palettes[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{Color, Palette, Rgb};
    const C1: crate::color::Color = Color::Rgb(Rgb::new_unchecked(0., 0., 0.));
    const C2: crate::color::Color = Color::Rgb(Rgb::new_unchecked(0., 1., 0.));
    const C3: crate::color::Color = Color::Rgb(Rgb::new_unchecked(1., 0., 1.));

    fn test_palettes() -> (NamedPalette, NamedPalette, NamedPalette, NamedPalette) {
        let p1 = NamedPalette {
            palette: Palette::from_iter([C1]),
            name: "p1".into(),
        };
        let p2 = NamedPalette {
            palette: Palette::from_iter([C1, C2]),
            name: "p2".into(),
        };
        let p3 = NamedPalette {
            palette: Palette::from_iter([C1, C2, C3]),
            name: "p3".into(),
        };
        let p4 = NamedPalette {
            palette: Palette::from_iter([C3]),
            name: "p4".into(),
        };
        (p1, p2, p3, p4)
    }

    #[test]
    fn removal() {
        let (p1, p2, p3, p4) = test_palettes();
        let mut palettes = Palettes::new(p1.clone());
        palettes.add(p1);
        palettes.add(p2);
        palettes.add(p3);
        palettes.add(p4);
    }

    #[test]
    fn addition() {
        let (p1, p2, p3, p4) = test_palettes();
        let mut palettes = Palettes::new(p1.clone());
        palettes.add(p1);
        palettes.add(p2);
        palettes.add(p3);
        palettes.add(p4);
    }
}
