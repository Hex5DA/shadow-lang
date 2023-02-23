use crate::errors::LexErrors;
use crate::prelude::*;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref IDN_RE: Regex = Regex::new(r"[a-zA-Z][a-zA-Z0-9_]*").unwrap();
}

/// sub-enum of lexemes; possible keyword
#[derive(Debug)]
pub enum Keywords {
    Fn,
    Return,
}

impl Keywords {
    fn new(from: &str) -> Option<Self> {
        Some(match from {
            "fn" => Keywords::Fn,
            "return" => Keywords::Return,
            _ => return None,
        })
    }
}

/// structure for holding different literals
#[derive(Debug)]
pub enum Literal {
    Integer(i64),
}

/// the master list of possible lexemes.
#[derive(Debug)]
pub enum LexemeTypes {
    Keyword(Keywords),
    Literal(Literal),
    Idn(String),
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
}

impl LexemeTypes {
    fn new(from: &str) -> Option<Self> {
        use LexemeTypes::*;
        Some(match from {
            "(" => OpenParen,
            ")" => CloseParen,
            "{" => OpenBrace,
            "}" => CloseBrace,
            ";" => Semicolon,
            other => {
                if let Some(kw) = Keywords::new(other) {
                    Keyword(kw)
                } else if let Ok(num) = other.parse::<i64>() {
                    Literal(self::Literal::Integer(num))
                } else if IDN_RE.is_match(other) {
                    Idn(other.to_string())
                } else {
                    return None;
                }
            }
        })
    }
}

#[derive(Debug)]
#[allow(dead_code)] // TODO: remove
pub struct Lexeme {
    ty: LexemeTypes,
    pos: PosInfo,
}

impl Lexeme {
    fn new(lb: &LexBuffer, raw_token: &String) -> Result<Lexeme> {
        let length = raw_token.len() as u64;
        let pos = PosInfo {
            line: lb.posinfo.line,
            column: lb.posinfo.column - length,
            length,
        };
        let ty = LexemeTypes::new(raw_token).ok_or_else(|| ShadowError::from_pos(
            LexErrors::UnrecognisedToken(raw_token.clone()),
            pos,
        ))?;
        Ok(Lexeme { ty, pos })
    }
}

/// simple buffer to make handling the input easier
struct LexBuffer {
    working: String,
    position: usize,
    posinfo: PosInfo,
}

impl LexBuffer {
    fn adv(&mut self, by: u64) {
        self.position += by as usize;
        self.posinfo.column += by;
    }

    fn over(&self) -> char {
        self.working.chars().nth(self.position).unwrap_or_else(|| panic!("position OOB; ({}/{})\n{:?}",
                self.position,
                self.working.len(),
                self.working))
    }

    fn eat(&mut self) -> String {
        let ret = self.working.drain(..self.position).collect();
        self.position = 0;
        ret
    }

    fn done(&self) -> bool {
        self.working.is_empty()
    }
}

pub fn lex(raw: &str) -> Result<Vec<Lexeme>> {
    let mut lb = LexBuffer {
        working: raw.to_owned(),
        position: 0,
        posinfo: PosInfo::default(),
    };
    let mut lexemes: Vec<Lexeme> = Vec::new();

    while !lb.done() {
        // strings of continous characters
        if lb.over().is_ascii_alphabetic() {
            while lb.over().is_ascii_alphabetic() {
                lb.adv(1);
            }
            let kw_idn = lb.eat();
            lexemes.push(Lexeme::new(&lb, &kw_idn)?);
            continue;
        }

        // strings of numbers
        if lb.over().is_ascii_digit() {
            while lb.over().is_ascii_digit() {
                lb.adv(1);
            }
            let num_lit = lb.eat();
            lexemes.push(Lexeme::new(&lb, &num_lit)?);
            continue;
        }

        // skip whitespace
        if lb.over().is_ascii_whitespace() {
            while !lb.working.is_empty() && lb.over().is_ascii_whitespace() {
                lb.adv(1);
                // ... but take note of newlines
                if lb.eat() == "\n" {
                    lb.posinfo.line += 1;
                    lb.posinfo.column = 0;
                }
            }
            continue;
        }

        lb.adv(1);
        let raw_token = &lb.eat();
        lexemes.push(Lexeme::new(&lb, raw_token)?);
    }

    Ok(lexemes)
}