use winnow::ascii::{dec_uint, float, multispace1};
use winnow::combinator::{alt, delimited, opt, preceded, repeat};
use winnow::error::{ContextError, ErrMode};
use winnow::token::{none_of, one_of, take_until, take_while};
use winnow::Parser;

pub type PResult<O, E = ContextError> = Result<O, ErrMode<E>>;

/// 跳过单行注释 `// ...`
pub fn line_comment<'i>(input: &mut &'i str) -> PResult<(), ContextError> {
    preceded("//", take_while(0.., |c| c != '\n' && c != '\r'))
        .void()
        .parse_next(input)
}

/// 跳过多行注释 `/* ... */`
pub fn block_comment<'i>(input: &mut &'i str) -> PResult<(), ContextError> {
    delimited("/*", take_until(0.., "*/"), "*/")
        .void()
        .parse_next(input)
}

/// 跳过任意空白符与注释
pub fn ws<'i>(input: &mut &'i str) -> PResult<(), ContextError> {
    repeat(
        0..,
        alt((multispace1.void(), line_comment, block_comment)),
    )
    .parse_next(input)
}

/// 前后包裹空白符的词法单元
pub fn lexeme<'i, O, F>(mut inner: F) -> impl FnMut(&mut &'i str) -> PResult<O, ContextError>
where
    F: Parser<&'i str, O, ErrMode<ContextError>>,
{
    move |input: &mut &'i str| {
        ws.parse_next(input)?;
        let res = inner.parse_next(input)?;
        ws.parse_next(input)?;
        Ok(res)
    }
}

/// 匹配指定关键字或符号（忽略周围空白）
pub fn symbol<'i>(s: &'static str) -> impl FnMut(&mut &'i str) -> PResult<&'i str, ContextError> {
    lexeme(s)
}

/// 解析 Rust 风格标识符（如 `spatial`, `q_ready`, `SoloV0Obs`）
pub fn ident<'i>(input: &mut &'i str) -> PResult<String, ContextError> {
    lexeme(|input: &mut &'i str| {
        let first = one_of(|c: char| c.is_alphabetic() || c == '_').parse_next(input)?;
        let rest = take_while(0.., |c: char| c.is_alphanumeric() || c == '_').parse_next(input)?;
        Ok(format!("{}{}", first, rest))
    })
    .parse_next(input)
}

/// 解析变量名，支持带有数组下标的形式（如 `self_x`, `params[0]`, `rel_pos[1]`）
pub fn var_ident<'i>(input: &mut &'i str) -> PResult<String, ContextError> {
    ws.parse_next(input)?;
    let base = ident.parse_next(input)?;
    let index: Option<usize> = opt(delimited(
        symbol("["),
        number_usize,
        symbol("]"),
    ))
    .parse_next(input)?;

    if let Some(idx) = index {
        Ok(format!("{}[{}]", base, idx))
    } else {
        Ok(base)
    }
}

/// 解析 usize 整数
pub fn number_usize<'i>(input: &mut &'i str) -> PResult<usize, ContextError> {
    lexeme(dec_uint).parse_next(input)
}

/// 解析浮点数（支持 `100`, `100.0`, `-0.005`, `1e-5`）
pub fn number_f32<'i>(input: &mut &'i str) -> PResult<f32, ContextError> {
    lexeme(float).parse_next(input)
}

/// 解析字符串字面量 `"..."`
pub fn string_literal<'i>(input: &mut &'i str) -> PResult<String, ContextError> {
    lexeme(delimited(
        "\"",
        repeat(0.., none_of(['\"'])).map(|s: Vec<char>| s.into_iter().collect::<String>()),
        "\"",
    ))
    .parse_next(input)
}
