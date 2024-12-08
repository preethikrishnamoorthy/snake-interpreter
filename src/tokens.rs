use logos::Logos;
use std::fmt; // to implement the Display trait
use std::num::ParseIntError;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
    InvalidLength(String),
    InvalidSet,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexicalError::InvalidInteger(e) => write!(f, "{}", e),
            LexicalError::InvalidToken => write!(f, "invalid token"),
            LexicalError::InvalidLength(msg) => write!(f, "{}", msg),
            LexicalError::InvalidSet => write!(f, "cannot set input"),
        }
    }
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+", skip r"#.*\n?", skip r"\r\n", error = LexicalError)]
pub enum Token {
    #[token("var")]
    KeywordVar,
    #[token("print")]
    KeywordPrint,
    #[token("apply")]
    KeywordApply,
    #[token("block")]
    KeywordBlock,
    #[token("let")]
    KeywordLet,
    #[token("set")]
    KeywordSet,
    #[token("repeat-until")]
    KeywordRepeatUntil,
    #[token("if")]
    KeywordIf,
    #[token("struct")]
    KeywordStruct,

    #[token("alloc")]
    KeywordAlloc,
    #[token("lookup")]
    KeywordLookup,
    #[token("index")]
    KeywordIndex,

    #[token("i64")]
    KeywordInt,
    #[token("boolean")]
    KeywordBool,

    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Boolean(bool),

    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex("[0-9]+|-[1-9]+", |lex| lex.slice().parse())]
    Integer(i32),

    #[token("null")]
    NullValue,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(";")]
    Semicolon,
    #[token("|")]
    Pipe,
    #[token(":=")]
    Assign,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(":0")]
    ANOTHAONE,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("//")]
    Comment,

    #[token("+")]
    OperatorAdd,
    #[token("-")]
    OperatorSub,
    #[token("*")]
    OperatorMul,
    #[token("/")]
    CmpToken,
    #[token(">")]
    OperatorGreater,
    #[token("<")]
    OperatorLess,
    #[token("=")]
    OperatorEqual,
    #[token(">=")]
    OperatorGreaterEqual,
    #[token("<=")]
    OperatorLessEqual,

    #[token("add1")]
    OperatorAdd1,
    #[token("sub1")]
    OperatorSub1,
    
    #[token("fun")]
    OperatorFun,   
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
