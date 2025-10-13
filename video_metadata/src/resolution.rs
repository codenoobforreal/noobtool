use std::{
    cmp::{Ordering, max},
    fmt,
    str::FromStr,
};

/// 朝向枚举
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Orientation {
    Landscape, // 横屏
    Portrait,  // 竖屏
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Resolution {
    /// 4k
    Uhd,
    /// 4k
    Vuhd,
    /// 2k
    Qhd,
    /// 2k
    Vqhd,
    /// 1080p
    Fhd,
    /// 1080p
    Vfhd,
    /// 720p
    Hd,
    /// 720p
    Vhd,
    Arbitrary {
        width: u16,
        height: u16,
    },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ResolutionError {
    #[error("width or height can't be zero")]
    Zero,
    #[error("delimiter x not found")]
    NoDelimiterX,
    #[error("width is missing")]
    MissingWidth,
    #[error("height is missing")]
    MissingHeight,
    #[error("error parsing {0}")]
    Parse(String),
}

impl FromStr for Resolution {
    type Err = ResolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('x') {
            Some(("", _)) => Err(ResolutionError::MissingWidth),
            Some((_, "")) => Err(ResolutionError::MissingHeight),
            Some((w, h)) => {
                let width = w
                    .parse::<f32>()
                    .map_err(|_| ResolutionError::Parse(w.to_string()))?
                    as u16;
                let height = h
                    .parse::<f32>()
                    .map_err(|_| ResolutionError::Parse(h.to_string()))?
                    as u16;

                Ok(Self::new(width, height)?)
            }
            None => Err(ResolutionError::NoDelimiterX),
        }
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Self::Fhd
    }
}

impl Resolution {
    pub fn new(width: u16, height: u16) -> Result<Self, ResolutionError> {
        if width == 0 || height == 0 {
            return Err(ResolutionError::Zero);
        }
        match (width, height) {
            (3_840, 2_160) => Ok(Self::Uhd),
            (2_160, 3_840) => Ok(Self::Vuhd),
            (2_560, 1_440) => Ok(Self::Qhd),
            (1_440, 2_560) => Ok(Self::Vqhd),
            (1_920, 1_080) => Ok(Self::Fhd),
            (1_080, 1_920) => Ok(Self::Vfhd),
            (1_280, 720) => Ok(Self::Hd),
            (720, 1_280) => Ok(Self::Vhd),
            _ => Ok(Self::Arbitrary { width, height }),
        }
    }

    pub fn pixels(&self) -> u32 {
        match self {
            Resolution::Uhd | Resolution::Vuhd => 8_294_400,
            Resolution::Qhd | Resolution::Vqhd => 3_686_400,
            Resolution::Fhd | Resolution::Vfhd => 2_073_600,
            Resolution::Hd | Resolution::Vhd => 921_600,
            &Resolution::Arbitrary { width, height } => (width as u32) * (height as u32),
        }
    }

    pub fn width(&self) -> u16 {
        match self {
            Resolution::Uhd => 3_840,
            Resolution::Vuhd => 2_160,
            Resolution::Qhd => 2_560,
            Resolution::Vqhd => 1_440,
            Resolution::Fhd => 1_920,
            Resolution::Vfhd => 1_080,
            Resolution::Hd => 1_280,
            Resolution::Vhd => 720,
            &Resolution::Arbitrary { width, height: _ } => width,
        }
    }

    pub fn height(&self) -> u16 {
        match self {
            Resolution::Uhd => 2_160,
            Resolution::Vuhd => 3_840,
            Resolution::Qhd => 1_440,
            Resolution::Vqhd => 2_560,
            Resolution::Fhd => 1_080,
            Resolution::Vfhd => 1_920,
            Resolution::Hd => 720,
            Resolution::Vhd => 1_280,
            &Resolution::Arbitrary { width: _, height } => height,
        }
    }

    /// 判断朝向
    pub fn get_orientation(&self) -> Orientation {
        match self.width().cmp(&self.height()) {
            Ordering::Greater | Ordering::Equal => Orientation::Landscape,
            Ordering::Less => Orientation::Portrait,
        }
    }

    /// 获取主要的缩放尺寸（取宽高中的较大值）
    pub fn get_primary_dimension(&self) -> u16 {
        max(self.width(), self.height())
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Resolution::Uhd => write!(f, "3840x2160"),
            Resolution::Vuhd => write!(f, "2160x3840"),
            Resolution::Qhd => write!(f, "2560x1440"),
            Resolution::Vqhd => write!(f, "1440x2560"),
            Resolution::Fhd => write!(f, "1920x1080"),
            Resolution::Vfhd => write!(f, "1080x1920"),
            Resolution::Hd => write!(f, "1280x720"),
            Resolution::Vhd => write!(f, "720x1280"),
            Resolution::Arbitrary { width, height } => write!(f, "{}x{}", width, height),
        }
    }
}
