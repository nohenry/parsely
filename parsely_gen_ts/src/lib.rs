use std::collections::HashMap;

use parsely_diagnostics::Diagnostic;
use parsely_lexer::{tokens::Token, AsSpan, Position, Range};
use parsely_parser::program::Program;

mod item;
mod expression;

pub mod module;

/// Code gen result
type Result<T> = std::result::Result<T, Diagnostic>;

pub struct TokenCache<'a> {
    program: &'a Program,
    pub(crate) positions: HashMap<Position, usize>,
    pub(crate) lines: HashMap<usize, Range>,
}

impl<'a> TokenCache<'a> {
    pub fn new(program: &'a Program) -> TokenCache<'a> {
        TokenCache {
            program,
            positions: HashMap::default(),
            lines: HashMap::new(),
        }
    }

    pub fn token_index(&mut self, position: &Position) -> usize {
        if let Some(position) = self.positions.get(position) {
            *position
        } else {
            let token_range = self.line_index(position.line);
            let start_index = token_range.0.start;

            let tokens = self.program.tokens(token_range);

            let token_index = tokens.iter().enumerate().position(|(i, tok)| {
                self.positions.insert(tok.as_span().start, start_index + i);

                tok.as_span().start == *position
            });

            start_index + token_index.expect("Unable to find token with position")
        }
    }

    pub fn line_index(&mut self, line: usize) -> Range {
        if let Some(range) = self.lines.get(&line) {
            range.clone()
        } else {
            let line_token_index = self
                .program
                .tokens
                .binary_search_by_key(&line, |tok| tok.as_span().start.line)
                .expect("Unable to find token with specified line number");

            let start_index = self
                .program
                .tokens
                .iter()
                .take(line_token_index)
                .rev()
                .position(|tok| tok.as_span().start.line != line)
                .unwrap_or(0);
            let start_index = line_token_index - start_index;

            let tokens = &self.program.tokens[start_index..];
            let end_index = tokens
                .iter()
                .position(|tok| {
                    if let Token::Eof(_) = tok {
                        false
                    } else {
                        tok.as_span().start.line != line
                    }
                })
                .unwrap_or(tokens.len());

            self.lines
                .insert(line, Range(start_index..start_index + end_index));

            Range(start_index..start_index + end_index)
        }
    }
}