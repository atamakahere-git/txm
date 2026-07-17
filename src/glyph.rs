use crate::ast::Expr;
use crate::layout_tree::LayoutNode;
use crate::style::Style;
use crate::ParseError;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Default, Clone, Copy)]
pub struct RenderCtx {
    pub depth: usize,
    pub current_style: Style,
}

pub trait Glyph: Debug + Send + Sync {
    fn required_args(&self) -> usize {
        0
    }

    fn has_optional(&self) -> bool {
        false
    }

    fn has_limits(&self) -> bool {
        false
    }

    fn render_macro(
        &self,
        args: &[Expr],
        opts: &[Expr],
        ctx: &mut RenderCtx,
        eval: &mut dyn FnMut(&Expr, &mut RenderCtx) -> Result<LayoutNode, ParseError>,
    ) -> Result<LayoutNode, ParseError> {
        let mut rendered_args = Vec::with_capacity(args.len());
        for arg in args {
            rendered_args.push(eval(arg, ctx)?);
        }

        let mut rendered_opts = Vec::with_capacity(opts.len());
        for opt in opts {
            rendered_opts.push(eval(opt, ctx)?);
        }

        Ok(self.render(&rendered_args, &rendered_opts, ctx))
    }

    fn render(
        &self,
        _args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        LayoutNode::empty()
    }
}

pub struct SymbolRegistry {
    map: HashMap<String, Box<dyn Glyph>>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: impl Into<String>, glyph: impl Glyph + 'static) {
        self.map.insert(name.into(), Box::new(glyph));
    }

    pub fn get(&self, name: &str) -> Option<&dyn Glyph> {
        self.map.get(name).map(|g| g.as_ref())
    }
}

#[derive(Debug)]
pub struct LimitGlyph;

impl Glyph for LimitGlyph {
    fn render(
        &self,
        _args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        LayoutNode::from_str("lim")
    }

    fn required_args(&self) -> usize {
        0
    }

    fn has_limits(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct UnicodeGlyph(pub char);

impl Glyph for UnicodeGlyph {
    fn render(
        &self,
        _args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        LayoutNode::from_char(self.0)
    }
}

#[derive(Debug)]
pub struct TextGlyph(pub &'static str);

impl Glyph for TextGlyph {
    fn render(
        &self,
        _args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        LayoutNode::from_str(self.0)
    }
}

#[derive(Debug)]
pub struct BinomGlyph;

impl Glyph for BinomGlyph {
    fn required_args(&self) -> usize {
        2
    }

    fn render(
        &self,
        args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        let inner = LayoutNode::vstack(
            args[0].clone(),
            args[1].clone(),
            crate::layout_tree::LineStyle::None,
        );

        LayoutNode::stretchy_delim(inner, '(', ')', false)
    }
}

#[derive(Debug)]
pub struct FracGlyph;

impl Glyph for FracGlyph {
    fn required_args(&self) -> usize {
        2
    }

    fn render(
        &self,
        args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        LayoutNode::vstack(
            args[0].clone(),
            args[1].clone(),
            crate::layout_tree::LineStyle::Solid,
        )
    }
}

#[derive(Debug)]
pub struct SqrtGlyph;

impl Glyph for SqrtGlyph {
    fn required_args(&self) -> usize {
        1
    }

    fn has_optional(&self) -> bool {
        true
    }

    fn render(&self, args: &[LayoutNode], opts: &[LayoutNode], _ctx: &mut RenderCtx) -> LayoutNode {
        let index = opts.first().cloned();
        LayoutNode::sqrt(args[0].clone(), index)
    }
}

#[derive(Debug)]
pub struct SummationGlyph;
impl Glyph for SummationGlyph {
    fn has_optional(&self) -> bool {
        true
    }

    fn has_limits(&self) -> bool {
        true
    }

    fn required_args(&self) -> usize {
        1
    }

    fn render(
        &self,
        args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        let inner = if args.is_empty() {
            None
        } else {
            Some(args[0].clone())
        };
        LayoutNode::summation(inner)
    }
}

#[derive(Debug)]
pub struct ProductGlyph;
impl Glyph for ProductGlyph {
    fn has_limits(&self) -> bool {
        true
    }

    fn required_args(&self) -> usize {
        1
    }

    fn render(
        &self,
        args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        let inner = if args.is_empty() {
            None
        } else {
            Some(args[0].clone())
        };
        LayoutNode::product(inner)
    }
}

#[derive(Debug)]
pub struct IntegralGlyph;

impl Glyph for IntegralGlyph {
    fn has_limits(&self) -> bool {
        true
    }

    fn required_args(&self) -> usize {
        1
    }

    fn render(&self, args: &[LayoutNode], _opts: &[LayoutNode], ctx: &mut RenderCtx) -> LayoutNode {
        let inner = if args.is_empty() {
            None
        } else {
            Some(args[0].clone())
        };
        let mut node = LayoutNode::integral(inner);
        node.style = ctx.current_style;
        node
    }
}

#[derive(Debug)]
pub struct AlphabetGlyph(pub fn(char) -> char);

impl Glyph for AlphabetGlyph {
    fn required_args(&self) -> usize {
        1
    }

    fn render(
        &self,
        args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        let src = &args[0];
        match &src.kind {
            crate::layout_tree::NodeKind::Text { content } => {
                let mapped: Vec<char> = content.iter().map(|&c| (self.0)(c)).collect();
                LayoutNode {
                    width: src.width,
                    height: src.height,
                    baseline: src.baseline,
                    style: src.style,
                    kind: crate::layout_tree::NodeKind::Text { content: mapped },
                }
            }
            _ => src.clone(),
        }
    }
}

fn shift(c: char, base: u32, off: u32) -> char {
    char::from_u32(base + off).unwrap_or(c)
}

pub fn to_bold(c: char) -> char {
    match c {
        'A'..='Z' => shift(c, 0x1D400, c as u32 - 'A' as u32),
        'a'..='z' => shift(c, 0x1D41A, c as u32 - 'a' as u32),
        '0'..='9' => shift(c, 0x1D7CE, c as u32 - '0' as u32),
        _ => c,
    }
}

pub fn to_bb(c: char) -> char {
    match c {
        'C' => 'ℂ',
        'H' => 'ℍ',
        'N' => 'ℕ',
        'P' => 'ℙ',
        'Q' => 'ℚ',
        'R' => 'ℝ',
        'Z' => 'ℤ',
        'A'..='Z' => shift(c, 0x1D538, c as u32 - 'A' as u32),
        'a'..='z' => shift(c, 0x1D552, c as u32 - 'a' as u32),
        '0'..='9' => shift(c, 0x1D7D8, c as u32 - '0' as u32),
        _ => c,
    }
}

pub fn to_upright(c: char) -> char {
    c
}

pub fn to_italic(c: char) -> char {
    match c {
        'h' => 'ℎ',
        'A'..='Z' => shift(c, 0x1D434, c as u32 - 'A' as u32),
        'a'..='z' => shift(c, 0x1D44E, c as u32 - 'a' as u32),
        _ => c,
    }
}

pub fn to_sans(c: char) -> char {
    match c {
        'A'..='Z' => shift(c, 0x1D5A0, c as u32 - 'A' as u32),
        'a'..='z' => shift(c, 0x1D5BA, c as u32 - 'a' as u32),
        '0'..='9' => shift(c, 0x1D7E2, c as u32 - '0' as u32),
        _ => c,
    }
}

#[derive(Debug)]
pub struct AbsGlyph;

impl Glyph for AbsGlyph {
    fn required_args(&self) -> usize {
        1
    }

    fn render(
        &self,
        args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        LayoutNode::stretchy_delim(args[0].clone(), '|', '|', false)
    }
}

#[derive(Debug)]
pub struct AccentGlyph {
    pub mark: char,
    pub stretch: bool,
}

impl Glyph for AccentGlyph {
    fn required_args(&self) -> usize {
        1
    }

    fn render(
        &self,
        args: &[LayoutNode],
        _opts: &[LayoutNode],
        _ctx: &mut RenderCtx,
    ) -> LayoutNode {
        LayoutNode::accent(args[0].clone(), self.mark, self.stretch)
    }
}

#[derive(Debug)]
pub struct TextColorGlyph;

impl Glyph for TextColorGlyph {
    fn required_args(&self) -> usize {
        2
    }

    fn render_macro(
        &self,
        args: &[Expr],
        _opts: &[Expr],
        ctx: &mut RenderCtx,
        eval: &mut dyn FnMut(&Expr, &mut RenderCtx) -> Result<LayoutNode, ParseError>,
    ) -> Result<LayoutNode, ParseError> {
        let color_str = if let Expr::Ident(c) = &args[0] {
            c.as_str()
        } else {
            panic!("what")
        };

        let prev_style = ctx.current_style;
        ctx.current_style = ctx.current_style.fg(crate::style::parse_color(color_str)?);

        let result = eval(&args[1], ctx);

        ctx.current_style = prev_style;
        result
    }
}
