// SPDX-License-Identifier: MPL-2.0

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    // keywords
    Int,
    Unsigned,
    Void,
    Const,
    Return,
    If,
    Else,
    While,
    Break,
    Continue,
    True,
    False,

    // identifiers / literals
    Ident(String),
    IntLit(i128),

    // punct
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Comma,

    // ops
    Assign,   // =
    EqEq,     // ==
    NotEq,    // !=
    Lt,       // <
    Le,       // <=
    Gt,       // >
    Ge,       // >=

    Plus,     // +
    Minus,    // -
    Star,     // *

    Eof,
}

#[derive(Clone, Debug)]
pub struct LexError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}:{})", self.msg, self.line, self.col)
    }
}

pub fn lex_all(src: &str) -> Result<Vec<Tok>, LexError> {
    let mut lx = Lexer::new(src);
    let mut out = Vec::new();
    loop {
        let t = lx.next_tok()?;
        let end = matches!(t, Tok::Eof);
        out.push(t);
        if end { break; }
    }
    Ok(out)
}

struct Lexer<'a> {
    s: &'a [u8],
    i: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self { s: src.as_bytes(), i: 0, line: 1, col: 1 }
    }

    fn err<T>(&self, msg: impl Into<String>) -> Result<T, LexError> {
        Err(LexError { msg: msg.into(), line: self.line, col: self.col })
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let c = self.s.get(self.i).copied()?;
        self.i += 1;
        if c == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn starts_with(&self, pat: &[u8]) -> bool {
        self.s.get(self.i..self.i + pat.len()) == Some(pat)
    }

    fn skip_ws_and_comments(&mut self) -> Result<(), LexError> {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
                self.bump();
            }

            if self.starts_with(b"//") {
                while let Some(c) = self.bump() {
                    if c == b'\n' { break; }
                }
                continue;
            }

            if self.starts_with(b"/*") {
                self.bump(); self.bump();
                while self.i < self.s.len() && !self.starts_with(b"*/") {
                    self.bump();
                }
                if !self.starts_with(b"*/") {
                    return self.err("unterminated block comment");
                }
                self.bump(); self.bump();
                continue;
            }

            break;
        }
        Ok(())
    }

    fn next_tok(&mut self) -> Result<Tok, LexError> {
        self.skip_ws_and_comments()?;

        let Some(c) = self.peek() else { return Ok(Tok::Eof); };

        // two-char ops
        if self.starts_with(b"==") { self.bump(); self.bump(); return Ok(Tok::EqEq); }
        if self.starts_with(b"!=") { self.bump(); self.bump(); return Ok(Tok::NotEq); }
        if self.starts_with(b"<=") { self.bump(); self.bump(); return Ok(Tok::Le); }
        if self.starts_with(b">=") { self.bump(); self.bump(); return Ok(Tok::Ge); }

        // single-char
        match c {
            b'(' => { self.bump(); return Ok(Tok::LParen); }
            b')' => { self.bump(); return Ok(Tok::RParen); }
            b'{' => { self.bump(); return Ok(Tok::LBrace); }
            b'}' => { self.bump(); return Ok(Tok::RBrace); }
            b';' => { self.bump(); return Ok(Tok::Semi); }
            b',' => { self.bump(); return Ok(Tok::Comma); }

            b'=' => { self.bump(); return Ok(Tok::Assign); }
            b'<' => { self.bump(); return Ok(Tok::Lt); }
            b'>' => { self.bump(); return Ok(Tok::Gt); }

            b'+' => { self.bump(); return Ok(Tok::Plus); }
            b'-' => { self.bump(); return Ok(Tok::Minus); }
            b'*' => { self.bump(); return Ok(Tok::Star); }
            _ => {}
        }

        // number
        if c.is_ascii_digit() {
            let mut v: i128 = 0;
            while let Some(d) = self.peek().filter(|x| x.is_ascii_digit()) {
                self.bump();
                v = v * 10 + (d - b'0') as i128;
            }
            return Ok(Tok::IntLit(v));
        }

        // ident / keyword
        if c.is_ascii_alphabetic() || c == b'_' {
            let start = self.i;
            while let Some(x) = self.peek().filter(|x| x.is_ascii_alphanumeric() || *x == b'_') {
                let _ = x;
                self.bump();
            }
            let text = std::str::from_utf8(&self.s[start..self.i]).unwrap();

            return Ok(match text {
                "int" => Tok::Int,
                "unsigned" => Tok::Unsigned,
                "void" => Tok::Void,
                "const" => Tok::Const,
                "return" => Tok::Return,
                "if" => Tok::If,
                "else" => Tok::Else,
                "while" => Tok::While,
                "break" => Tok::Break,
                "continue" => Tok::Continue,
                "true" => Tok::True,
                "false" => Tok::False,
                _ => Tok::Ident(text.to_string()),
            });
        }

        self.err(format!("unexpected char: {:?}", c as char))
    }
}