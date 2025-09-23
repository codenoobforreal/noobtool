use std::fmt;
use std::str::FromStr;

/// https://x265.readthedocs.io/en/master/presets.html#presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Preset {
    // Ultrafast,
    // Superfast,
    Veryfast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    Veryslow,
    // Placebo,
}

impl Default for Preset {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PresetParseError {
    #[error("no such preset: {0}")]
    NoSuchPreset(String),
}

impl FromStr for Preset {
    type Err = PresetParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "veryfast" => Ok(Self::Veryfast),
            "faster" => Ok(Self::Faster),
            "fast" => Ok(Self::Fast),
            "medium" => Ok(Self::Medium),
            "slow" => Ok(Self::Slow),
            "slower" => Ok(Self::Slower),
            "veryslow" => Ok(Self::Veryslow),
            _ => Err(PresetParseError::NoSuchPreset(s.to_string())),
        }
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Preset::Veryfast => write!(f, "veryfast"),
            Preset::Faster => write!(f, "faster"),
            Preset::Fast => write!(f, "fast"),
            Preset::Medium => write!(f, "medium"),
            Preset::Slow => write!(f, "slow"),
            Preset::Slower => write!(f, "slower"),
            Preset::Veryslow => write!(f, "veryslow"),
        }
    }
}
