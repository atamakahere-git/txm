use ratatui_core::{
    buffer::Buffer,
    layout::{HorizontalAlignment, Rect, Size, VerticalAlignment},
    style::Style,
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

use crate::backends::memory::{Backend as _, MemBuf, MemoryBackend};
use crate::layout_tree::LayoutNode;

#[derive(Debug, Clone)]
pub struct Math {
    mem: MemBuf,
    style: Style,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl Math {
    pub fn new(input: &str) -> Result<Self, crate::ParseError> {
        let tree = crate::layout(input)?;
        let backend = MemoryBackend::new();
        let mem = backend.render(&tree).unwrap();
        Ok(Self {
            mem,
            style: Style::default(),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        })
    }

    pub fn from_tree(tree: &LayoutNode) -> Self {
        let backend = MemoryBackend::new();
        let mem = backend.render(tree).unwrap();
        Self {
            mem,
            style: Style::default(),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    pub fn size(&self) -> Size {
        Rect::new(0, 0, self.mem.width as u16, self.mem.height as u16).as_size()
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn horizontal_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    pub fn vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}

impl Widget for &Math {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let render_width = self.mem.width as u16;
        let render_height = self.mem.height as u16;
        if render_width == 0 || render_height == 0 {
            return;
        }

        for y in area.y..area.y.saturating_add(area.height) {
            for x in area.x..area.x.saturating_add(area.width) {
                buf[(x, y)].reset();
            }
        }

        let (content_x, draw_x, visible_width) =
            align_horizontal_span(render_width, area.width, self.horizontal_alignment);
        let (content_y, draw_y, visible_height) =
            align_vertical_span(render_height, area.height, self.vertical_alignment);

        for row in 0..visible_height as usize {
            let src_y = content_y as usize + row;
            if src_y >= self.mem.height {
                break;
            }

            let target_row = draw_y as usize + row;
            if target_row >= area.height as usize {
                break;
            }

            let mut col = 0u16;
            let mut byte_start = 0usize;

            for (x_idx, cell) in self.mem.cells[src_y].iter().enumerate() {
                let x = area.x.saturating_add(draw_x).saturating_add(col);
                let y = area.y.saturating_add(target_row as u16);

                if x >= area.x + area.width {
                    break;
                }

                let cell_str = cell.ch.to_string();
                let cell_width = UnicodeWidthStr::width(cell_str.as_str()) as u16;

                if x + cell_width > area.x + area.width {
                    break;
                }

                buf.set_stringn(x, y, &cell_str, cell_width as usize, self.style);
                col += cell_width;
            }
        }
    }
}

fn slice_by_width(line: &str, start: u16, width: u16) -> &str {
    if width == 0 {
        return "";
    }

    let start = usize::from(start);
    let end = start.saturating_add(usize::from(width));
    let mut col = 0usize;
    let mut start_byte = 0usize;
    let mut end_byte = line.len();

    for (byte_idx, ch) in line.char_indices() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        let next_col = col.saturating_add(ch_width);

        if next_col <= start {
            start_byte = byte_idx + ch.len_utf8();
        } else if col < start {
            start_byte = byte_idx + ch.len_utf8();
        }

        if col >= end {
            end_byte = byte_idx;
            break;
        }

        if next_col > end {
            end_byte = byte_idx;
            break;
        }

        if next_col == end {
            end_byte = byte_idx + ch.len_utf8();
            break;
        }

        col = next_col;
    }

    if start_byte >= end_byte {
        ""
    } else {
        &line[start_byte.min(line.len())..end_byte.min(line.len())]
    }
}

fn align_horizontal_span(
    content: u16,
    area: u16,
    alignment: HorizontalAlignment,
) -> (u16, u16, u16) {
    let visible = content.min(area);

    if content <= area {
        let draw = match alignment {
            HorizontalAlignment::Left => 0,
            HorizontalAlignment::Center => (area - content) / 2,
            HorizontalAlignment::Right => area - content,
        };
        (0, draw, visible)
    } else {
        let content_start = match alignment {
            HorizontalAlignment::Left => 0,
            HorizontalAlignment::Center => (content - area) / 2,
            HorizontalAlignment::Right => content - area,
        };
        (content_start, 0, visible)
    }
}

fn align_vertical_span(content: u16, area: u16, alignment: VerticalAlignment) -> (u16, u16, u16) {
    let visible = content.min(area);

    if content <= area {
        let draw = match alignment {
            VerticalAlignment::Top => 0,
            VerticalAlignment::Center => (area - content) / 2,
            VerticalAlignment::Bottom => area - content,
        };
        (0, draw, visible)
    } else {
        let content_start = match alignment {
            VerticalAlignment::Top => 0,
            VerticalAlignment::Center => (content - area) / 2,
            VerticalAlignment::Bottom => content - area,
        };
        (content_start, 0, visible)
    }
}

#[cfg(test)]
mod tests {
    use super::Math;
    use ratatui_core::{
        buffer::Buffer,
        layout::{HorizontalAlignment, Rect, VerticalAlignment},
        style::Style,
        widgets::Widget,
    };

    #[test]
    fn render_clears_short_rows_before_writing() {
        let tree = {
            use crate::layout_tree::{LayoutNode, NodeKind};
            use crate::style::Style as TxmStyle;
            LayoutNode {
                width: 2,
                height: 2,
                baseline: 0,
                style: TxmStyle::new(),
                kind: NodeKind::HStack {
                    children: vec![LayoutNode::from_char('a'), LayoutNode::from_char('b')],
                    spacing: 0,
                },
            }
        };

        let math = Math::from_tree(&tree);
        let area = Rect::new(0, 0, 2, 2);
        let mut buffer = Buffer::empty(area);
        for y in 0..area.height {
            for x in 0..area.width {
                buffer[(x, y)].set_symbol("x");
            }
        }

        (&math).render(area, &mut buffer);

        assert_eq!(buffer[(0, 0)].symbol(), "a");
        assert_eq!(buffer[(1, 0)].symbol(), "b");
    }

    #[test]
    fn render_clears_alignment_padding() {
        let tree = {
            use crate::layout_tree::{LayoutNode, NodeKind};
            use crate::style::Style as TxmStyle;
            LayoutNode {
                width: 1,
                height: 1,
                baseline: 0,
                style: TxmStyle::new(),
                kind: NodeKind::Text { content: vec!['a'] },
            }
        };

        let math = Math::from_tree(&tree)
            .style(Style::default())
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center);

        let area = Rect::new(0, 0, 3, 3);
        let mut buffer = Buffer::empty(area);
        for y in 0..area.height {
            for x in 0..area.width {
                buffer[(x, y)].set_symbol("x");
            }
        }

        (&math).render(area, &mut buffer);

        assert_eq!(buffer[(0, 0)].symbol(), " ");
        assert_eq!(buffer[(1, 1)].symbol(), "a");
        assert_eq!(buffer[(2, 2)].symbol(), " ");
    }
}
