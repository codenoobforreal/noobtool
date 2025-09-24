use std::{fmt, str::FromStr};

use crate::ThumbnailError;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct Grid {
    pub(crate) row: u8,
    pub(crate) col: u8,
}

impl Grid {
    pub fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }

    pub fn count(&self) -> u16 {
        self.row as u16 * self.col as u16
    }
}

impl FromStr for Grid {
    type Err = ThumbnailError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('x') {
            Some(("", _)) => Err(ThumbnailError::MissingRow),
            Some((_, "")) => Err(ThumbnailError::MissingCol),
            Some((r, c)) => {
                let row = r
                    .parse::<u8>()
                    .map_err(|_| ThumbnailError::Parse(r.to_string()))?;
                let col = c
                    .parse::<u8>()
                    .map_err(|_| ThumbnailError::Parse(c.to_string()))?;

                Ok(Self::new(row, col))
            }
            None => Err(ThumbnailError::NoDelimiterX),
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.row, self.col)
    }
}
