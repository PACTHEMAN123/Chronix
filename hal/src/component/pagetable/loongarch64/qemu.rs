#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageLevel {
    Huge = 0,
    Big = 1,
    Middle = 2,
    Small = 3
}

impl PageLevel {
    pub const fn page_count(self) -> usize {
        match self {
            PageLevel::Huge => 512 * 512 * 512,
            PageLevel::Big => 512 * 512,
            PageLevel::Middle => 512,
            PageLevel::Small => 1,
        }
    }

    pub const fn lower(self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Big,
            PageLevel::Big => PageLevel::Middle,
            PageLevel::Middle => PageLevel::Small,
            PageLevel::Small => PageLevel::Small,
        }
    }

    pub const fn higher(self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Huge,
            PageLevel::Big => PageLevel::Huge,
            PageLevel::Middle => PageLevel::Big,
            PageLevel::Small => PageLevel::Middle,
        }
    }

    pub const fn lowest(self) -> bool {
        match self {
            PageLevel::Small => true,
            _ => false
        }
    }

    pub const fn highest(self) -> bool {
        match self {
            PageLevel::Huge => true,
            _ => false
        }
    }

    pub const fn from_count(count: usize) -> Option<Self> {
        match count {
            0x1 => Some(Self::Small),
            0x200 => Some(Self::Middle),
            0x40000 => Some(Self::Big),
            0x8000000 => Some(Self::Huge),
            _ => None
        }
    }
}

impl From<usize> for PageLevel {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Huge,
            1 => Self::Big,
            2 => Self::Middle,
            3 => Self::Small,
            _ => panic!("unsupport Page Level")
        }
    }
}
