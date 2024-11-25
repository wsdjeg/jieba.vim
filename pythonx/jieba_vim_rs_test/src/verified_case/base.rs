use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Vim modes.
pub enum Mode {
    Normal,
    VisualChar,
    VisualLine,
    VisualBlock,
    Operator,
}

impl FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "n" => Ok(Self::Normal),
            "c" | "xc" => Ok(Self::VisualChar),
            "l" | "xl" => Ok(Self::VisualLine),
            "b" | "xb" => Ok(Self::VisualBlock),
            "o" => Ok(Self::Operator),
            s => Err(ParseModeError(s.into())),
        }
    }
}

#[derive(Debug)]
pub struct ParseModeError(String);

impl fmt::Display for ParseModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unsupported mode: `{}`", self.0)
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "n"),
            Self::VisualChar => write!(f, "xc"),
            Self::VisualLine => write!(f, "xl"),
            Self::VisualBlock => write!(f, "xb"),
            Self::Operator => write!(f, "o"),
        }
    }
}

impl Mode {
    pub fn visual_prefix(&self) -> Option<&'static str> {
        match self {
            Self::VisualChar => Some("v"),
            Self::VisualLine => Some("V"),
            Self::VisualBlock => Some(r"\<C-v>"),
            _ => None,
        }
    }
}

/// Vim word motions. The enclosed bool is `true` if the motion is a word
/// motion, otherwise a WORD motion.
pub enum Motion {
    W(bool),
    E(bool),
    B(bool),
    Ge(bool),
}

impl FromStr for Motion {
    type Err = ParseMotionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "w" => Ok(Self::W(true)),
            "W" => Ok(Self::W(false)),
            "e" => Ok(Self::E(true)),
            "E" => Ok(Self::E(false)),
            "b" => Ok(Self::B(true)),
            "B" => Ok(Self::B(false)),
            "ge" => Ok(Self::Ge(true)),
            "gE" => Ok(Self::Ge(false)),
            s => Err(ParseMotionError(s.into())),
        }
    }
}

#[derive(Debug)]
pub struct ParseMotionError(String);

impl fmt::Display for ParseMotionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unsupported motion: `{}`", self.0)
    }
}

impl fmt::Display for Motion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::W(true) => write!(f, "w"),
            Self::W(false) => write!(f, "W"),
            Self::E(true) => write!(f, "e"),
            Self::E(false) => write!(f, "E"),
            Self::B(true) => write!(f, "b"),
            Self::B(false) => write!(f, "B"),
            Self::Ge(true) => write!(f, "ge"),
            Self::Ge(false) => write!(f, "gE"),
        }
    }
}

/// Supported Vim operators in tests.
pub enum Operator {
    NoOp,
    Yank,
    Delete,
    Change,
}

impl FromStr for Operator {
    type Err = ParseOperatorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Self::NoOp),
            "y" => Ok(Self::Yank),
            "d" => Ok(Self::Delete),
            "c" => Ok(Self::Change),
            s => Err(ParseOperatorError(s.into())),
        }
    }
}

#[derive(Debug)]
pub struct ParseOperatorError(String);

impl fmt::Display for ParseOperatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unsupported operator: `{}`", self.0)
    }
}

/// Count of Vim motions. The count is implicitly 1 if the enclosed u32 is 0.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Count(u64);

impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 > 0 {
            write!(f, "{}", self.0)
        } else {
            Ok(())
        }
    }
}

impl From<u64> for Count {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Option<u64>> for Count {
    fn from(value: Option<u64>) -> Self {
        Self(value.unwrap_or(0))
    }
}

impl Count {
    pub fn explicit(&self) -> u64 {
        if self.0 == 0 {
            1
        } else {
            self.0
        }
    }
}
