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
    SmallW(u32),
    LargeW(u32),
    SmallE(u32),
    LargeE(u32),
    SmallB(u32),
    LargeB(u32),
    SmallGe(u32),
    LargeGe(u32),
}

impl FromStr for Motion {
    type Err = ParseMotionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "w" => Ok(Self::SmallW(0)),
            "W" => Ok(Self::LargeW(0)),
            "e" => Ok(Self::SmallE(0)),
            "E" => Ok(Self::LargeE(0)),
            "b" => Ok(Self::SmallB(0)),
            "B" => Ok(Self::LargeB(0)),
            "ge" => Ok(Self::SmallGe(0)),
            "gE" => Ok(Self::LargeGe(0)),
            s => Err(ParseMotionError(s.into())),
        }
    }
}

impl Motion {
    pub fn with_count(self, count: u32) -> Self {
        match self {
            Self::SmallW(_) => Self::SmallW(count),
            Self::LargeW(_) => Self::LargeW(count),
            Self::SmallE(_) => Self::LargeE(count),
            Self::LargeE(_) => Self::LargeE(count),
            Self::SmallB(_) => Self::LargeB(count),
            Self::LargeB(_) => Self::LargeB(count),
            Self::SmallGe(_) => Self::SmallGe(count),
            Self::LargeGe(_) => Self::LargeGe(count),
        }
    }
}

#[derive(Debug)]
pub struct ParseMotionError(String);

impl fmt::Display for Motion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SmallW(c) if *c == 0 => write!(f, "w"),
            Self::SmallW(c) => write!(f, "{}w", c),
            Self::LargeW(c) if *c == 0 => write!(f, "W"),
            Self::LargeW(c) => write!(f, "{}W", c),
            Self::SmallE(c) if *c == 0 => write!(f, "e"),
            Self::SmallE(c) => write!(f, "{}e", c),
            Self::LargeE(c) if *c == 0 => write!(f, "E"),
            Self::LargeE(c) => write!(f, "{}E", c),
            Self::SmallB(c) if *c == 0 => write!(f, "b"),
            Self::SmallB(c) => write!(f, "{}b", c),
            Self::LargeB(c) if *c == 0 => write!(f, "B"),
            Self::LargeB(c) => write!(f, "{}B", c),
            Self::SmallGe(c) if *c == 0 => write!(f, "ge"),
            Self::SmallGe(c) => write!(f, "{}ge", c),
            Self::LargeGe(c) if *c == 0 => write!(f, "gE"),
            Self::LargeGe(c) => write!(f, "{}gE", c),
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
