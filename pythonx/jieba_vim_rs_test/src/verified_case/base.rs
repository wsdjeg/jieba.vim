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

/// Vim word motions.
pub enum Motion {
    SmallW,
    LargeW,
    SmallE,
    LargeE,
    SmallB,
    LargeB,
    SmallGe,
    LargeGe,
}

impl FromStr for Motion {
    type Err = ParseMotionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "w" => Ok(Self::SmallW),
            "W" => Ok(Self::LargeW),
            "e" => Ok(Self::SmallE),
            "E" => Ok(Self::LargeE),
            "b" => Ok(Self::SmallB),
            "B" => Ok(Self::LargeB),
            "ge" => Ok(Self::SmallGe),
            "gE" => Ok(Self::LargeGe),
            s => Err(ParseMotionError(s.into())),
        }
    }
}

#[derive(Debug)]
pub struct ParseMotionError(String);

impl fmt::Display for Motion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SmallW => write!(f, "w"),
            Self::LargeW => write!(f, "W"),
            Self::SmallE => write!(f, "e"),
            Self::LargeE => write!(f, "E"),
            Self::SmallB => write!(f, "b"),
            Self::LargeB => write!(f, "B"),
            Self::SmallGe => write!(f, "ge"),
            Self::LargeGe => write!(f, "gE"),
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
