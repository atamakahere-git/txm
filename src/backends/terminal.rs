use std::fmt;

use crate::backend::Backend;
use crate::backend::RenderTarget;
use crate::layout_tree::LayoutNode;
use crate::style::Style;

use super::generic_backend;

pub struct TerminalBackend;

impl TerminalBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TerminalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for TerminalBackend {
    type Output = String;
    type Error = fmt::Error;

    fn render(&self, tree: &LayoutNode) -> Result<String, fmt::Error> {
        let mut buf = CharBuf::new(tree.width, tree.height);
        generic_backend::render_node(tree, &mut buf, 0, 0);
        Ok(buf.to_string())
    }
}

struct CharBuf {
    data: Vec<char>,
    styles: Vec<Style>,
    width: usize,
    height: usize,
}

impl CharBuf {
    fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![' '; width * height],
            styles: vec![Style::new(); width * height],
            width,
            height,
        }
    }
}

impl fmt::Display for CharBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            let row_start = y * self.width;
            for x in 0..self.width {
                let i = row_start + x;
                let style = self.styles[i];

                if !style.is_empty() {
                    style.write_ansi_prefix(f)?;
                    write!(f, "{}", self.data[i])?;
                    write!(f, "\x1b[0m")?;
                } else {
                    write!(f, "{}", self.data[i])?;
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl RenderTarget for CharBuf {
    fn set(&mut self, x: usize, y: usize, ch: char, style: Style) {
        if x < self.width && y < self.height {
            let i = y * self.width + x;
            self.data[i] = ch;
            self.styles[i] = style;
        }
    }

    fn fill_row(&mut self, y: usize, x_start: usize, x_end: usize, ch: char, style: Style) {
        for x in x_start..x_end.min(self.width) {
            self.set(x, y, ch, style);
        }
    }
}
