use std::{collections::HashMap, fmt, fmt::Debug, io::BufRead};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    If(Condition),
    Else,
    End,
    Text {
        speaker: Option<String>,
        content: String,
    },
    Choice(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationEntity {
    Variable(String),
    Number(i32),
}

impl RelationEntity {
    pub fn get_value(&self, variables: &HashMap<String, i32>) -> i32 {
        match self {
            RelationEntity::Number(n) => *n,
            RelationEntity::Variable(var) => variables[var],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    IsSet(String),
    Equals(RelationEntity, RelationEntity),
    MoreThan(RelationEntity, RelationEntity),
}

impl Condition {
    pub fn check(&self, variables: &HashMap<String, i32>) -> bool {
        match self {
            Condition::IsSet(var) => variables.contains_key(var),
            Condition::Equals(lhs, rhs) => lhs.get_value(variables) == rhs.get_value(variables),
            Condition::MoreThan(lhs, rhs) => lhs.get_value(variables) > rhs.get_value(variables),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ParseResult {
    Statement(Statement),
    Error(usize, ParseError),
}

#[derive(Error, Clone, PartialEq, Debug)]
pub enum ParseError {
    #[error(transparent)]
    If(#[from] IfError),

    #[error("Unknown syntax '{0}'")]
    UnknownSyntax(String),
}

pub struct ParseErrColl(Vec<(usize, ParseError)>);

impl fmt::Display for ParseErrColl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[\n\t{}\n]",
            self.0
                .iter()
                .map(|r| { format!("Line {}: `{}`", r.0, r.1) })
                .collect::<Vec<_>>()
                .join("\n\t")
        )?;
        Ok(())
    }
}

impl Debug for ParseErrColl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl std::error::Error for ParseErrColl {}

pub fn parse(reader: impl BufRead) -> Result<Vec<Statement>, ParseErrColl> {
    let res: (Vec<ParseResult>, Vec<ParseResult>) = reader
        .lines()
        .enumerate()
        .filter_map(|(n, line)| {
            if let Ok(line) = line {
                Some({
                    let line = line.trim_start();

                    if line.is_empty() {
                        return None;
                    }

                    let statement: Option<Result<Statement, ParseError>> = parse_if(line)
                        .or_else(|| parse_choice(line))
                        .or_else(|| parse_text(line));

                    if let Some(statement) = statement {
                        match statement {
                            Ok(statement) => ParseResult::Statement(statement),
                            Err(err) => ParseResult::Error(n + 1, err),
                        }
                    } else if line == "end" {
                        ParseResult::Statement(Statement::End)
                    } else if line == "else" {
                        ParseResult::Statement(Statement::Else)
                    } else {
                        ParseResult::Error(n + 1, ParseError::UnknownSyntax(line.to_owned()))
                    }
                })
            } else {
                None
            }
        })
        .partition(|r| matches!(*r, ParseResult::Statement(..)));
    if !res.1.is_empty() {
        Err(ParseErrColl(
            res.1
                .iter()
                .map(|r| {
                    if let ParseResult::Error(n, err) = r {
                        (*n, err.clone())
                    } else {
                        panic!()
                    }
                })
                .collect(),
        ))
    } else {
        Ok(res
            .0
            .iter()
            .map(|r| {
                if let ParseResult::Statement(stmt) = r {
                    stmt.clone()
                } else {
                    panic!()
                }
            })
            .collect())
    }
}

#[derive(Error, Clone, PartialEq, Debug)]
pub enum ChoiceError {}

fn parse_choice(s: &str) -> Option<Result<Statement, ParseError>> {
    if s.starts_with('[') && s.ends_with(']') {
        let line = &s[1..s.len() - 1];
        Some(Ok(Statement::Choice(
            line.split('/').map(|s| s.trim().to_string()).collect(),
        )))
    } else {
        None
    }
}

#[derive(Error, Clone, PartialEq, Debug)]
pub enum TextError {}

fn parse_text(s: &str) -> Option<Result<Statement, ParseError>> {
    if let Some(idx) = s.find(':') {
        let speaker = {
            let speaker = &s[..idx];
            if !speaker.is_empty() {
                Some(speaker.to_owned())
            } else {
                None
            }
        };
        Some(Ok(Statement::Text {
            speaker,
            content: s[idx + 1..].trim_start().to_owned(),
        }))
    } else {
        None
    }
}

fn parse_relation_entity(s: &str) -> RelationEntity {
    if let Ok(n) = s.parse::<i32>() {
        RelationEntity::Number(n)
    } else {
        RelationEntity::Variable(s.to_owned())
    }
}

#[derive(Error, Clone, PartialEq, Debug)]
pub enum IfError {
    #[error("The if contains an unexpected amount of parts '{0}'")]
    InvalidAmountOfParts(usize),
    #[error("The if contains an unknown comparison symbol '{0}'")]
    InvalidComparisonSymbol(String),
}

fn parse_if(s: &str) -> Option<Result<Statement, ParseError>> {
    s.strip_prefix("if ").map(|s| {
        let parts: Vec<&str> = s.split_whitespace().collect();
        Ok(Statement::If(match parts.len() {
            1 => Ok(Condition::IsSet(parts[0].to_owned())),
            3 => {
                let lhs = parse_relation_entity(parts[0]);
                let rhs = parse_relation_entity(parts[2]);
                match parts[1] {
                    "=" => Ok(Condition::Equals(lhs, rhs)),
                    ">" => Ok(Condition::MoreThan(lhs, rhs)),
                    s => Err(IfError::InvalidComparisonSymbol(s.to_owned())),
                }
            }
            n => Err(IfError::InvalidAmountOfParts(n)),
        }?))
    })
}
