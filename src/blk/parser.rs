use super::{Block, Entry, Property};

const MAX_DEPTH: usize = 128;

/// A text parsing error. Lines and columns are one-based; columns count Unicode
/// characters (a tab counts as one column). An initial UTF-8 BOM is ignored.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message} at line {line}, column {column}")]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

/// Parses a text BLK export into an unnamed root block.
///
/// Quoted names and values support single or double quotes, Dagor tilde escapes
/// (`~n`, `~r`, `~t`, `~~`, `~\"`, `~'`), and equivalent backslash escapes.
/// Unrecognized escapes are kept verbatim. Unquoted values are trimmed at their
/// edges; internal whitespace and optional vector commas are preserved. Comments
/// are excluded from values. Unknown types are retained without interpretation.
///
/// Known types are validated: `i` is a signed decimal i32, `i64` is a signed
/// decimal i64, `r` is a finite f64, and `b` is one of `yes`, `no`, `true`,
/// `false`, `1`, or `0`. Their original string representation is retained.
/// Properties must end at a newline, semicolon, closing brace, or end of input.
/// Binary BLK and bracketed arrays are not supported. Includes are never read.
/// At most 128 nested blocks are accepted, excluding the unnamed root.
pub fn parse(source: &str) -> Result<Block, ParseError> {
    let mut parser = Parser {
        source: source.strip_prefix('\u{feff}').unwrap_or(source),
        offset: 0,
        line: 1,
        column: 1,
    };
    parser.block(String::new(), 0, None)
}

#[derive(Clone, Copy)]
struct Location {
    line: usize,
    column: usize,
}

impl Location {
    fn error(self, message: impl Into<String>) -> ParseError {
        ParseError {
            line: self.line,
            column: self.column,
            message: message.into(),
        }
    }
}

struct Parser<'a> {
    source: &'a str,
    offset: usize,
    line: usize,
    column: usize,
}

impl Parser<'_> {
    fn location(&self) -> Location {
        Location {
            line: self.line,
            column: self.column,
        }
    }

    fn rest(&self) -> &str {
        &self.source[self.offset..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        let follows_cr = self.offset > 0 && self.source.as_bytes()[self.offset - 1] == b'\r';
        self.offset += ch.len_utf8();
        match ch {
            '\r' => {
                self.line += 1;
                self.column = 1;
            }
            '\n' => {
                if !follows_cr {
                    self.line += 1;
                }
                self.column = 1;
            }
            _ => self.column += 1,
        }
        Some(ch)
    }

    fn block(
        &mut self,
        name: String,
        depth: usize,
        opening: Option<Location>,
    ) -> Result<Block, ParseError> {
        let mut entries = Vec::new();
        loop {
            self.skip_trivia(true)?;
            match self.peek() {
                Some(';') => {
                    self.bump();
                }
                Some('}') => {
                    if opening.is_none() {
                        return Err(self.location().error("unexpected closing brace '}'"));
                    }
                    self.bump();
                    return Ok(Block { name, entries });
                }
                None => {
                    if let Some(opening) = opening {
                        return Err(self.location().error(format!(
                            "unterminated block {name:?} (opened at line {}, column {}); expected '}}'",
                            opening.line, opening.column
                        )));
                    }
                    return Ok(Block { name, entries });
                }
                _ => entries.push(self.entry(depth)?),
            }
        }
    }

    fn entry(&mut self, depth: usize) -> Result<Entry, ParseError> {
        let name = self.name("block or property name")?;
        self.skip_trivia(true)?;
        if self.peek() == Some('{') {
            let opening = self.location();
            if depth >= MAX_DEPTH {
                return Err(opening.error(format!(
                    "maximum block nesting depth of {MAX_DEPTH} exceeded"
                )));
            }
            self.bump();
            return self.block(name, depth + 1, Some(opening)).map(Entry::Block);
        }

        // The directive has no '='. A property or block literally named
        // 'include' still follows the ordinary grammar.
        if name == "include" && !matches!(self.peek(), Some(':' | '=')) {
            let (value, _) = self.value()?;
            if value.is_empty() {
                return Err(self.location().error("include path must not be empty"));
            }
            return Ok(Entry::Property(Property {
                name,
                kind: "t".into(),
                value,
            }));
        }

        let kind = if self.peek() == Some(':') {
            self.bump();
            self.skip_trivia(false)?;
            let kind = self.bare_name("property type after ':'")?;
            self.skip_trivia(false)?;
            kind
        } else {
            String::new()
        };
        if self.peek() != Some('=') {
            return Err(self.location().error("expected '{', ':' or '=' after name"));
        }
        self.bump();
        let (value, location) = self.value()?;
        validate_value(&kind, &value, location)?;
        Ok(Entry::Property(Property { name, kind, value }))
    }

    fn name(&mut self, expected: &str) -> Result<String, ParseError> {
        match self.peek() {
            Some('\'' | '"') => self.quoted(),
            _ => self.bare_name(expected),
        }
    }

    fn bare_name(&mut self, expected: &str) -> Result<String, ParseError> {
        let start = self.offset;
        while let Some(ch) = self.peek() {
            if ch.is_whitespace()
                || matches!(ch, ':' | '=' | '{' | '}' | ';' | ',' | '\'' | '"')
                || self.rest().starts_with("//")
                || self.rest().starts_with("/*")
            {
                break;
            }
            self.check_character(ch)?;
            self.bump();
        }
        if start == self.offset {
            return Err(self.location().error(format!("expected {expected}")));
        }
        Ok(self.source[start..self.offset].to_owned())
    }

    fn value(&mut self) -> Result<(String, Location), ParseError> {
        let crossed_line = self.skip_trivia(false)?;
        let location = self.location();
        if crossed_line || matches!(self.peek(), None | Some('\n' | '\r' | ';' | '}')) {
            return Err(location.error("expected a property value"));
        }
        let value = if matches!(self.peek(), Some('\'' | '"')) {
            let value = self.quoted()?;
            let crossed_line = self.skip_trivia(false)?;
            if !crossed_line && !matches!(self.peek(), None | Some('\n' | '\r' | ';' | '}')) {
                return Err(self
                    .location()
                    .error("expected ';', newline, or '}' after quoted value"));
            }
            value
        } else {
            self.raw_value()?
        };

        Ok((value, location))
    }

    fn raw_value(&mut self) -> Result<String, ParseError> {
        let start = self.location();
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if matches!(ch, '\n' | '\r' | ';' | '}') || self.rest().starts_with("//") {
                break;
            }
            if self.rest().starts_with("/*") {
                let crossed_line = self.block_comment()?;
                value.push(' ');
                if crossed_line {
                    break;
                }
                continue;
            }
            self.check_character(ch)?;
            if matches!(ch, '{' | '=' | '\'' | '"') {
                return Err(self.location().error(format!(
                    "unexpected {ch:?} in unquoted value; separate entries with ';' or newline"
                )));
            }
            value.push(ch);
            self.bump();
        }
        let value = value.trim().to_owned();
        if value.is_empty() {
            return Err(start.error("expected a property value"));
        }
        Ok(value)
    }

    fn quoted(&mut self) -> Result<String, ParseError> {
        let opening = self.location();
        let quote = self.bump().expect("quoted() is called at a quote");
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            self.bump();
            if ch == quote {
                return Ok(value);
            }
            if ch == '\\' || ch == '~' {
                let Some(escaped) = self.peek() else {
                    break;
                };
                let decoded = match escaped {
                    'n' => Some('\n'),
                    'r' => Some('\r'),
                    't' => Some('\t'),
                    '\'' => Some('\''),
                    '"' => Some('"'),
                    escape if escape == ch => Some(ch),
                    _ => None,
                };
                if let Some(decoded) = decoded {
                    self.bump();
                    value.push(decoded);
                } else {
                    // Keep unknown escapes, rather than dropping path characters.
                    value.push(ch);
                }
            } else {
                if ch.is_control() && !matches!(ch, '\n' | '\r' | '\t') {
                    return Err(opening.error("unsupported control character in quoted string"));
                }
                value.push(ch);
            }
        }
        Err(opening.error("unterminated quoted string"))
    }

    /// Skips comments and whitespace, returning whether a newline was consumed.
    /// With `newlines == false`, bare newlines remain for the caller to handle;
    /// a block comment may nevertheless contain and consume a newline.
    fn skip_trivia(&mut self, newlines: bool) -> Result<bool, ParseError> {
        let mut crossed_line = false;
        loop {
            match self.peek() {
                Some('\n' | '\r') if newlines => {
                    crossed_line = true;
                    self.bump();
                }
                Some(ch) if ch.is_whitespace() && !matches!(ch, '\n' | '\r') => {
                    self.bump();
                }
                Some('/') if self.rest().starts_with("//") => {
                    self.bump();
                    self.bump();
                    while !matches!(self.peek(), None | Some('\n' | '\r')) {
                        self.bump();
                    }
                }
                Some('/') if self.rest().starts_with("/*") => {
                    crossed_line |= self.block_comment()?;
                }
                _ => return Ok(crossed_line),
            }
        }
    }

    fn block_comment(&mut self) -> Result<bool, ParseError> {
        let opening = self.location();
        self.bump();
        self.bump();
        let mut crossed_line = false;
        while self.peek().is_some() {
            if self.rest().starts_with("*/") {
                self.bump();
                self.bump();
                return Ok(crossed_line);
            }
            crossed_line |= matches!(self.bump(), Some('\n' | '\r'));
        }
        Err(opening.error("unterminated block comment"))
    }

    fn check_character(&self, ch: char) -> Result<(), ParseError> {
        if matches!(ch, '[' | ']') {
            return Err(self.location().error("bracketed arrays are not supported"));
        }
        if ch.is_control() && !matches!(ch, '\n' | '\r' | '\t') {
            return Err(self.location().error(
                "unsupported control character; expected a text BLK export, not binary BLK",
            ));
        }
        Ok(())
    }
}

fn validate_value(kind: &str, value: &str, location: Location) -> Result<(), ParseError> {
    let valid = match kind {
        "i" => value.parse::<i32>().is_ok(),
        "i64" => value.parse::<i64>().is_ok(),
        "r" => value.parse::<f64>().is_ok_and(|value| value.is_finite()),
        "b" => matches!(value, "yes" | "no" | "true" | "false" | "1" | "0"),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        let expected = match kind {
            "i" => "a signed decimal 32-bit integer",
            "i64" => "a signed decimal 64-bit integer",
            "r" => "a finite real number",
            "b" => "yes, no, true, false, 1, or 0",
            _ => unreachable!(),
        };
        Err(location.error(format!(
            "invalid value {value:?} for type {kind:?}; expected {expected}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn property(name: &str, kind: &str, value: &str) -> Entry {
        Entry::Property(Property {
            name: name.into(),
            kind: kind.into(),
            value: value.into(),
        })
    }

    fn fails(source: &str, message: &str) -> ParseError {
        let error = parse(source).expect_err(source);
        assert!(
            error.message.contains(message),
            "{source:?}: expected {message:?}, got {error:?}"
        );
        assert!(error.line > 0 && error.column > 0);
        error
    }

    #[test]
    fn empty_root_bom_and_comments() {
        for source in [
            "",
            "\u{feff}",
            " \t\r\n;;;",
            "// eof",
            "/* empty */",
            "\u{feff}// hi\n",
        ] {
            assert_eq!(
                parse(source).unwrap(),
                Block {
                    name: String::new(),
                    entries: Vec::new(),
                }
            );
        }
    }

    #[test]
    fn entry_order_duplicates_and_immediate_iterators() {
        let root = parse("key:i=1; child { key:i=9 } key:i=2; child {}; other:t=x").unwrap();
        assert_eq!(root.name, "");
        assert_eq!(root.entries.len(), 5);
        assert_eq!(root.entries[0], property("key", "i", "1"));
        assert!(matches!(&root.entries[1], Entry::Block(block) if block.name == "child"));
        assert_eq!(root.entries[2], property("key", "i", "2"));
        assert!(matches!(
            &root.entries[3], Entry::Block(block) if block.entries.is_empty()
        ));
        assert_eq!(root.entries[4], property("other", "t", "x"));
        assert_eq!(
            root.blocks().map(|b| b.name.as_str()).collect::<Vec<_>>(),
            ["child", "child"]
        );
        assert_eq!(
            root.properties()
                .map(|p| p.value.as_str())
                .collect::<Vec<_>>(),
            ["1", "2", "x"]
        );
        assert_eq!(root.property("key").unwrap().value, "1");
        assert!(root.property("missing").is_none());
        let root = parse("outer { descendant:t=hidden }").unwrap();
        assert!(root.property("descendant").is_none());
    }

    #[test]
    fn nested_and_adjacent_empty_blocks() {
        let root = parse("a{b{c{}}}d{};e{}\n").unwrap();
        assert_eq!(root.blocks().count(), 3);
        assert_eq!(
            root.blocks().next().unwrap().blocks().next().unwrap().name,
            "b"
        );
    }

    #[test]
    fn typed_and_untyped_properties_keep_spelling() {
        let root =
            parse("text=plain words; empty:t=\"\"\nnumber:i=+0007\nreal:r=1.25e-2\nflag:b=yes")
                .unwrap();
        assert_eq!(
            root.entries,
            vec![
                property("text", "", "plain words"),
                property("empty", "t", ""),
                property("number", "i", "+0007"),
                property("real", "r", "1.25e-2"),
                property("flag", "b", "yes"),
            ]
        );
    }

    #[test]
    fn quoted_names_and_values_ignore_syntax_inside_quotes() {
        let root =
            parse(r#""a : {} // ;" { 'key with spaces':t="{ } // /* ; = : [ ]"; "":t='value' }"#)
                .unwrap();
        let block = root.blocks().next().unwrap();
        assert_eq!(block.name, "a : {} // ;");
        assert_eq!(
            block.entries,
            vec![
                property("key with spaces", "t", "{ } // /* ; = : [ ]"),
                property("", "t", "value"),
            ]
        );
    }

    #[test]
    fn dagor_and_backslash_escapes_are_unescaped() {
        let root =
            parse(r#"dagor:t="a~nb~rc~td~~e~"f~'g"; slash:t="a\nb\rc\td\\e\"f\'g""#).unwrap();
        assert_eq!(root.property("dagor").unwrap().value, "a\nb\rc\td~e\"f'g");
        assert_eq!(root.property("slash").unwrap().value, "a\nb\rc\td\\e\"f'g");
        let root =
            parse(r#""a~"b":t='it~'s'; path:t="C:\Users\Pilot\unknown"; tilde:t="~z""#).unwrap();
        assert_eq!(root.property("a\"b").unwrap().value, "it's");
        assert_eq!(
            root.property("path").unwrap().value,
            r"C:\Users\Pilot\unknown"
        );
        assert_eq!(root.property("tilde").unwrap().value, "~z");
    }

    #[test]
    fn unicode_and_literal_quoted_newlines() {
        let root = parse("\"操縦\" { \"é\":t=\"first\r\nsecond 🛩\" }").unwrap();
        let block = root.blocks().next().unwrap();
        assert_eq!(block.name, "操縦");
        assert_eq!(block.property("é").unwrap().value, "first\r\nsecond 🛩");
    }

    #[test]
    fn unknown_types_and_vectors_remain_raw() {
        let root = parse("a:p3=  1,  -2.5, 3  ; b:p4=1 2 3 4\nc:future=odd:syntax / path, true\nd:future=\"escaped~ntext\"").unwrap();
        assert_eq!(
            root.entries,
            vec![
                property("a", "p3", "1,  -2.5, 3"),
                property("b", "p4", "1 2 3 4"),
                property("c", "future", "odd:syntax / path, true"),
                property("d", "future", "escaped\ntext"),
            ]
        );
    }

    #[test]
    fn inline_comments_and_block_comment_newlines() {
        let root = parse("/* lead */ a/* name */:/* type */i/* equals */=/* value */1/* end */; // line\nb:t=hello/* middle */world // end\nc:i=2/* multi\nline */d:i=3\ne:t=\"x\"/* multi\nline */f:i=4\n// eof").unwrap();
        assert_eq!(
            root.entries,
            vec![
                property("a", "i", "1"),
                property("b", "t", "hello world"),
                property("c", "i", "2"),
                property("d", "i", "3"),
                property("e", "t", "x"),
                property("f", "i", "4"),
            ]
        );
    }

    #[test]
    fn includes_are_preserved_without_reading_files() {
        let root = parse("include \"does/not/exist.blk\"; include 'other.blk'\nouter { include config/base.blk }\ninclude:t=\"ordinary.blk\"\ninclude=untyped.blk\ninclude{}").unwrap();
        assert_eq!(
            root.entries[0],
            property("include", "t", "does/not/exist.blk")
        );
        assert_eq!(root.entries[1], property("include", "t", "other.blk"));
        let outer = root.blocks().next().unwrap();
        assert_eq!(
            outer.entries,
            vec![property("include", "t", "config/base.blk")]
        );
        assert_eq!(root.entries[3], property("include", "t", "ordinary.blk"));
        assert_eq!(root.entries[4], property("include", "", "untyped.blk"));
        assert_eq!(root.blocks().last().unwrap().name, "include");
        fails("include \"\"", "include path must not be empty");
        fails("include", "expected a property value");
    }

    #[test]
    fn integer_boundaries_and_overflow() {
        for (kind, values) in [
            ("i", vec!["-2147483648", "2147483647", "+0", "-0"]),
            ("i64", vec!["-9223372036854775808", "9223372036854775807"]),
        ] {
            for value in values {
                let root = parse(&format!("value:{kind}={value}")).unwrap();
                assert_eq!(root.property("value").unwrap().value, value);
            }
        }
        for source in [
            "v:i=2147483648",
            "v:i=-2147483649",
            "v:i64=9223372036854775808",
            "v:i64=-9223372036854775809",
            "v:i=1.0",
            "v:i=0x10",
            "v:i=hello",
            "v:i=1,2",
            "v:i=+",
            "v:i64=1e3",
        ] {
            fails(source, "invalid value");
        }
    }

    #[test]
    fn real_numbers_and_non_finite_rejection() {
        for value in ["0", "-0", "+1.5", ".5", "1.", "-2.5e-3", "1E+9"] {
            parse(&format!("v:r={value}")).unwrap();
        }
        for value in ["NaN", "inf", "-inf", "1e999", "1.2.3", "one", "1,2"] {
            fails(&format!("v:r={value}"), "finite real number");
        }
    }

    #[test]
    fn boolean_spellings_and_validation_inside_quotes() {
        for value in ["yes", "no", "true", "false", "1", "0"] {
            parse(&format!("v:b={value}")).unwrap();
        }
        for value in ["YES", "True", "2", "on", "off"] {
            fails(&format!("v:b={value}"), "invalid value");
        }
        parse("i:i=\"12\"; f:r='1.5'; b:b=\"yes\"").unwrap();
        fails("v:i=\"twelve\"", "invalid value");
    }

    #[test]
    fn errors_have_precise_unicode_crlf_and_bom_locations() {
        let error = fails("\u{feff}ok:i=1\r\nbad:i=no", "invalid value");
        assert_eq!((error.line, error.column), (2, 7));
        let error = fails("\"é\":i=bad", "invalid value");
        assert_eq!((error.line, error.column), (1, 7));
        let error = fails("a:i=1\rb:i=bad", "invalid value");
        assert_eq!((error.line, error.column), (2, 5));
        let error = fails("a{}\n}", "unexpected closing brace");
        assert_eq!((error.line, error.column), (2, 1));
        let error = fails("a {\n /* unclosed", "unterminated block comment");
        assert_eq!((error.line, error.column), (2, 2));
        let error = fails("a:t=\"unterminated", "unterminated quoted string");
        assert_eq!((error.line, error.column), (1, 5));
    }

    #[test]
    fn malformed_and_unsupported_syntax_is_rejected() {
        for (source, message) in [
            ("}", "unexpected closing brace"),
            ("a { b {}", "unterminated block"),
            ("orphan", "expected '{', ':' or '='"),
            ("a:=1", "property type"),
            ("a:i 1", "expected '{', ':' or '='"),
            ("a:i=", "expected a property value"),
            ("a:t=;", "expected a property value"),
            ("a=\nnext=1", "expected a property value"),
            ("a:i=// no value", "expected a property value"),
            ("a:i=/* comment */", "expected a property value"),
            ("a:i=/* multi\nline */1", "expected a property value"),
            ("a:t=\"x\" b:i=1", "after quoted value"),
            ("a:unknown=x b:i=1", "separate entries"),
            ("a:unknown={ nested }", "unquoted value"),
            ("a:t=prefix\"suffix\"", "unquoted value"),
            ("a:t=[1,2]", "arrays are not supported"),
            ("a[]:i=1", "arrays are not supported"),
            ("[a]", "arrays are not supported"),
            ("\0BBF", "text BLK export"),
            ("a:t=\"\0\"", "control character"),
            ("/* never closed", "unterminated block comment"),
            ("a:t=\"ending\\", "unterminated quoted string"),
            ("a:t=\"ending~", "unterminated quoted string"),
            ("a:t=\"\"\"triple\"\"\"", "after quoted value"),
        ] {
            fails(source, message);
        }
    }

    #[test]
    fn bounded_nesting_accepts_limit_and_rejects_excess() {
        let source = format!("{}{}", "a{".repeat(MAX_DEPTH), "}".repeat(MAX_DEPTH));
        parse(&source).unwrap();
        let source = format!(
            "{}{}",
            "a{".repeat(MAX_DEPTH + 1),
            "}".repeat(MAX_DEPTH + 1)
        );
        let error = fails(&source, "maximum block nesting depth");
        assert_eq!((error.line, error.column), (1, MAX_DEPTH * 2 + 2));
        fails(&"a{".repeat(10_000), "maximum block nesting depth");
    }

    #[test]
    fn actual_war_thunder_export_preserves_repeated_bindings() {
        let root = parse(include_str!("../../tests/fixtures/inputs_wt.blk")).unwrap();
        let controls = root
            .blocks()
            .find(|block| block.name == "controls")
            .unwrap();
        assert_eq!(controls.property("version").unwrap().value, "5");
        let hotkeys = controls
            .blocks()
            .find(|block| block.name == "hotkeys")
            .unwrap();
        let reverse = hotkeys
            .blocks()
            .find(|block| block.name == "ID_AIR_REVERSE")
            .unwrap();
        assert_eq!(
            reverse
                .properties()
                .map(|p| p.value.as_str())
                .collect::<Vec<_>>(),
            ["29", "48"]
        );
        assert!(
            hotkeys
                .blocks()
                .filter(|block| block.name == "ID_CAMERA_NEUTRAL")
                .count()
                > 1
        );
    }

    #[test]
    fn arbitrary_short_utf8_inputs_do_not_panic() {
        let alphabet = [
            'a', 'é', '🛩', '\0', '\n', '\r', ' ', '{', '}', '[', ']', ':', '=', ';', '\'', '"',
            '~', '\\', '/', '*',
        ];
        for first in alphabet {
            for second in alphabet {
                for third in alphabet {
                    let source: String = [first, second, third].into_iter().collect();
                    let _ = parse(&source);
                }
            }
        }
    }
}
