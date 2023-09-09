use std::{fmt::Display, path::Path};

use colored::Colorize;
use parsely_lexer::{
    tokens::{self, Token},
    AsSpan, Range, Span,
};
// use parsely_parser::program::Program;

pub mod token_cache;
use token_cache::TokenCache;

pub type Result<T> = std::result::Result<T, Diagnostic>;

pub trait Program {
    fn path(&self) -> &Path;
    fn tokens(&self) -> &[Token];
    fn slice(&self, span: &Span) -> Option<&str>;

    fn len(&self) -> usize {
        self.tokens().len()
    }

    fn tokens_in_range(&self, range: impl Into<Range>) -> &[Token] {
        let range: Range = range.into();
        &self.tokens()[range.0.start..range.0.end]
    }

    fn token(&self, index: usize) -> &Token {
        &self.tokens()[index]
    }
}

// impl Program for parsely_parser::program::Program {
//     fn path(&self) -> &Path {
//         &self.path
//     }

//     fn tokens(&self) -> &[Token] {
//         &self.tokens
//     }

//     fn slice(&self, span: &Span) -> Option<&str> {
//         self.source_lines_slice(span)
//     }
// }

/// Severity of diagnostic
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    Internal,
    Info,
    Warning,
    Error,
}

/// A diagnostic produced by code generation
#[derive(Debug, Clone)]
pub enum Diagnostic {
    /// Error when formatting to buffer
    FormatError(std::fmt::Error),
    /// The provided symbol was not found in the symbol table
    SymbolNotFound(tokens::Ident),
    /// The type doesn't match the context
    IncompatibleType(Span),
    /// The two types don't match
    IncompatibleTypes(Span, Span),

    /// Encountered an unexpected token while parsing
    UnexpectedToken {
        /// Found token 'found'
        found: Token,
        /// Expected this token. This is a string to support tokens with data values.
        expected: String,
    },
    /// No more input (EOF)
    UnexpectedEnd(Span),
    /// Encountered an unexpected size (such as in int32 size)
    UnexpectedSize {
        span: Span,
        found: String,
    },

    PackFile(String, DiagnosticLevel),

    /// A generic message
    Message(String, Span, DiagnosticLevel),
    /// Special meaning that error was logged already
    Caught(Span),
}

impl Diagnostic {
    pub fn level(&self) -> DiagnosticLevel {
        match self {
            Diagnostic::FormatError(_) => DiagnosticLevel::Error,
            Diagnostic::SymbolNotFound(_) => DiagnosticLevel::Error,
            Diagnostic::IncompatibleType(_) => DiagnosticLevel::Error,
            Diagnostic::IncompatibleTypes(_, _) => DiagnosticLevel::Error,

            Diagnostic::UnexpectedToken { .. } => DiagnosticLevel::Error,
            Diagnostic::UnexpectedSize { .. } => DiagnosticLevel::Error,
            Diagnostic::UnexpectedEnd(_) => DiagnosticLevel::Error,

            Diagnostic::PackFile(_, level) => *level,
            Diagnostic::Message(_, _, level) => *level,
            Diagnostic::Caught(_) => DiagnosticLevel::Internal,
        }
    }

    pub fn primary_span(&self) -> Span {
        match self {
            Diagnostic::FormatError(_) => Span::EMPTY,
            Diagnostic::SymbolNotFound(ident) => ident.as_span(),
            Diagnostic::IncompatibleType(span) => *span,
            Diagnostic::IncompatibleTypes(span, _) => *span,

            Diagnostic::UnexpectedToken { found, .. } => found.as_span(),
            Diagnostic::UnexpectedSize { span, .. } => *span,
            Diagnostic::UnexpectedEnd(span) => *span,

            Diagnostic::PackFile(_, _) => Span::EMPTY,
            Diagnostic::Message(_, span, _) => *span,
            Diagnostic::Caught(span) => *span,
        }
    }

    /// Format a diagnostic into 'f'
    /// `module` and `program` should be for the same file
    /// `cache` is token and line information cache
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.level() {
            DiagnosticLevel::Internal => return Ok(()),
            DiagnosticLevel::Info => write!(f, "{}", "info".bold().cyan()),
            DiagnosticLevel::Warning => write!(f, "{}", "warning".bold().yellow()),
            DiagnosticLevel::Error => write!(f, "{}", "error".bold().red()),
        }?;

        write!(f, "{}", ": ".bold())?;

        match self {
            Diagnostic::FormatError(e) => {
                return writeln!(f, "Error when formatting into buffer: {e}");
            }
            Diagnostic::SymbolNotFound(ident) => {
                write!(
                    f,
                    "{}",
                    format!("Symbol `{}` not found in scope", ident.value).bold(),
                )?;
            }
            Diagnostic::IncompatibleType(_) => {
                write!(f, "{}", "Unexpected type".to_string().bold())?;
            }
            Diagnostic::IncompatibleTypes(_, _) => {
                write!(
                    f,
                    "{}",
                    "Types don't match in expression".to_string().bold()
                )?;
            }

            Diagnostic::UnexpectedToken { found, expected } => {
                write!(f, "Expected `{}` but found `{}`", expected, found)?;
            }
            Diagnostic::UnexpectedSize { found, .. } => {
                write!(f, "Expected size but `{}`", found)?;
            }
            Diagnostic::UnexpectedEnd(_) => {
                write!(f, "Unexpected end of input")?;
            }

            Diagnostic::PackFile(msg, _) => write!(f, "Pack File: {}", msg.bold())?,

            Diagnostic::Message(msg, _, _) => {
                write!(f, "{}", msg.bold())?;
            }
            Diagnostic::Caught(_) => (),
        };

        writeln!(f)?;

        Ok(())
    }

    /// Format a diagnostic into 'f'
    /// `module` and `program` should be for the same file
    /// `cache` is token and line information cache
    pub fn format_module<P: Program>(
        &self,
        program: &P,
        cache: &mut TokenCache<P>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        // How many characters on each side of the diagnostic's span should be displayed
        const LINE_PEEK: usize = 3;

        match self.level() {
            DiagnosticLevel::Internal => return Ok(()),
            DiagnosticLevel::Info => write!(f, "{}", "info".bold().cyan()),
            DiagnosticLevel::Warning => write!(f, "{}", "warning".bold().yellow()),
            DiagnosticLevel::Error => write!(f, "{}", "error".bold().red()),
        }?;

        write!(f, "{}", ": ".bold())?;

        let pad = match self {
            Diagnostic::FormatError(e) => {
                return writeln!(f, "Error when formatting into buffer: {e}");
            }
            Diagnostic::SymbolNotFound(ident) => {
                write!(
                    f,
                    "{}",
                    format!("Symbol `{}` not found in scope", ident.value).bold(),
                )?;

                ident.as_span().end.line.to_string().len()
            }
            Diagnostic::IncompatibleType(ty) => {
                write!(f, "{}", "Unexpected type".to_string().bold())?;
                ty.end.line.to_string().len()
            }
            Diagnostic::IncompatibleTypes(left, right) => {
                write!(
                    f,
                    "{}",
                    "Types don't match in expression".to_string().bold()
                )?;
                left.end.line.max(right.end.line).to_string().len()
            }

            Diagnostic::UnexpectedToken { found, expected } => {
                write!(f, "Expected `{}` but found `{}`", expected, found)?;
                found.as_span().end.line.to_string().len()
            }
            Diagnostic::UnexpectedSize { found, span } => {
                write!(f, "Expected size but found `{}`", found)?;
                span.end.line.to_string().len()
            }
            Diagnostic::UnexpectedEnd(span) => {
                write!(f, "Unexpected end of input")?;
                span.end.line.to_string().len()
            }

            Diagnostic::PackFile(msg, _) => {
                write!(f, "Pack File: {}", msg.bold())?;
                0
            }
            Diagnostic::Message(msg, span, _) => {
                write!(f, "{}", msg.bold())?;
                span.end.line.to_string().len()
            }
            Diagnostic::Caught(_) => 0,
        };

        writeln!(f)?;

        let span = self.primary_span();
        writeln!(
            f,
            " {} {}:{}:{}",
            "-->".bold().cyan(),
            program.path().to_str().unwrap(),
            span.start.line + 1,
            span.start.column + 1,
        )?;

        writeln!(f, "{:width$} {}", "", "|".bold().cyan(), width = pad)?;

        // write line number
        let write_ln = |f: &mut std::fmt::Formatter<'_>, line_number: usize| {
            write!(
                f,
                "{:width$} {} ",
                line_number.to_string().bold().white(),
                "|".bold().cyan(),
                width = pad
            )
        };

        // Print one line of source code
        let write_line = |f: &mut std::fmt::Formatter<'_>,
                          highlight: &Span,
                          peeked: &Span,
                          line_offset: usize,
                          line: &str|
         -> std::fmt::Result {
            assert!(peeked.contains(highlight));

            write_ln(f, highlight.start.line + line_offset)?;

            if line_offset == 0 {
                let peek = &line[peeked.start.column..highlight.start.column];
                write!(f, "{}", peek)?;
            }

            if highlight.start.line == highlight.end.line {
                write!(f, "{}", &line[highlight.start.column..highlight.end.column],)?;
            } else if line_offset == 0 {
                write!(f, "{}", &line[highlight.start.column..])?;
            } else if line_offset > 0 {
                write!(f, "{}", &line[..highlight.end.column])?;
            } else {
                unreachable!()
            }

            if line_offset == highlight.end.line - highlight.start.line {
                let peek = &line[highlight.end.column..peeked.end.column];
                write!(f, "{}", peek)?;
            }

            writeln!(f)?;

            write!(f, "{:width$} {} ", "", "|".bold().cyan(), width = pad)?;

            let (swidth, ewidth) = if highlight.start.line == highlight.end.line {
                (
                    highlight.start.column - peeked.start.column,
                    peeked.end.column - highlight.end.column,
                )
            } else if line_offset == 0 {
                (highlight.start.column - peeked.start.column, 0)
            } else if line_offset > 0 {
                (0, peeked.end.column - highlight.end.column)
            } else {
                unreachable!()
            };

            let carets = String::from_utf8(vec![b'^'; highlight.len()]).unwrap();
            let carets = match self.level() {
                DiagnosticLevel::Info => carets.green(),
                DiagnosticLevel::Warning => carets.yellow(),
                DiagnosticLevel::Error => carets.red(),
                _ => carets.white(),
            }
            .bold();

            writeln!(
                f,
                "{0:<swidth$}{1}{0:<ewidth$}",
                "",
                carets,
                swidth = swidth,
                ewidth = ewidth,
            )
        };

        // Print source code using `span`
        let mut write_span = |span: &Span| -> std::fmt::Result {
            let token_index = cache.token_index(&span.start);
            let token_range = cache.line_index(span.start.line);
            assert!(token_range.0.contains(&token_index));

            let start_token = &program.token(
                token_index
                    .saturating_sub(LINE_PEEK)
                    .max(token_range.0.start),
            );

            let end_token_index = token_index
                .saturating_add(LINE_PEEK)
                .min(token_range.0.end - 1);

            // We don't want to include the EOF token
            let end_token_index = if end_token_index >= program.len() - 1 {
                end_token_index - 1
            } else {
                end_token_index
            };

            let end_token = &program.token(end_token_index);

            // The span of the padded tokens
            let peeked_span = start_token.as_span().join(end_token.as_span());

            // str of tokens with padding
            let lines = program
                .slice(&peeked_span)
                .expect("Unable to slice into source file!");

            for (i, line) in lines.lines().enumerate() {
                write_line(f, span, &peeked_span, i, line)?;
            }

            Ok(())
        };

        match self {
            Diagnostic::SymbolNotFound(ident) => {
                let span = ident.as_span();
                assert_eq!(span.start.line, span.end.line); // Identifiers should always be on the same line (i think)

                write_span(&span)?;
            }
            Diagnostic::IncompatibleType(ty) => {
                write_span(ty)?;
            }
            Diagnostic::IncompatibleTypes(left, right) => {
                write_span(left)?;
                write_span(right)?;
            }

            Diagnostic::UnexpectedToken { found, .. } => {
                write_span(&found.as_span())?;
            }
            Diagnostic::UnexpectedSize { span, .. } => {
                write_span(span)?;
            }
            Diagnostic::UnexpectedEnd(span) => {
                write_span(span)?;
            }

            Diagnostic::Message(_, span, _) => {
                write_span(span)?;
            }
            _ => unimplemented!(),
        }

        writeln!(f, "{:width$} {}", "", "|".bold().cyan(), width = pad)?;

        Ok(())
    }
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Diagnostic as std::fmt::Debug>::fmt(self, f)
    }
}

impl From<std::fmt::Error> for Diagnostic {
    fn from(value: std::fmt::Error) -> Self {
        Self::FormatError(value)
    }
}

/// Helper struct for easy formatting
/// This is specific to a module
///
/// # Example
///
/// ```
/// let fmtr = DiagnosticModuleFmt(&module.errors, &module, program);
/// println!("{}", fmtr);
/// ```    
pub struct DiagnosticModuleFmt<'a, P: Program>(pub &'a [Diagnostic], pub &'a P);

impl<P: Program> Display for DiagnosticModuleFmt<'_, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut cache = TokenCache::new(self.1);
        for diag in self.0 {
            diag.format_module(self.1, &mut cache, f)?;
        }
        Ok(())
    }
}

/// Helper struct for easy formatting
///
/// # Example
///
/// ```
/// let fmtr = DiagnosticFmt(&module.errors);
/// println!("{}", fmtr);
/// ```    
pub struct DiagnosticFmt<'a>(pub &'a [Diagnostic]);

impl Display for DiagnosticFmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for diag in self.0 {
            diag.format(f)?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostic {}

trait ErrorHelper {
    fn caught(self) -> Self;
    fn caught_span(self, span: Span) -> Self;
}

impl<T> ErrorHelper for Result<T> {
    fn caught(self) -> Self {
        Err(Diagnostic::Caught(Span::EMPTY))
    }

    fn caught_span(self, span: Span) -> Self {
        Err(Diagnostic::Caught(span))
    }
}

impl ErrorHelper for Diagnostic {
    fn caught(self) -> Self {
        Diagnostic::Caught(Span::EMPTY)
    }

    fn caught_span(self, span: Span) -> Self {
        Diagnostic::Caught(span)
    }
}

/// If `$res` is an error it is logged, otherwise the value is returned
/// If `$or` is provided, it is run instead of logging
#[macro_export]
macro_rules! attempt {
    ($self:expr, $res:expr) => {{
        match $res {
            Ok(o) => Ok(o),
            Err($crate::Diagnostic::Caught(e)) => Err($crate::Diagnostic::Caught(e)),
            Err(e) => {
                $self.push_error(e.clone());
                Err(e)
            }
        }
    }};
    (@stmt $self:expr, $res:expr) => {{
        match $res {
            Ok(_) => Ok(()),
            Err($crate::Diagnostic::Caught(_)) => Ok(()),
            Err(e) => {
                $self.push_error(e.clone());
                Err(e)
            }
        }
    }};
    ($self:expr, $res:expr, {$or:tt}) => {{
        match $res {
            Ok(o) => Ok(o),
            Err($crate::Diagnostic::Caught) => Err($crate::Diagnostic::Caught),
            Err(e) => $or,
        }
    }};
}

/// Log the specified error
#[macro_export]
macro_rules! raise {
    (@not_found => $self:expr, $ident:expr) => {{
        let error = $crate::Diagnostic::SymbolNotFound($ident);
        $self.push_error(error.clone());
        error
    }};
    (@mismatch => $self:expr, $left:expr, $right:expr) => {{
        let error = $crate::Diagnostic::IncompatibleTypes($left, $right);
        $self.push_error(error.clone());
        error
    }};
    (@mismatch => $self:expr, $expr:expr) => {{
        let error = $crate::Diagnostic::IncompatibleType($expr);
        $self.push_error(error.clone());
        error
    }};
    (@log $level:ident => $self:expr, $msg:expr, $span:expr) => {{
        let error =
            $crate::Diagnostic::Message($msg.to_string(), $span, $crate::DiagnosticLevel::$level);
        $self.push_error(error.clone());
        error
    }};
}