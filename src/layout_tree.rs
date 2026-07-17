use crate::ast::BinOp;
use crate::style::Style;

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub width: usize,
    pub height: usize,
    pub baseline: usize,
    pub style: Style,
    pub kind: NodeKind,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Text {
        content: Vec<char>,
    },

    HStack {
        children: Vec<LayoutNode>,
        spacing: usize,
    },

    VStack {
        top: Box<LayoutNode>,
        bottom: Box<LayoutNode>,
        line: LineStyle,
    },

    Infix {
        lhs: Box<LayoutNode>,
        op: BinOp,
        rhs: Box<LayoutNode>,
    },

    Superscript {
        base: Box<LayoutNode>,
        exp: Box<LayoutNode>,
    },

    Subscript {
        base: Box<LayoutNode>,
        sub: Box<LayoutNode>,
    },

    BothScripts {
        base: Box<LayoutNode>,
        sub: Box<LayoutNode>,
        sup: Box<LayoutNode>,
    },

    StretchyDelim {
        inner: Box<LayoutNode>,
        left: char,
        right: char,
        fill: bool,
    },

    Accent {
        inner: Box<LayoutNode>,
        mark: char,
        stretch: bool,
    },

    Limits {
        base: Box<LayoutNode>,
        lower: Box<LayoutNode>,
        upper: Box<LayoutNode>,
    },

    Sqrt {
        inner: Box<LayoutNode>,
        index: Option<Box<LayoutNode>>,
    },

    Matrix {
        name: String,
        rows: Vec<Vec<LayoutNode>>,
    },

    Summation {
        inner: Option<Box<LayoutNode>>,
    },

    Product {
        inner: Option<Box<LayoutNode>>,
    },

    Integral {
        inner: Option<Box<LayoutNode>>,
    },

    Neg {
        inner: Box<LayoutNode>,
    },

    Prime {
        base: Box<LayoutNode>,
        count: usize,
    },

    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    None,
}

impl LayoutNode {
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            baseline: 0,
            style: Style::new(),
            kind: NodeKind::Empty,
        }
    }

    pub fn text(content: Vec<char>) -> Self {
        let width = content.len();
        Self {
            width,
            height: 1,
            baseline: 0,
            style: Style::new(),
            kind: NodeKind::Text { content },
        }
    }

    pub fn text_with_style(content: Vec<char>, style: Style) -> Self {
        let width = content.len();
        Self {
            width,
            height: 1,
            baseline: 0,
            style,
            kind: NodeKind::Text { content },
        }
    }

    pub fn from_char(c: char) -> Self {
        Self::text(vec![c])
    }

    pub fn text_str(s: &str) -> Self {
        Self::text(s.chars().collect())
    }

    pub fn hstack(children: &[LayoutNode], spacing: usize) -> Self {
        if children.is_empty() {
            return Self::empty();
        }
        if children.len() == 1 {
            return children[0].clone();
        }

        let baseline = children.iter().map(|n| n.baseline).max().unwrap_or(0);
        let height = children
            .iter()
            .map(|n| {
                let below = n.height.saturating_sub(n.baseline);
                baseline + below
            })
            .max()
            .unwrap_or(0);

        let total_width: usize =
            children.iter().map(|n| n.width).sum::<usize>() + spacing * (children.len() - 1);

        Self {
            width: total_width,
            height,
            baseline,
            style: Style::new(),
            kind: NodeKind::HStack {
                children: children.to_vec(),
                spacing,
            },
        }
    }

    pub fn vstack(top: LayoutNode, bottom: LayoutNode, line: LineStyle) -> Self {
        let inner_w = top.width.max(bottom.width);
        let pad = 1;
        let w = inner_w + 2 * pad;
        let h = top.height + bottom.height + 1;
        let baseline = top.height;

        Self {
            width: w,
            height: h,
            baseline,
            style: Style::new(),
            kind: NodeKind::VStack {
                top: Box::new(top),
                bottom: Box::new(bottom),
                line,
            },
        }
    }

    pub fn infix(lhs: LayoutNode, op: BinOp, rhs: LayoutNode) -> Self {
        let baseline = lhs.baseline.max(rhs.baseline);
        let lhs_y = baseline - lhs.baseline;
        let rhs_y = baseline - rhs.baseline;

        let width = lhs.width + 3 + rhs.width;
        let height = (lhs_y + lhs.height).max(rhs_y + rhs.height);

        Self {
            width,
            height,
            baseline,
            style: Style::new(),
            kind: NodeKind::Infix {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            },
        }
    }

    pub fn superscript(base: LayoutNode, exp: LayoutNode) -> Self {
        let height = exp.height + base.height;
        let width = exp.width + base.width;
        let baseline = base.baseline + exp.height;

        Self {
            width,
            height,
            baseline,
            style: Style::new(),
            kind: NodeKind::Superscript {
                base: Box::new(base),
                exp: Box::new(exp),
            },
        }
    }

    pub fn subscript(base: LayoutNode, sub: LayoutNode) -> Self {
        let height = (base.baseline + 1 + sub.height).max(base.height);
        let width = base.width + sub.width;

        Self {
            width,
            height,
            baseline: base.baseline,
            style: Style::new(),
            kind: NodeKind::Subscript {
                base: Box::new(base),
                sub: Box::new(sub),
            },
        }
    }

    pub fn both_scripts(base: LayoutNode, sub: LayoutNode, sup: LayoutNode) -> LayoutNode {
        let sup_h = sup.height;
        let height = sup_h + sub.height + base.height;
        let width = base.width + sub.width.max(sup.width);
        let baseline = base.baseline + sup_h;

        Self {
            width,
            height,
            baseline,
            style: Style::new(),
            kind: NodeKind::BothScripts {
                base: Box::new(base),
                sub: Box::new(sub),
                sup: Box::new(sup),
            },
        }
    }

    pub fn stretchy_delim(inner: LayoutNode, left: char, right: char, fill: bool) -> Self {
        if inner.height <= 1 {
            let w = inner.width + 2;
            return Self {
                width: w,
                height: 1,
                baseline: 0,
                style: Style::new(),
                kind: NodeKind::StretchyDelim {
                    inner: Box::new(inner),
                    left,
                    right,
                    fill,
                },
            };
        }

        let h = inner.height;
        let w = inner.width + 4;

        Self {
            width: w,
            height: h,
            baseline: inner.baseline,
            style: Style::new(),
            kind: NodeKind::StretchyDelim {
                inner: Box::new(inner),
                left,
                right,
                fill,
            },
        }
    }

    pub fn accent(inner: LayoutNode, mark: char, stretch: bool) -> Self {
        let width = inner.width.max(1);
        let height = inner.height + 1;

        Self {
            width,
            height,
            baseline: inner.baseline + 1,
            style: Style::new(),
            kind: NodeKind::Accent {
                inner: Box::new(inner),
                mark,
                stretch,
            },
        }
    }

    pub fn limits(base: LayoutNode, lower: LayoutNode, upper: LayoutNode) -> Self {
        let max_h = upper.height.max(lower.height);
        let w = base.width.max(lower.width).max(upper.width) + 1;
        let h = base.height + 2 * max_h;

        Self {
            width: w,
            height: h,
            baseline: max_h + base.baseline,
            style: Style::new(),
            kind: NodeKind::Limits {
                base: Box::new(base),
                lower: Box::new(lower),
                upper: Box::new(upper),
            },
        }
    }

    pub fn sqrt(inner: LayoutNode, index: Option<LayoutNode>) -> Self {
        let h = inner.height + 1;
        let w = inner.width + 3;
        let baseline = inner.baseline + 1;

        Self {
            width: w,
            height: h,
            baseline,
            style: Style::new(),
            kind: NodeKind::Sqrt {
                inner: Box::new(inner),
                index: index.map(Box::new),
            },
        }
    }

    pub fn summation(inner: Option<LayoutNode>) -> Self {
        match &inner {
            None => Self {
                width: 4,
                height: 3,
                baseline: 1,
                style: Style::new(),
                kind: NodeKind::Summation { inner: None },
            },
            Some(inner) if inner.height <= 2 => {
                let w = inner.width + 4;
                Self {
                    width: w,
                    height: 3,
                    baseline: 1,
                    style: Style::new(),
                    kind: NodeKind::Summation {
                        inner: Some(Box::new(inner.clone())),
                    },
                }
            }
            Some(inner) => {
                let h = inner.height;
                let w_sigma = ((1.5 * h as f32) as usize).max(h / 2 + 2);
                let w = w_sigma + 1 + inner.width;
                Self {
                    width: w,
                    height: h,
                    baseline: inner.baseline,
                    style: Style::new(),
                    kind: NodeKind::Summation {
                        inner: Some(Box::new(inner.clone())),
                    },
                }
            }
        }
    }

    pub fn product(inner: Option<LayoutNode>) -> Self {
        match &inner {
            None => Self {
                width: 4,
                height: 2,
                baseline: 1,
                style: Style::new(),
                kind: NodeKind::Product { inner: None },
            },
            Some(inner) => {
                let w = inner.width + 4;
                let h = inner.height.max(1) + 1;
                Self {
                    width: w,
                    height: h,
                    baseline: inner.baseline + 1,
                    style: Style::new(),
                    kind: NodeKind::Product {
                        inner: Some(Box::new(inner.clone())),
                    },
                }
            }
        }
    }

    pub fn integral(inner: Option<LayoutNode>) -> Self {
        match &inner {
            None => Self {
                width: 2,
                height: 3,
                baseline: 1,
                style: Style::new(),
                kind: NodeKind::Integral { inner: None },
            },
            Some(inner) if inner.height <= 3 => {
                let w = inner.width + 1;
                Self {
                    width: w,
                    height: 3,
                    baseline: 1,
                    style: Style::new(),
                    kind: NodeKind::Integral {
                        inner: Some(Box::new(inner.clone())),
                    },
                }
            }
            Some(inner) => {
                let h = inner.height;
                let w = inner.width + 2;
                Self {
                    width: w,
                    height: h,
                    baseline: inner.baseline,
                    style: Style::new(),
                    kind: NodeKind::Integral {
                        inner: Some(Box::new(inner.clone())),
                    },
                }
            }
        }
    }

    pub fn negate(inner: LayoutNode) -> Self {
        let width = inner.width + 1;
        let height = inner.height;
        let baseline = inner.baseline;

        Self {
            width,
            height,
            baseline,
            style: Style::new(),
            kind: NodeKind::Neg {
                inner: Box::new(inner),
            },
        }
    }

    pub fn prime(base: LayoutNode, count: usize) -> Self {
        Self {
            width: base.width + count,
            height: base.height,
            baseline: base.baseline,
            style: Style::new(),
            kind: NodeKind::Prime {
                base: Box::new(base),
                count,
            },
        }
    }

    pub fn matrix(
        name: &str,
        rows: &[Vec<LayoutNode>],
    ) -> Result<LayoutNode, crate::error::ParseError> {
        if rows.is_empty() || rows[0].is_empty() {
            return Ok(Self::empty());
        }

        let num_rows = rows.len();
        let num_cols = rows[0].len();

        let mut row_max_depths = vec![0; num_rows];
        let mut row_max_baselines = vec![0; num_rows];
        let mut max_item_width = 0;

        for (i, row) in rows.iter().enumerate() {
            let mut max_b = 0;
            let mut max_d = 0;
            for item in row {
                max_item_width = max_item_width.max(item.width);
                max_b = max_b.max(item.baseline);
                max_d = max_d.max(item.height.saturating_sub(item.baseline));
            }
            row_max_baselines[i] = max_b;
            row_max_depths[i] = max_d;
        }

        let cell_width = max_item_width;
        let mut cell_height = 0;
        for i in 0..num_rows {
            let row_content_height = row_max_baselines[i] + row_max_depths[i];
            cell_height = cell_height.max(row_content_height);
        }

        let hspacing = 4;
        let vspacing = 1;

        let mut matrix_layout_height = num_rows * cell_height + (num_rows - 1) * vspacing;
        let matrix_layout_width = num_cols * cell_width + (num_cols - 1) * hspacing;

        if matrix_layout_height.is_multiple_of(2) {
            matrix_layout_height += 1;
        }

        let baseline = matrix_layout_height / 2;

        let (left_delim, right_delim) = match name {
            "matrix" => (' ', ' '),
            "bmatrix" => ('[', ']'),
            "pmatrix" => ('(', ')'),
            _ => {
                return Err(crate::error::ParseError(format!(
                    "unknown matrix environment: {name}"
                )));
            }
        };

        let inner = Self {
            width: matrix_layout_width,
            height: matrix_layout_height,
            baseline,
            style: Style::new(),
            kind: NodeKind::Matrix {
                name: name.to_string(),
                rows: rows.to_vec(),
            },
        };

        Ok(Self::stretchy_delim(inner, left_delim, right_delim, true))
    }
}
