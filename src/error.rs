use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    BadLength { path: String, word: String },
    BadChar { path: String, word: String },
    Duplicate { path: String, word: String },
    Empty { path: String },
    TooManyAnswers(usize),
    TooManyGuesses(usize),
    NoCandidate { node: u16 },
    DepthExceeded { node: u16, depth: u32 },
    BadSecret(i32),
    BadFeedback(String),
    NoMatch,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::BadLength { path, word } => write!(f, "{path}: '{word}' has wrong length"),
            Self::BadChar { path, word } => {
                write!(f, "{path}: '{word}' has a non a-z character")
            }
            Self::Duplicate { path, word } => write!(f, "{path}: duplicate word '{word}'"),
            Self::Empty { path } => write!(f, "{path}: no words"),
            Self::TooManyAnswers(n) => write!(f, "too many answers ({n})"),
            Self::TooManyGuesses(n) => write!(f, "guess list too long for u16 indices ({n})"),
            Self::NoCandidate { node } => write!(f, "no candidate guess at node {node}"),
            Self::DepthExceeded { node, depth } => {
                write!(f, "worst case exceeds {depth} guesses at node {node}")
            }
            Self::BadSecret(i) => write!(f, "secret index {i} out of range"),
            Self::BadFeedback(s) => write!(f, "need exactly 5 chars of g/y/b, got '{s}'"),
            Self::NoMatch => write!(f, "no word matches that feedback"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
