macro_rules! define_tokens {
    ($vis:vis enum $token_enum:ident { $($struct_name:ident = $st:expr),*, $(,)* }) => {
        $(
            #[derive(Debug, PartialEq, Eq)]
            $vis struct $struct_name(pub(crate) $crate::Span);

            impl $struct_name {
                pub fn from_span_start(start: $crate::Position) -> Token {
                    Token::$struct_name($struct_name($crate::Span {
                        end: $crate::Position {
                            line: start.line,
                            column: start.column + $st.len(),
                        },
                        start,
                    }))
                }

                pub fn len() -> usize {
                    $st.len()
                }
            }
        )*

        #[derive(Debug, PartialEq)]
        $vis enum $token_enum {
            Ident(Ident),
            Int(Int),
            Float(Float),
            Bool(Bool),
            String(String),
            Char(Char),
           $(
                $struct_name($struct_name)
           ),*
        }

        impl $token_enum {
            pub fn len(&self) -> usize {
                match self {
                    $token_enum::Ident(i) => i.span.len(),
                    $token_enum::Int(i) => i.span.len(),
                    $token_enum::Float(i) => i.span.len(),
                    $token_enum::Bool(i) => i.span.len(),
                    $token_enum::String(i) => i.span.len(),
                    $token_enum::Char(i) => i.span.len(),
                    $(
                        $token_enum::$struct_name(_) => $struct_name::len()
                    ),*
                }
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ident {
    pub value: std::string::String,
    pub(crate) span: crate::Span,
}

impl Ident {
    pub fn from_span_start(raw: &[char], start: crate::Position) -> Token {
        Token::Ident(Ident {
            value: raw.iter().collect(),
            span: crate::Span {
                end: crate::Position {
                    line: start.line,
                    column: start.column + raw.len(),
                },
                start,
            },
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Int {
    pub value: u64,
    pub(crate) span: crate::Span,
}

impl Int {
    pub fn from_span_start(raw: &[char], start: crate::Position) -> Token {
        Token::Int(Int {
            value: raw
                .iter()
                .collect::<std::string::String>()
                .parse()
                .expect("Unexpected value in int!"),
            span: crate::Span {
                end: crate::Position {
                    line: start.line,
                    column: start.column + raw.len(),
                },
                start,
            },
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Float {
    pub value: f64,
    pub(crate) span: crate::Span,
}

impl Float {
    pub fn from_span_start(raw: &[char], start: crate::Position) -> Token {
        Token::Float(Float {
            value: raw
                .iter()
                .collect::<std::string::String>()
                .parse()
                .expect("Unexpected value in float!"),
            span: crate::Span {
                end: crate::Position {
                    line: start.line,
                    column: start.column + raw.len(),
                },
                start,
            },
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Bool {
    pub value: bool,
    pub(crate) span: crate::Span,
}

impl Bool {
    pub fn from_value(value: bool, raw: &[char], start: crate::Position) -> Token {
        Token::Bool(Bool {
            value,
            span: crate::Span {
                end: crate::Position {
                    line: start.line,
                    column: start.column + raw.len(),
                },
                start,
            },
        })
    }

    pub fn from_span_start(raw: &str, start: crate::Position) -> Token {
        Token::Bool(Bool {
            value: raw.parse().expect("Unexpected value in bool!"),
            span: crate::Span {
                end: crate::Position {
                    line: start.line,
                    column: start.column + raw.len(),
                },
                start,
            },
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct String {
    pub value: std::string::String,
    pub(crate) span: crate::Span,
}

impl String {
    pub fn from_span_start(raw: &str, start: crate::Position) -> Token {
        Token::String(String {
            value: raw[1..raw.len() - 1].to_string(),
            span: crate::Span {
                end: crate::Position {
                    line: start.line,
                    column: start.column + raw.len(),
                },
                start,
            },
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Char {
    pub value: char,
    pub(crate) span: crate::Span,
}

impl Char {
    pub fn from_span_start(raw: &str, start: crate::Position) -> Token {
        Token::Char(Char {
            value: raw[1..raw.len() - 1].to_string().chars().next().unwrap(),
            span: crate::Span {
                end: crate::Position {
                    line: start.line,
                    column: start.column + raw.len(),
                },
                start,
            },
        })
    }
}

define_tokens! {
    pub enum Token {
        // Keyword
        Const = "const",
        Continue = "continue",
        Break = "break",
        Else = "else",
        Enum = "enum",
        Export = "export",
        External = "external",
        For = "for",
        If = "if",
        Match = "match",
        Nones = "none",
        Persist = "persist",
        Return = "return",
        Struct = "struct",
        Typedef = "typedef",
        Typeof = "typeof",
        Void = "void",
        While = "while",

        // Punctuation
        Semi = ";",
        Colon = ":",
        Comma = ",",
        Quote = "\"",
        Pound = "#",
        SingleQuote = "'",
        OpenBrace = "{",
        CloseBrace = "}",
        OpenBracket = "[",
        CloseBracket = "]",
        OpenParen = "(",
        CloseParen = ")",

        // Operators
        And = "&",
        AndEq = "&=",
        Assign = "=",
        Eq = "==",
        Dot = ".",
        Gt = ">",
        GtEq = ">=",
        LeftShift = "<<",
        LeftShiftEq = "<<=",
        LogicalAnd = "&&",
        LogicalOr = "||",
        Lt = "<",
        LtEq = "<=",
        Minus = "-",
        MinusEq = "-=",
        Not = "!",
        NotEq = "!=",
        Or = "|",
        Plus = "+",
        PlusEq = "+=",
        Rem = "%",
        RemEq = "%=",
        RightShift = ">>",
        RightShiftEq = ">>=",
        Slash = "/",
        SlashEq = "/=",
        Star = "*",
        StarEq = "*=",
        Xor = "^",
        XorEq = "^=",
    }
}
