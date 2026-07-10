use std::fmt;

#[derive(Debug, Clone)]
pub struct RenderNode {
    pub width: usize,
    pub height: usize,
    pub baseline: usize,
    pub data: Vec<char>,
}

impl RenderNode {
    pub fn new(width: usize, height: usize, baseline: usize) -> Self {
        Self {
            width,
            height,
            baseline,
            data: vec![' '; width * height],
        }
    }

    pub fn from_char(c: char) -> Self {
        Self {
            width: 1,
            height: 1,
            baseline: 0,
            data: vec![c],
        }
    }

    pub fn from_str(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        let width = chars.len();
        Self {
            width,
            height: 1,
            baseline: 0,
            data: chars,
        }
    }

    pub fn blit_into(&self, target: &mut [char], target_width: usize, x: usize, y: usize) {
        for row in 0..self.height {
            if y + row >= target.len() / target_width {
                break;
            }

            let src_start = row * self.width;
            let src_end = src_start + self.width;
            let dst_start = (y + row) * target_width + x;

            if dst_start + self.width <= target.len() {
                target[dst_start..dst_start + self.width]
                    .copy_from_slice(&self.data[src_start..src_end]);
            }
        }
    }

    pub fn hstack(nodes: &[Self], spacing: usize) -> Self {
        if nodes.is_empty() {
            return Self::new(0, 0, 0);
        }

        if nodes.len() == 1 {
            return nodes[0].clone();
        }

        let baseline = nodes.iter().map(|n| n.baseline).max().unwrap();
        let height = nodes
            .iter()
            .map(|n| {
                let below = n.height.saturating_sub(n.baseline);
                baseline + below
            })
            .max()
            .unwrap();

        let total_width: usize =
            nodes.iter().map(|n| n.width).sum::<usize>() + spacing * (nodes.len() - 1);
        let mut data = vec![' '; total_width * height];

        let mut x = 0;
        for node in nodes {
            let y = baseline.saturating_sub(node.baseline);
            node.blit_into(&mut data, total_width, x, y);
            x += node.width + spacing;
        }

        Self {
            width: total_width,
            height,
            baseline,
            data,
        }
    }

    pub fn vstack(top: &Self, bottom: &Self, line_char: char, pad: usize) -> Self {
        let inner_w = top.width.max(bottom.width);
        let w = inner_w + 2 * pad;
        let h = top.height + 1 + bottom.height;
        let baseline = top.height;

        let mut data = vec![' '; w * h];

        let top_x = pad + (inner_w.saturating_sub(top.width)) / 2;
        top.blit_into(&mut data, w, top_x, 0);

        let bot_x = pad + (inner_w.saturating_sub(bottom.width)) / 2;
        bottom.blit_into(&mut data, w, bot_x, baseline + 1);

        for x in 0..w {
            data[baseline * w + x] = line_char;
        }

        Self {
            width: w,
            height: h,
            baseline,
            data,
        }
    }

    pub fn infix(lhs: &Self, op: char, rhs: &Self) -> Self {
        let baseline = lhs.baseline.max(rhs.baseline);
        let lhs_y = baseline - lhs.baseline;
        let rhs_y = baseline - rhs.baseline;

        let width = lhs.width + 3 + rhs.width;
        let height = (lhs_y + lhs.height).max(rhs_y + rhs.height);

        let mut data = vec![' '; width * height];

        lhs.blit_into(&mut data, width, 0, lhs_y);
        data[baseline * width + lhs.width + 1] = op;
        rhs.blit_into(&mut data, width, lhs.width + 3, rhs_y);

        Self {
            width,
            height,
            baseline,
            data,
        }
    }

    pub fn superscript(base: &Self, exp: &Self) -> Self {
        let height = exp.height + base.height;
        let width = exp.width + base.width;
        let baseline = base.baseline + exp.height;

        let mut data = vec![' '; width * height];
        base.blit_into(&mut data, width, 0, exp.height);
        exp.blit_into(&mut data, width, base.width, 0);

        Self {
            width,
            height,
            baseline,
            data,
        }
    }

    pub fn superscript_digit(base: &Self, digit: char) -> Self {
        let sup = match digit {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            _ => return Self::superscript(base, &Self::from_char(digit)),
        };

        let width = base.width + 1;
        let mut data = vec![' '; width * base.height];
        base.blit_into(&mut data, width, 0, 0);
        data[base.width] = sup;

        Self {
            width,
            height: base.height,
            baseline: base.baseline,
            data,
        }
    }

    pub fn subscript(base: &Self, sub: &Self) -> Self {
        let sub_y = base.baseline + 1;
        let height = (sub_y + sub.height).max(base.height);
        let width = base.width + sub.width;

        let mut data = vec![' '; width * height];
        base.blit_into(&mut data, width, 0, 0);
        sub.blit_into(&mut data, width, base.width, sub_y);

        Self {
            width,
            height,
            baseline: base.baseline,
            data,
        }
    }

    pub fn both_scripts(base: &Self, sub: &Self, sup: &Self) -> Self {
        let sup_h = sup.height;
        let sub_h = sub.height;
        let sub_y = base.baseline + 1;
        let height = sup_h + sub_h + base.height;
        let width = base.width + sub.width.max(sup.width);
        let baseline = base.baseline + sup_h;

        let mut data = vec![' '; width * height];
        base.blit_into(&mut data, width, 0, sup_h);
        sup.blit_into(&mut data, width, base.width, 0);
        sub.blit_into(&mut data, width, base.width, sub_y + sup_h);

        Self {
            width,
            height,
            baseline,
            data,
        }
    }

    pub fn prime_suffix(base: &Self, n: usize) -> Self {
        let s: String = std::iter::repeat_n('\'', n).collect();
        let p = Self::from_str(&s);
        Self::hstack(&[base.clone(), p], 0)
    }

    /// If fill is true, the middle line will also use left and right chars
    pub fn stretchy_delim(inner: &Self, left: char, right: char, fill: bool) -> Self {
        // one-liner expressions
        if inner.height <= 1 {
            let mut result = Self::new(inner.width + 2, 1, 0);
            result.data[0] = left;
            inner.blit_into(&mut result.data, result.width, 1, 0);
            result.data[result.width - 1] = right;
            return result;
        }

        let h = inner.height;
        let w = inner.width + 4;
        let baseline = inner.baseline;

        let mut data = vec![' '; w * h];
        inner.blit_into(&mut data, w, 2, 0);

        let (tl, tr, bl, br, mid_l, mid_r) = match (left, right) {
            ('(', ')') => ('⎛', '⎞', '⎝', '⎠', '⎟', '⎟'),
            ('[', ']') => ('⎡', '⎤', '⎣', '⎦', '⎢', '⎢'),
            ('{', '}') => ('⎧', '⎫', '⎩', '⎭', '⎪', '⎪'),
            _ if fill => (left, right, left, right, left, right),
            _ => (left, right, left, right, '│', '│'),
        };

        data[0] = tl;
        data[w - 1] = tr;
        data[(h - 1) * w] = bl;
        data[(h - 1) * w + w - 1] = br;

        for y in 1..h - 1 {
            if left == '{' && y == baseline {
                data[y * w] = '⎨';
            } else {
                data[y * w] = mid_l;
            }
            if right == '}' && y == baseline {
                data[y * w + w - 1] = '⎬';
            } else {
                data[y * w + w - 1] = mid_r;
            }
        }

        Self {
            width: w,
            height: h,
            baseline,
            data,
        }
    }

    pub fn abs(inner: &Self) -> Self {
        let h = inner.height;
        let w = inner.width + 2;
        let mut data = vec![' '; w * h];

        inner.blit_into(&mut data, w, 1, 0);

        for y in 0..h {
            data[y * w] = '│';
            data[y * w + w - 1] = '│';
        }
        Self {
            width: w,
            height: h,
            baseline: inner.baseline,
            data,
        }
    }

    pub fn sqrt_inner(inner: &Self) -> Self {
        let h = inner.height + 1;
        let w = inner.width + 3;
        let baseline = inner.baseline + 1;

        let mut data = vec![' '; w * h];
        inner.blit_into(&mut data, w, 3, 1);

        data[1] = '┌';

        for x in 2..w {
            data[x] = '─';
        }

        for y in 1..h {
            data[y * w + 1] = '│';
        }

        data[(h - 1) * w] = '╲';

        Self {
            width: w,
            height: h,
            baseline,
            data,
        }
    }

    #[allow(dead_code)]
    pub fn limits(base: &Self, lower: &Self, upper: &Self) -> Self {
        let inner_w = base.width.max(lower.width).max(upper.width);
        let h = upper.height + base.height + lower.height;
        let w = inner_w;

        let mut data = vec![' '; w * h];

        let ux = (w.saturating_sub(upper.width)) / 2;
        upper.blit_into(&mut data, w, ux, 0);

        let bx = (w.saturating_sub(base.width)) / 2;
        base.blit_into(&mut data, w, bx, upper.height);

        let lx = (w.saturating_sub(lower.width)) / 2;
        lower.blit_into(&mut data, w, lx, upper.height + base.height);

        Self {
            width: w,
            height: h,
            baseline: upper.height + base.baseline,
            data,
        }
    }

    pub fn matrix(name: &str, rendered_rows: &[Vec<RenderNode>]) -> RenderNode {
        let (left_delim, right_delim) = match name {
            "matrix" => (' ', ' '),
            "bmatrix" => ('[', ']'),
            "pmatrix" => ('(', ')'),
            _ => panic!("invalid matrix type: {name}"),
        };

        let num_rows = rendered_rows.len();
        if num_rows == 0 || rendered_rows[0].is_empty() {
            return Self::new(0, 0, 0);
        }

        let num_cols = rendered_rows[0].len();

        // depth = height - baseline
        let mut row_max_depths = vec![0; num_rows];
        let mut row_max_baselines = vec![0; num_rows];
        let mut max_item_width = 0;

        // analyze dimensions per row to preserve baselines
        for (i, row) in rendered_rows.iter().enumerate() {
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

        // define a uniform cell size for the matrix grid
        let cell_width = max_item_width;
        let mut cell_height = 0;

        for i in 0..num_rows {
            let row_content_height = row_max_baselines[i] + row_max_depths[i];
            cell_height = cell_height.max(row_content_height);
        }

        let row_padding = 1;
        cell_height = cell_height.max(1);

        let active_cell_height = if num_rows > 1 {
            cell_height + row_padding
        } else {
            cell_height
        };

        let hspacing = 4;
        let vspacing = 1;

        let mut matrix_layout_height = num_rows * cell_height + (num_rows - 1) * vspacing;
        let matrix_layout_width = num_cols * cell_width + (num_cols - 1) * hspacing;

        // make the total height odd so the baseline sits right in the center
        if matrix_layout_height.is_multiple_of(2) {
            matrix_layout_height += 1;
        }

        let baseline = matrix_layout_height / 2;
        let mut data = vec![' '; matrix_layout_height * matrix_layout_width];

        for (i, row) in rendered_rows.iter().enumerate() {
            let row_content_height = row_max_baselines[i] + row_max_depths[i];
            // let row_padding_top = (cell_height - row_content_height) / 2;
            let row_padding_top = (active_cell_height - row_content_height) / 2;
            let row_cell_baseline = row_padding_top + row_max_baselines[i];

            for (j, item) in row.iter().enumerate() {
                let cell_x = j * (cell_width + hspacing);
                let cell_y = i * active_cell_height;

                // horizontally center the item in the cell
                let item_x_in_cell = (cell_width - item.width) / 2;
                // vertically align the item's baseline with row's
                let item_y_in_cell = row_cell_baseline - item.baseline;

                let center_x = cell_x + item_x_in_cell;
                let center_y = cell_y + item_y_in_cell;

                item.blit_into(&mut data, matrix_layout_width, center_x, center_y);
            }
        }

        let matrix = Self {
            width: matrix_layout_width,
            height: matrix_layout_height,
            baseline,
            data,
        };

        RenderNode::stretchy_delim(&matrix, left_delim, right_delim, true)
    }
}

impl fmt::Display for RenderNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            let start = y * self.width;
            let end = start + self.width;

            for &c in &self.data[start..end] {
                write!(f, "{c}")?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
