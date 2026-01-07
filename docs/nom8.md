# nom 8.0 API 详细使用文档

nom 是一个基于 Rust 的解析器组合库（parser combinators）。它的目标是提供构建安全解析器的工具，同时不牺牲速度或内存消耗。

在 nom 8.0 中，最显著的变化是从**基于闭包的组合器**转向了**基于 Trait 的组合器**。现在推荐使用 `parser.parse(input)` 而不是 `parser(input)`。

---

## 1. 核心概念

### `IResult<I, O, E>`

nom 解析器的标准返回类型。

- `I`: 剩余输入的类型（通常是 `&[u8]` 或 `&str`）。
- `O`: 解析输出的类型。
- `E`: 错误类型（默认为 [(I, ErrorKind)](cci:1://file:///d:/Users/admin/workspace/nom/examples/json.rs:27:0-36:1)）。

```rust
type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), Err<E>>;
```

### `Parser` Trait

nom 8.0 的核心。几乎所有的解析器都实现了这个接口，允许链式调用。

- [parse(input)](cci:1://file:///d:/Users/admin/workspace/nom/examples/json.rs:38:0-55:1): 执行解析。
- `map(f)`: 转换输出结果。
- `and_then(p2)`: 将第一个解析器的输出作为第二个解析器的输入。

---

## 2. 基础解析器 (Basic Elements)

用于识别语法中最底层的元素。

| 函数         | 说明                         | 示例                                 |
| :----------- | :--------------------------- | :----------------------------------- | ------- | ----------------------------------- |
| `tag`        | 匹配特定的字符串或字节序列   | `tag("hello").parse("hello world")`  |
| `char`       | 匹配单个字符                 | `char('a').parse("abc")`             |
| `take`       | 提取指定数量的字节或字符     | `take(4usize).parse("hello")`        |
| `take_while` | 只要满足谓词就一直提取       | `take_while(                         | c: char | c.is_alphabetic()).parse("abc123")` |
| `is_a`       | 匹配包含在给定集合中的字符   | `is_a("abc").parse("aabbccdde")`     |
| `is_not`     | 匹配不包含在给定集合中的字符 | `is_not(" \t").parse("hello world")` |

---

## 3. 选择组合器 (Choice Combinators)

用于处理多种可能的解析路径。

### `alt`

尝试一系列解析器，返回第一个成功的解析结果。

```rust
use nom::branch::alt;
use nom::bytes::complete::tag;

let mut parser = alt((tag("abc"), tag("def")));
assert_eq!(parser.parse("abc"), Ok(("", "abc")));
assert_eq!(parser.parse("def"), Ok(("", "def")));
```

---

## 4. 序列组合器 (Sequence Combinators)

用于按顺序组合多个解析器。

| 函数             | 说明                                   | 示例                                                         |
| :--------------- | :------------------------------------- | :----------------------------------------------------------- |
| `tuple`          | 按顺序执行多个解析器，结果存入元组     | `tuple((tag("ab"), tag("cd"))).parse("abcd")`                |
| `preceded`       | 匹配第一个并丢弃，返回第二个的结果     | `preceded(tag("("), tag("abc")).parse("(abc")`               |
| `terminated`     | 匹配第一个并返回，匹配第二个并丢弃     | `terminated(tag("abc"), tag(")")).parse("abc)")`             |
| `delimited`      | 匹配三个，丢弃首尾，返回中间的结果     | `delimited(char('('), tag("abc"), char(')')).parse("(abc)")` |
| `pair`           | 匹配两个解析器，返回元组               | `pair(tag("a"), tag("b")).parse("ab")`                       |
| `separated_pair` | 匹配两个解析器，中间由第三个分隔并丢弃 | `separated_pair(tag("a"), char(','), tag("b")).parse("a,b")` |

---

## 5. 重复解析器 (Multi Parsers)

用于多次应用同一个解析器。

| 函数              | 说明                                 | 示例                                                |
| :---------------- | :----------------------------------- | :-------------------------------------------------- |
| `many0`           | 匹配 0 次或多次，结果存入 `Vec`      | `many0(char('a')).parse("aaa")`                     |
| `many1`           | 匹配 1 次或多次                      | `many1(char('a')).parse("aaa")`                     |
| `separated_list0` | 匹配由分隔符隔开的列表（0 次或多次） | `separated_list0(char(','), digit1).parse("1,2,3")` |
| `count`           | 匹配精确指定的次数                   | `count(char('a'), 3).parse("aaa")`                  |
| `many_till`       | 重复第一个解析器直到第二个解析器成功 | `many_till(anychar, tag("end")).parse("abcend")`    |

---

## 6. 转换与修饰 (Modifiers)

用于修改解析器的行为或转换其输出。

- **`map`**: 转换解析结果。
  ```rust
  map(digit1, |s: &str| s.parse::<u32>().unwrap()).parse("123")
  ```
- **`map_res`**: 转换结果，如果转换函数返回 `Result::Err`，则解析失败。
- **`opt`**: 使解析器变为可选（返回 `Option<O>`）。
- **`cut`**: 错误升级。一旦 `cut` 之后的解析失败，将不再回溯（从 `Error` 变为 `Failure`）。
- **`peek`**: 查看输入但不消耗。
- **`recognize`**: 如果子解析器成功，返回其消耗的原始输入。
- **`all_consuming`**: 要求解析器消耗掉所有输入，否则报错。

---

## 7. 数字解析器 (Number Parsers)

位于 `nom::number::complete` 或 `nom::number::streaming`。

- **整数**: `be_u32` (大端), `le_i16` (小端), `u8`, `i64` 等。
- **浮点数**: `float`, `double`。
- **十六进制**: `hex_u32`。

---

## 8. 字符解析器 (Character Parsers)

位于 `nom::character::complete`。

- `alpha1` / `alpha0`: 字母。
- `digit1` / `digit0`: 数字。
- `alphanumeric1`: 字母或数字。
- `multispace1` / `multispace0`: 空格、制表符、换行符。
- `line_ending`: `\n` 或 `\r\n`。

---

## 9. 错误处理 (Error Handling)

nom 8.0 推荐配合 `nom-language` 使用以获得更好的错误提示。

- **`ErrorKind`**: 基础错误枚举。
- **`VerboseError`**: (需 `nom-language` 库) 记录完整的错误路径和上下文。
- **`context`**: 为解析步骤添加描述性标签。

```rust
use nom_language::error::VerboseError;
fn parser(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    context("my_tag", tag("hello")).parse(i)
}
```

---

## 10. nom 8.0 迁移要点

1.  **Trait 优先**: 以前是 `tag("abc")(input)`，现在是 `tag("abc").parse(input)`。
2.  **元组参数**: `alt` 和 `tuple` 现在接受元组作为参数，而不是以前的宏或嵌套。
    - 正确: `alt((p1, p2, p3))`
3.  **模块路径**: `nom::bits` 不再在根目录重导出。请使用 `nom::bits::complete::tag`。
4.  **Input Trait**: 多个输入相关的 Trait（如 `InputIter`, `Slice`）已合并到统一的 `Input` Trait 中。

---
