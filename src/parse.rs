// SPDX-License-Identifier: MPL-2.0

use crate::lex::{lex_all, Tok};
use ir::lower_ast::frontend as s;

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn parse_translation_unit(src: &str) -> Result<s::Program, ParseError> {
    let toks = lex_all(src).map_err(|e| ParseError(e.to_string()))?;
    let mut p = Parser { toks, i: 0 };

    let mut globals = Vec::new();
    let mut functions = Vec::new();

    while !p.is_eof() {
        if p.peek_is(&Tok::Const) {
            globals.push(p.parse_global_const()?);
        } else {
            functions.push(p.parse_function()?);
        }
    }

    Ok(s::Program { globals, functions })
}

struct Parser {
    toks: Vec<Tok>,
    i: usize,
}

impl Parser {
    fn is_eof(&self) -> bool {
        matches!(self.toks.get(self.i), Some(Tok::Eof) | None)
    }

    fn peek(&self) -> &Tok {
        self.toks.get(self.i).unwrap_or(&Tok::Eof)
    }

    fn peek2(&self) -> &Tok {
        self.toks.get(self.i + 1).unwrap_or(&Tok::Eof)
    }

    fn bump(&mut self) -> Tok {
        let t = self.toks.get(self.i).cloned().unwrap_or(Tok::Eof);
        self.i += 1;
        t
    }

    fn peek_is(&self, t: &Tok) -> bool {
        self.peek() == t
    }

    fn expect(&mut self, want: Tok) -> Result<(), ParseError> {
        let got = self.bump();
        if got == want {
            Ok(())
        } else {
            Err(ParseError(format!("expected {:?}, got {:?}", want, got)))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.bump() {
            Tok::Ident(s) => Ok(s),
            other => Err(ParseError(format!("expected identifier, got {:?}", other))),
        }
    }

    fn parse_type(&mut self) -> Result<s::TypeRef, ParseError> {
        // 매우 간단: [unsigned] int | void
        let mut signed = true;
        if self.peek_is(&Tok::Unsigned) {
            self.bump();
            signed = false;
        }

        match self.bump() {
            Tok::Int => Ok(s::TypeRef::Int { bits: 32, signed }),
            Tok::Void => Ok(s::TypeRef::Void),
            other => Err(ParseError(format!("expected type, got {:?}", other))),
        }
    }

    fn lit_i32(v: i128) -> s::Expr {
        s::Expr::Lit(s::Lit::Int { bits: 32, signed: true, value: v })
    }

    fn ensure_bool(e: s::Expr) -> s::Expr {
    match e {
        s::Expr::Cmp { .. } => e,
        s::Expr::Lit(s::Lit::Bool(_)) => e,
        _ => s::Expr::Cmp {
            left: Box::new(e),
            op: s::CmpOpRef::Ne,
            right: Box::new(Self::lit_i32(0)),
        },
    }
}

    fn parse_global_const(&mut self) -> Result<s::GlobalConst, ParseError> {
        self.expect(Tok::Const)?;
        let ty = self.parse_type()?;
        let name = self.expect_ident()?;
        self.expect(Tok::Assign)?;
        let init = self.parse_expr()?;
        self.expect(Tok::Semi)?;
        Ok(s::GlobalConst { name, ty, init })
    }

    fn parse_function(&mut self) -> Result<s::Function, ParseError> {
        let return_type = self.parse_type()?;
        let name = self.expect_ident()?;

        self.expect(Tok::LParen)?;
        let mut parameters = Vec::new();
        if !self.peek_is(&Tok::RParen) {
            loop {
                let ty = self.parse_type()?;
                let pname = self.expect_ident()?;
                parameters.push(s::Parameter { name: pname, ty });

                if self.peek_is(&Tok::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
        }
        self.expect(Tok::RParen)?;

        let body = self.parse_block()?; // 함수는 무조건 { ... }
        Ok(s::Function { name, parameters, return_type, body })
    }

    fn parse_block(&mut self) -> Result<Vec<s::Stmt>, ParseError> {
        self.expect(Tok::LBrace)?;
        let mut out = Vec::new();
        while !self.peek_is(&Tok::RBrace) {
            let mut part = self.parse_stmt()?; // stmt는 Vec로 (블록 flatten)
            out.append(&mut part);
        }
        self.expect(Tok::RBrace)?;
        Ok(out)
    }

    fn parse_stmt_or_block(&mut self) -> Result<Vec<s::Stmt>, ParseError> {
        if self.peek_is(&Tok::LBrace) {
            self.parse_block()
        } else {
            self.parse_stmt()
        }
    }

    fn parse_stmt(&mut self) -> Result<Vec<s::Stmt>, ParseError> {
        match self.peek() {
            Tok::LBrace => return self.parse_block(),

            Tok::Return => {
                self.bump();
                if self.peek_is(&Tok::Semi) {
                    self.bump();
                    return Ok(vec![s::Stmt::Return(None)]);
                }
                let e = self.parse_expr()?;
                self.expect(Tok::Semi)?;
                return Ok(vec![s::Stmt::Return(Some(e))]);
            }

            Tok::Const => {
                self.bump();
                let ty = self.parse_type()?;
                let name = self.expect_ident()?;
                self.expect(Tok::Assign)?;
                let init = self.parse_expr()?;
                self.expect(Tok::Semi)?;
                return Ok(vec![s::Stmt::ConstDecl { name, ty, init }]);
            }

            Tok::Int | Tok::Unsigned => {
                let ty = self.parse_type()?;
                let name = self.expect_ident()?;
                let init = if self.peek_is(&Tok::Assign) {
                    self.bump();
                    Some(self.parse_expr()?)
                } else {
                    None // C의 "int x;" -> IR에서 undef로 처리(위 패치가 담당)
                };
                self.expect(Tok::Semi)?;
                return Ok(vec![s::Stmt::VarDecl { name, ty, init }]);
            }

            Tok::If => {
                self.bump();
                self.expect(Tok::LParen)?;
                let cond_expr = self.parse_expr()?;
                let cond = Self::ensure_bool(cond_expr);
                self.expect(Tok::RParen)?;

                let then_body = self.parse_stmt_or_block()?;
                let else_body = if self.peek_is(&Tok::Else) {
                    self.bump();
                    self.parse_stmt_or_block()?
                } else {
                    Vec::new()
                };

                return Ok(vec![s::Stmt::If { cond, then_body, else_body }]);
            }

            Tok::While => {
                self.bump();
                self.expect(Tok::LParen)?;
                let cond_expr = self.parse_expr()?;
                let cond = Self::ensure_bool(cond_expr);
                self.expect(Tok::RParen)?;
                let body = self.parse_stmt_or_block()?;
                return Ok(vec![s::Stmt::While { cond, body }]);
            }

            Tok::Break => {
                self.bump();
                self.expect(Tok::Semi)?;
                return Ok(vec![s::Stmt::Break]);
            }

            Tok::Continue => {
                self.bump();
                self.expect(Tok::Semi)?;
                return Ok(vec![s::Stmt::Continue]);
            }

            Tok::Ident(_) => {
                // assign or exprstmt
                if matches!((self.peek(), self.peek2()), (Tok::Ident(_), Tok::Assign)) {
                    let name = self.expect_ident()?;
                    self.expect(Tok::Assign)?;
                    let value = self.parse_expr()?;
                    self.expect(Tok::Semi)?;
                    return Ok(vec![s::Stmt::Assign { name, value }]);
                }

                let e = self.parse_expr()?;
                self.expect(Tok::Semi)?;
                return Ok(vec![s::Stmt::ExprStmt(e)]);
            }

            _ => {}
        }

        // fallback: exprstmt
        let e = self.parse_expr()?;
        self.expect(Tok::Semi)?;
        Ok(vec![s::Stmt::ExprStmt(e)])
    }

    // expr := cmp
    fn parse_expr(&mut self) -> Result<s::Expr, ParseError> {
        self.parse_cmp()
    }

    // cmp := add ( (==|!=|<|<=|>|>=) add )?
    fn parse_cmp(&mut self) -> Result<s::Expr, ParseError> {
        let left = self.parse_add()?;
        let op = match self.peek() {
            Tok::EqEq => Some(s::CmpOpRef::Eq),
            Tok::NotEq => Some(s::CmpOpRef::Ne),
            Tok::Lt => Some(s::CmpOpRef::Lt),
            Tok::Le => Some(s::CmpOpRef::Le),
            Tok::Gt => Some(s::CmpOpRef::Gt),
            Tok::Ge => Some(s::CmpOpRef::Ge),
            _ => None,
        };

        if let Some(op) = op {
            self.bump();
            let right = self.parse_add()?;
            Ok(s::Expr::Cmp { left: Box::new(left), op, right: Box::new(right) })
        } else {
            Ok(left)
        }
    }

    // add := mul (('+'|'-') mul)*
    fn parse_add(&mut self) -> Result<s::Expr, ParseError> {
        let mut e = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => Some(s::BinOpRef::Add),
                Tok::Minus => Some(s::BinOpRef::Sub),
                _ => None,
            };
            let Some(op) = op else { break; };
            self.bump();
            let r = self.parse_mul()?;
            e = s::Expr::Binary { left: Box::new(e), op, right: Box::new(r) };
        }
        Ok(e)
    }

    // mul := primary (('*') primary)*
    fn parse_mul(&mut self) -> Result<s::Expr, ParseError> {
        let mut e = self.parse_primary()?;
        while self.peek_is(&Tok::Star) {
            self.bump();
            let r = self.parse_primary()?;
            e = s::Expr::Binary { left: Box::new(e), op: s::BinOpRef::Mul, right: Box::new(r) };
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> Result<s::Expr, ParseError> {
        match self.bump() {
            Tok::IntLit(v) => Ok(s::Expr::Lit(s::Lit::Int { bits: 32, signed: true, value: v })),
            Tok::Ident(name) => Ok(s::Expr::Var(name)),
            Tok::True => Ok(s::Expr::Lit(s::Lit::Bool(true))),
            Tok::False => Ok(s::Expr::Lit(s::Lit::Bool(false))),
            Tok::LParen => {
                let e = self.parse_expr()?;
                self.expect(Tok::RParen)?;
                Ok(e)
            }
            other => Err(ParseError(format!("expected primary, got {:?}", other))),
        }
    }
}