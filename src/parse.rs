use crate::lex::{Keyword, Lexeme, Literal};
use anyhow::{bail, Context, Result};
use std::collections::VecDeque;

#[derive(Default, Debug)]
pub enum PrimitiveType {
    // is this bad? this feels bad
    #[default]
    Void,
    Int,
}

impl PrimitiveType {
    pub fn from_str(from: String) -> Result<Self> {
        Ok(match from.as_str() {
            "void" => Self::Void,
            "int" => Self::Int,
            _ => bail!(
                "'Custom' variable types not implemented yet (given {})",
                from
            ),
        })
    }
}

macro_rules! consume {
    ( $variant:pat in $vec:expr => $then:stmt) => {
        match $vec.pop_front() {
            Some($variant) => Ok::<(), anyhow::Error>({$then}),
            None => bail!("Unexpected EOF"),
            got @ _ => bail!("Expected {}, got {:?}", stringify!($variant), got),
         }
    };
    ( $($variant:pat),+ in $vec:expr) => {
        $(
        consume!($variant in $vec => {})
        )+
    };
}

pub trait ASTNode: std::fmt::Debug {
    fn new(tokens: &mut VecDeque<Lexeme>) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum Statement {
    Return(Option<Expression>),
    Function(Function),
    // VariableAssignment(Assignment),
}

impl ASTNode for Statement {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        Ok(match lexemes.front().context("Unexpected EOF")? {
            Lexeme::Keyword(Keyword::Fn) => Self::Function(Function::new(lexemes)?),
            Lexeme::Keyword(Keyword::Return) => {
                consume!(Lexeme::Keyword(Keyword::Return) in lexemes)?;
                let expr = if matches!(lexemes.front().context("Unexpected EOF")?, Lexeme::Newline) {
                    None
                } else {
                    Some(Expression::new(lexemes)?)
                };
                consume!(Lexeme::Newline in lexemes)?;
                Self::Return(expr)
            },
            _ => todo!(),
        })
    }
}

#[derive(Debug)]
pub enum Expression{
    Literal(Literal),
}

impl Expression {
    pub fn evaltype(&self) -> PrimitiveType {
        match self {
            Self::Literal(lit) => match lit {
                Literal::Integer(_) => PrimitiveType::Int,
            },
        }
    }
    pub fn eval(&self) -> i64 {
        match self {
            Self::Literal(lit) => match lit {
                Literal::Integer(inner) => *inner,
            },
        }
    }
}

impl ASTNode for Expression {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let node: Self;
        if let Lexeme::Literal(lit) = lexemes.front().context("Unexpected EOF")? {
            node = Expression::Literal(*lit);
            lexemes.pop_front();
        } else {
            bail!("Only literal expressions are supported for now!");
        }

        Ok(node)
    }
}

#[derive(Debug, Default)]
pub struct Parameter {
    pub name: String,
    pub pm_type: PrimitiveType,
}

impl ASTNode for Parameter {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let mut node = Self::default();

        consume!(Lexeme::Idn(pmt) in lexemes => {
            node.pm_type = PrimitiveType::from_str(pmt)?;
        })?;
        consume!(Lexeme::Idn(nm) in lexemes => {
            node.name = nm;
        })?;

        Ok(node)
    }
}

#[derive(Debug, Default)]
pub struct Function {
    pub name: String,
    pub body: Block,
    pub return_type: PrimitiveType,
    pub params: Vec<Parameter>,
}

impl ASTNode for Function {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let mut node = Function::default();

        consume!(Lexeme::Keyword(Keyword::Fn) in lexemes)?;
        consume!(Lexeme::Idn(tp) in lexemes => {
            node.return_type = PrimitiveType::from_str(tp)?;
        })?;
        consume!(Lexeme::Idn(nm) in lexemes => {
            node.name = nm;
        })?;
        consume!(Lexeme::OpenParen in lexemes)?;

        if !matches!(lexemes.front(), Some(Lexeme::CloseParen)) {
            while !lexemes.is_empty() {
                node.params.push(Parameter::new(lexemes)?);
                match lexemes.front() {
                    Some(Lexeme::Delimiter) => {
                        consume!(Lexeme::Delimiter in lexemes)?;
                    }
                    _ => break,
                }
            }
        }

        consume!(Lexeme::CloseParen in lexemes)?;
        node.body = Block::new(lexemes)?;
        Ok(node)
    }
}

#[derive(Debug, Default)]
pub struct Block {
    pub stmts: Vec<Statement>,
}

impl ASTNode for Block {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let mut node = Self::default();

        consume!(Lexeme::OpenBrace in lexemes)?;
        while !lexemes.is_empty() {
            if let Some(Lexeme::CloseBrace) = lexemes.front() {
                break;
            }
            node.stmts.push(Statement::new(lexemes)?);
        }
        consume!(Lexeme::CloseBrace in lexemes)?;

        Ok(node)
    }
}

#[derive(Debug, Default)]
pub struct Root {
    pub stmts: Vec<Statement>,
}

impl ASTNode for Root {
    fn new(lexemes: &mut VecDeque<Lexeme>) -> Result<Self> {
        let mut node = Self::default();

        while !lexemes.is_empty() {
            if let Some(Lexeme::CloseBrace) = lexemes.front() {
                break;
            }
            node.stmts.push(Statement::new(lexemes)?);
        }

        Ok(node)
    }
}

pub fn parse(lexemes: Vec<Lexeme>) -> Result<Root> {
    Root::new(&mut VecDeque::from(lexemes))
}
