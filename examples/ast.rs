#![feature(allocator_api)]
#![allow(unused)]

use std::{alloc::Allocator, fmt::Debug};

use rikualloc::{allocator::bump::BumpAllocator, mutex::Locked, source::{os_heap::OsHeap, static_buff::StaticBuffer}};

fn main() {
    const BUFFER_SIZE: usize = 1024 * 1024 * 256;

    let src = include_str!("./expr.txt");
    let bump_alloc = Locked::new(BumpAllocator::new(OsHeap));
    let bump_ref = &bump_alloc;

    let parse_result = parse(src, &bump_ref);

    println!("{:?}", parse_result);
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LeftParen,
    RightParen,
    Integer(u64),
    Plus,
    Minus,
    Star,
    Slash,
}

#[derive(PartialEq)]
enum Ast<A: Allocator> {
    Integer(u64),
    BinaryOp {
        left: Box<Ast<A>, A>,
        op: Token,
        right: Box<Ast<A>, A>,
    },
    UnaryOp {
        op: Token,
        right: Box<Ast<A>, A>,
    },
    Group(Box<Ast<A>, A>),
}

impl <A: Allocator> Debug for Ast<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ast::Integer(i) => f.debug_tuple("Integer").field(i).finish(),
            Ast::BinaryOp { left, op, right } => f.debug_tuple("BinaryOp").field(left).field(op).field(right).finish(),
            Ast::UnaryOp { op, right } => f.debug_tuple("UnaryOp").field(op).field(right).finish(),
            Ast::Group(ast) => f.debug_tuple("Group").field(ast).finish(),
        }
    }
}

fn tokenize(src: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => {
                chars.next();
            } // 空白は無視
            '0'..='9' => {
                // 数字トークン
                let mut num = 0u64;
                while let Some(&d) = chars.peek() {
                    if d.is_alphanumeric() {
                        num = num * 10 + d.to_digit(10).unwrap() as u64;
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Integer(num));
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen);
                chars.next();
            }
            other => {
                panic!("unexpected character: {other}");
            }
        }
    }

    tokens
}

struct Parser<'b, T: Allocator> {
    tokens: Vec<Token>,
    pos: usize,
    alloc: &'b T,
}

impl<'b, A: Allocator> Parser<'b, A> {
    fn new(tokens: Vec<Token>, alloc: &'b A) -> Self {
        Parser {
            tokens,
            pos: 0,
            alloc,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.peek().cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if let Some(tok) = self.next() {
            if tok == expected {
                Ok(())
            } else {
                Err(format!(
                    "期待したトークン {expected:?} ではなく {tok:?} が来ました"
                ))
            }
        } else {
            Err(format!(
                "期待したトークン {expected:?} がありますが入力が終わりました"
            ))
        }
    }

    /// expr = term ( ('+'|'-') term )*
    fn parse_expr(&mut self) -> Result<Ast<&'b A>, String> {
        let mut node = self.parse_term()?;

        while let Some(op) = self.peek() {
            match op {
                Token::Plus | Token::Minus => {
                    let op = self.next().unwrap();
                    let rhs = self.parse_term()?;
                    node = Ast::BinaryOp {
                        left: Box::new_in(node, self.alloc),
                        op,
                        right: Box::new_in(rhs, self.alloc),
                    };
                }
                _ => break,
            }
        }

        Ok(node)
    }

    /// term = factor ( ('*'|'/') factor )*
    fn parse_term(&mut self) -> Result<Ast<&'b A>, String> {
        let mut node = self.parse_factor()?;

        while let Some(op) = self.peek() {
            match op {
                Token::Star | Token::Slash => {
                    let op = self.next().unwrap();
                    let rhs = self.parse_factor()?;
                    node = Ast::BinaryOp {
                        left: Box::new_in(node, self.alloc),
                        op,
                        right: Box::new_in(rhs, self.alloc),
                    };
                }
                _ => break,
            }
        }

        Ok(node)
    }

    /// factor = ('+'|'-') factor | primary
    fn parse_factor(&mut self) -> Result<Ast<&'b A>, String> {
        if let Some(Token::Plus) = self.peek() {
            let op = self.next().unwrap();
            let expr = self.parse_factor()?;
            return Ok(Ast::UnaryOp {
                op,
                right: Box::new_in(expr, self.alloc),
            });
        }
        if let Some(Token::Minus) = self.peek() {
            let op = self.next().unwrap();
            let expr = self.parse_factor()?;
            return Ok(Ast::UnaryOp {
                op,
                right: Box::new_in(expr, self.alloc),
            });
        }
        self.parse_primary()
    }

    /// primary = Integer | '(' expr ')'
    fn parse_primary(&mut self) -> Result<Ast<&'b A>, String> {
        match self.next() {
            Some(Token::Integer(n)) => Ok(Ast::Integer(n)),
            Some(Token::LeftParen) => {
                let expr = self.parse_expr()?;
                self.expect(Token::RightParen)?;
                Ok(Ast::Group(Box::new_in(expr, self.alloc)))
            }
            Some(tok) => Err(format!("予期しないトークン: {tok:?}")),
            None => Err("入力が途中で終わりました".into()),
        }
    }
}

/// トークン列を直接パースして AST を返すユーティリティ
fn parse<'a, T: Allocator>(src: &'a str, alloc: &'a T) -> Ast<&'a T> {
    let tokens = tokenize(src);
    let mut parser = Parser::new(tokens, alloc);
    let ast = parser.parse_expr().unwrap();
    if parser.peek().is_some() {
        panic!("式の最後に不要なトークンがあります")
    } else {
        ast
    }
}

struct Interpreter {
    stack: Vec<i64>,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter { stack: Vec::new() }
    }

    fn run<A: Allocator>(&mut self, ast: Ast<&A>) -> i64 {
        self.eval(ast);
        self.stack.pop().unwrap()
    }

    fn eval<A: Allocator>(&mut self, ast: Ast<&A>) {
        match ast {
            Ast::Integer(n) => self.stack.push(n as i64),
            Ast::BinaryOp { left, op, right } => {
                self.eval(*left);
                self.eval(*right);
                self.bin_op(op);
            },
            Ast::UnaryOp { op, right } => {
                self.eval(*right);
                self.unary_op(op);
            },
            Ast::Group(expr) => {
                self.eval(*expr);
            },
        }
    }

    fn bin_op(&mut self, op: Token) {
        match op {
            Token::Plus => {
                let rhs = self.stack.pop().unwrap();
                let lhs = self.stack.pop().unwrap();
                self.stack.push(lhs + rhs);
            },
            Token::Minus => {
                let rhs = self.stack.pop().unwrap();
                let lhs = self.stack.pop().unwrap();
                self.stack.push(lhs - rhs);
            },
            Token::Star => {
                let rhs = self.stack.pop().unwrap();
                let lhs = self.stack.pop().unwrap();
                self.stack.push(lhs * rhs);
            },
            Token::Slash => {
                let rhs = self.stack.pop().unwrap();
                let lhs = self.stack.pop().unwrap();
                let rhs = if rhs == 0 {
                    1
                } else { rhs };
                self.stack.push(lhs / rhs);
            },
            _ => unreachable!(),
        }
    }

    fn unary_op(&mut self, op: Token) {
        match op {
            Token::Plus => {
                let n = self.stack.pop().unwrap();
                self.stack.push(n);
            },
            Token::Minus => {
                let n = self.stack.pop().unwrap();
                self.stack.push(-n);
            },
            _ => unreachable!(),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
