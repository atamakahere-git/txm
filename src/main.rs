use chumsky::pratt::*;
use chumsky::prelude::*;

const COMPACT_SIMPLE_FRACTIONAL_EXPONENTS: bool = false;

#[derive(Debug, Clone)]
pub enum Expr {
    VarNum(String),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Eq(Box<Self>, Box<Self>),
    Fraction(Box<Self>, Box<Self>),
    Power(Box<Self>, Box<Self>),
    Parens(Box<Self>),
    Braces(Box<Self>),
    Brackets(Box<Self>),
    Fn {
        name: String,
        power: Option<Box<Self>>,
        derivatives: usize,
        args: Box<Self>,
    },
}

fn parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Simple<'a, char>>> {
    let num = text::digits::<_, extra::Err<Simple<char>>>(10)
        .to_slice()
        .map(|s: &str| Expr::VarNum(s.to_string()));

    let var = text::ident().map(|s: &str| Expr::VarNum(s.to_string()));

    recursive(|expr| {
        let primes = just('\'').repeated().collect::<Vec<_>>().map(|v| v.len());

        let func_call = text::ident()
            .then(primes)
            .then(just('^').ignore_then(expr.clone()).or_not())
            .then(
                expr.clone()
                    .delimited_by(just('('), just(')'))
                    .map(Box::new),
            )
            .map(
                |(((name, der), power), args): (((&str, usize), Option<Expr>), Box<Expr>)| {
                    Expr::Fn {
                        name: name.to_string(),
                        power: power.map(Box::new),
                        derivatives: der,
                        args,
                    }
                },
            );

        let atom = num
            .or(func_call) // Look for functions BEFORE variables so 'f' isn't eaten up as a bare var
            .or(var)
            .or(expr
                .clone()
                .delimited_by(just('('), just(')'))
                .map(|e| Expr::Parens(Box::new(e))))
            .or(expr
                .clone()
                .delimited_by(just('{'), just('}'))
                .map(|e| Expr::Braces(Box::new(e))))
            .or(expr
                .clone()
                .delimited_by(just('['), just(']'))
                .map(|e| Expr::Brackets(Box::new(e))))
            .padded();

        atom.pratt((
            infix(left(1), just('='), |l, _, r, _| {
                Expr::Eq(Box::new(l), Box::new(r))
            }),
            infix(left(2), just('+'), |l, _, r, _| {
                Expr::Add(Box::new(l), Box::new(r))
            }),
            infix(left(2), just('-'), |l, _, r, _| {
                Expr::Sub(Box::new(l), Box::new(r))
            }),
            infix(left(3), just('*'), |l, _, r, _| {
                Expr::Mul(Box::new(l), Box::new(r))
            }),
            infix(left(3), just('/'), |l, _, r, _| {
                Expr::Fraction(Box::new(l), Box::new(r))
            }),
            infix(right(4), just('^'), |l, _, r, _| {
                Expr::Power(Box::new(l), Box::new(r))
            }),
        ))
    })
}

pub struct Layout {
    width: usize,
    height: usize,
    baseline: usize,

    /// 2D char buffer, format: Row<Column>
    /// All columns are of the same size.
    grid: Vec<Vec<char>>,
}

impl Layout {
    pub fn new(width: usize, height: usize, baseline: usize) -> Self {
        Self {
            width,
            height,
            baseline,
            grid: vec![vec![' '; width]; height],
        }
    }

    pub fn blit(&mut self, other: &Layout, x_off: usize, y_off: usize) {
        for y in 0..other.height {
            for x in 0..other.width {
                self.grid[y + y_off][x + x_off] = other.grid[y][x];
            }
        }
    }

    /// The Layout Engine.
    pub fn compute_layout(expr: &Expr, depth: usize) -> Layout {
        match expr {
            Expr::VarNum(x) => {
                let length = x.len();
                let mut layout = Layout::new(length, 1, 0);

                for (i, c) in x.chars().enumerate() {
                    layout.grid[0][i] = c;
                }

                layout
            }

            Expr::Power(base, exponent) => render_power(
                Layout::compute_layout(base, depth),
                strip_parenthesis(exponent),
                depth,
            ),

            Expr::Add(lhs, rhs) => render_infix(lhs, '+', rhs, depth),
            Expr::Sub(lhs, rhs) => render_infix(lhs, '-', rhs, depth),
            Expr::Mul(lhs, rhs) => {
                let lhs_layout = Layout::compute_layout(lhs, depth);
                let rhs_layout = Layout::compute_layout(rhs, depth);

                let is_num = |e: &Expr| {
                    if let Expr::VarNum(s) = e {
                        s.chars().all(|c| c.is_ascii_digit())
                    } else {
                        false
                    }
                };

                let needs_symbol = if is_num(lhs) && is_num(rhs) {
                    true
                } else if let Expr::Power(_, exp) = &**lhs
                    && is_num(exp)
                    && is_num(rhs)
                {
                    true
                } else if let Expr::VarNum(s) = &**lhs
                    && !s.chars().all(|c| c.is_ascii_digit())
                    && is_num(rhs)
                {
                    true
                } else if let Expr::Fraction(_, _) = &**lhs {
                    true
                } else {
                    false
                };

                // CRITICAL FIX: Calculate layout boundaries relative to baseline alignment
                let baseline = lhs_layout.baseline.max(rhs_layout.baseline);
                let lhs_below = lhs_layout.height - lhs_layout.baseline;
                let rhs_below = rhs_layout.height - rhs_layout.baseline;
                let height = baseline + lhs_below.max(rhs_below);

                if needs_symbol {
                    let width = lhs_layout.width + 3 + rhs_layout.width;
                    let mut layout = Layout::new(width, height, baseline);

                    layout.blit(&lhs_layout, 0, baseline - lhs_layout.baseline);
                    layout.grid[baseline][lhs_layout.width + 1] = '·';
                    layout.blit(
                        &rhs_layout,
                        lhs_layout.width + 3,
                        baseline - rhs_layout.baseline,
                    );

                    layout
                } else {
                    let width = lhs_layout.width + rhs_layout.width;
                    let mut layout = Layout::new(width, height, baseline);

                    layout.blit(&lhs_layout, 0, baseline - lhs_layout.baseline);
                    layout.blit(
                        &rhs_layout,
                        lhs_layout.width,
                        baseline - rhs_layout.baseline,
                    );

                    layout
                }
            }
            Expr::Eq(lhs, rhs) => render_infix(lhs, '=', rhs, depth),

            Expr::Fraction(numerator, denominator) => {
                let numerator_layout =
                    Layout::compute_layout(strip_parenthesis(numerator), depth + 1);
                let denominator_layout =
                    Layout::compute_layout(strip_parenthesis(denominator), depth + 1);

                let height = numerator_layout.height + denominator_layout.height + 1;
                let side_padding = if depth == 0 { 1 } else { 0 };
                let width =
                    numerator_layout.width.max(denominator_layout.width) + (side_padding * 2);
                let baseline = numerator_layout.height;

                // center top and bottom
                let numerator_x_offset = (width - numerator_layout.width) / 2;
                let denominator_x_offset = (width - denominator_layout.width) / 2;

                let mut layout = Layout::new(width, height, baseline);
                layout.blit(&numerator_layout, numerator_x_offset, 0);
                layout.blit(&denominator_layout, denominator_x_offset, baseline + 1);
                for x in 0..width {
                    layout.grid[baseline][x] = '─';
                }

                layout
            }

            Expr::Parens(expr) => {
                let expr_layout = Layout::compute_layout(expr, depth);
                render_parens(expr_layout)
            }

            Expr::Brackets(expr) => {
                let expr_layout = Layout::compute_layout(expr, depth);
                render_brackets(expr_layout)
            }

            Expr::Braces(expr) => {
                let expr_layout = Layout::compute_layout(expr, depth);
                render_braces(expr_layout)
            }

            Expr::Fn {
                name,
                power,
                derivatives: 0,
                args,
            } if name == "sqrt" => render_sqrt(power.as_ref(), args, depth),

            Expr::Fn {
                name,
                power,
                derivatives,
                args,
            } => {
                let name = format!("{name}{}", "'".repeat(*derivatives));
                let mut name_layout = Layout::new(name.len(), 1, 0);

                for (i, c) in name.chars().enumerate() {
                    name_layout.grid[0][i] = c;
                }

                if let Some(power) = power {
                    name_layout = render_power(name_layout, power, depth);
                }

                let args = Layout::compute_layout(&Expr::Parens(args.clone()), depth);

                let final_height = name_layout.height.max(args.height);
                let final_baseline = name_layout.baseline.max(args.baseline);
                let final_width = name_layout.width + args.width;

                let mut final_layout = Layout::new(final_width, final_height, final_baseline);
                final_layout.blit(&name_layout, 0, final_baseline - name_layout.baseline);
                final_layout.blit(&args, name_layout.width, final_baseline - args.baseline);

                final_layout
            }
        }
    }
}

fn render_parens(expr_layout: Layout) -> Layout {
    let height = expr_layout.height;
    let mut width = expr_layout.width + 2;
    let baseline = expr_layout.baseline;

    if height > 1 {
        width += 2; // padding

        let mut layout = Layout::new(width, height, baseline);
        layout.blit(&expr_layout, 2, 0);
        let g = &mut layout.grid;

        g[0][0] = '⎛';
        g[0][width - 1] = '⎞';
        g[height - 1][0] = '⎝';
        g[height - 1][width - 1] = '⎠';

        for y in 0..height {
            if y != 0 && y != height - 1 {
                g[y][0] = '⎟';
                g[y][width - 1] = '⎟';
            }
        }

        layout
    } else {
        let mut layout = Layout::new(width, height, baseline);
        layout.blit(&expr_layout, 1, 0);
        layout.grid[0][0] = '(';
        layout.grid[0][width - 1] = ')';
        layout
    }
}

fn render_brackets(expr_layout: Layout) -> Layout {
    let height = expr_layout.height;
    let mut width = expr_layout.width + 2;
    let baseline = expr_layout.baseline;

    if height > 1 {
        width += 2;
        let mut layout = Layout::new(width, height, baseline);
        layout.blit(&expr_layout, 2, 0);
        let g = &mut layout.grid;

        g[0][0] = '⎡';
        g[0][width - 1] = '⎤';
        g[height - 1][0] = '⎣';
        g[height - 1][width - 1] = '⎦';

        for y in 1..height - 1 {
            g[y][0] = '⎢';
            g[y][width - 1] = '⎢';
        }
        layout
    } else {
        let mut layout = Layout::new(width, height, baseline);
        layout.blit(&expr_layout, 1, 0);
        layout.grid[0][0] = '[';
        layout.grid[0][width - 1] = ']';
        layout
    }
}

fn render_braces(expr_layout: Layout) -> Layout {
    let height = expr_layout.height;
    let mut width = expr_layout.width + 2;
    let baseline = expr_layout.baseline;

    if height > 1 {
        width += 2;
        let mut layout = Layout::new(width, height, baseline);
        layout.blit(&expr_layout, 2, 0);
        let g = &mut layout.grid;

        // Caps
        g[0][0] = '⎧';
        g[0][width - 1] = '⎫';
        g[height - 1][0] = '⎩';
        g[height - 1][width - 1] = '⎭';

        // Vertical extensions and Baseline middle
        for y in 1..height - 1 {
            if y == baseline {
                g[y][0] = '⎨';
                g[y][width - 1] = '⎬';
            } else {
                g[y][0] = '⎪';
                g[y][width - 1] = '⎪';
            }
        }
        layout
    } else {
        let mut layout = Layout::new(width, height, baseline);
        layout.blit(&expr_layout, 1, 0);
        layout.grid[0][0] = '{';
        layout.grid[0][width - 1] = '}';
        layout
    }
}

fn render_sqrt(power: Option<&Box<Expr>>, args: &Expr, depth: usize) -> Layout {
    let arg_layout = Layout::compute_layout(args, depth);

    // Only use the ultra-compact '√' if the argument is a single atomic variable/number (e.g., √x, √42)
    let layout = if arg_layout.height == 1 && matches!(args, Expr::VarNum(_)) {
        let width = arg_layout.width + 1;
        let mut lay = Layout::new(width, 1, 0);

        lay.grid[0][0] = '√';
        lay.blit(&arg_layout, 1, 0);

        lay
    } else {
        let height = arg_layout.height + 1;
        let width = arg_layout.width + 2;
        let baseline = arg_layout.baseline + 1;

        let mut lay = Layout::new(width, height, baseline);
        lay.blit(&arg_layout, 2, 1);

        lay.grid[0][1] = '┌';
        for x in 2..width {
            lay.grid[0][x] = '─';
        }

        for y in 1..height {
            lay.grid[y][1] = '│';
        }

        lay.grid[height - 1][0] = '╲';

        lay
    };

    if let Some(power) = power {
        return render_power(render_parens(layout), power, depth);
    }

    layout
}

fn render_power(base_layout: Layout, exponent: &Expr, depth: usize) -> Layout {
    if let Expr::VarNum(x) = exponent
        && x.len() == 1
        && let Some(ch) = x.chars().next()
        && ch.is_ascii_digit()
    {
        let mut layout = Layout::new(
            base_layout.width + 1,
            base_layout.height,
            base_layout.baseline,
        );
        layout.blit(&base_layout, 0, 0);
        layout.grid[0][layout.width - 1] =
            ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'][(ch as u8 - b'0') as usize];

        layout
    } else if COMPACT_SIMPLE_FRACTIONAL_EXPONENTS
        && let Expr::Fraction(n, d) = exponent
        && let Expr::VarNum(n) = &**n
        && let Expr::VarNum(d) = &**d
    {
        let exp_string = format!("{n}/{d}");
        let mut layout = Layout::new(
            base_layout.width + exp_string.len(),
            base_layout.height + 1,
            base_layout.baseline,
        );

        layout.blit(&base_layout, 0, 1);

        let mut chars = exp_string.chars();
        for c in &mut layout.grid[0][base_layout.width..] {
            *c = chars.next().unwrap();
        }

        layout
    } else {
        let exponent_layout = Layout::compute_layout(exponent, depth + 1);

        // exponent goes to the Row 0
        // and the base's baseline is pushed down by exponent's height

        let height = exponent_layout.height + base_layout.height;
        let width = exponent_layout.width + base_layout.width;
        let baseline = base_layout.baseline + exponent_layout.height;

        let mut layout = Layout::new(width, height, baseline);
        layout.blit(&base_layout, 0, exponent_layout.height);
        layout.blit(&exponent_layout, base_layout.width, 0);

        layout
    }
}

fn render_infix(lhs: &Expr, op: char, rhs: &Expr, depth: usize) -> Layout {
    let lhs_layout = Layout::compute_layout(lhs, depth);
    let rhs_layout = Layout::compute_layout(rhs, depth);

    let baseline = lhs_layout.baseline.max(rhs_layout.baseline);

    let lhs_y_offset = baseline - lhs_layout.baseline;
    let rhs_y_offset = baseline - rhs_layout.baseline;

    let width = lhs_layout.width + rhs_layout.width + 3;
    let height = (lhs_y_offset + lhs_layout.height).max(rhs_y_offset + rhs_layout.height);

    let mut layout = Layout::new(width, height, baseline);
    layout.blit(&lhs_layout, 0, lhs_y_offset);
    layout.grid[baseline][lhs_layout.width + 1] = op;
    layout.blit(&rhs_layout, lhs_layout.width + 3, rhs_y_offset);

    layout
}

fn strip_parenthesis(lhs: &Expr) -> &Expr {
    match lhs {
        Expr::Parens(e) => e,
        e => e,
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for column in &self.grid {
            for c in column {
                write!(f, "{c}")?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

fn main() {
    render_boxed("f(x) = f(a) + f'(a) * (x - a) + (f''(a) / 2) * (x - a)^2 + (f'''(a) / 3) * (x - a)^3");
    render_boxed("f(x) = 1 / sqrt(2 * pi) * (1/e^(x^2 / 2))");
    render_boxed("sin(π/10) = (sqrt(5) - 1)/4");
    render_boxed("e^iπ + 1 = 0");
    render_boxed("1/e^((sin^2(x) / 2)) * cos(sqrt(x^2 + 1))");
    render_boxed("dy/dx = (x^2 + 1) / (y + 1)");
}

fn render(input: &str) {
    let res = parser().parse(input).into_result();
    let expr = res.unwrap();
    let layout = Layout::compute_layout(&expr, 0);
    println!("{layout}");
}

fn render_boxed(input: &str) {
    let res = parser().parse(input).into_result();
    let expr = res.unwrap();
    let layout = Layout::compute_layout(&expr, 0);
    let mut box_layout = Layout::new(layout.width + 4, layout.height + 4, 0);

    let grid = &mut box_layout.grid;
    let w = box_layout.width;
    let h = box_layout.height;

    // draw a box
    grid[0][0] = '┌';
    grid[h - 1][0] = '└';
    grid[0][w - 1] = '┐';
    grid[h - 1][w - 1] = '┘';

    for y in 0..h {
        if y == 0 || y == h - 1 {
            for x in 1..w - 1 {
                grid[y][x] = '─';
            }
        } else {
            grid[y][0] = '│';
            grid[y][w - 1] = '│';
        }
    }

    box_layout.blit(&layout, 2, 2);

    // println!("\n{input}");
    print!("{box_layout}");
}
