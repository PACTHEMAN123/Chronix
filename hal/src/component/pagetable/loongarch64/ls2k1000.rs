
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageLevel {
    Huge = 0,
    Big = 1,
    Small = 2
}

impl PageLevel {
    pub const fn page_count(self) -> usize {
        match self {
            PageLevel::Huge => 512 * 512,
            PageLevel::Big => 512,
            PageLevel::Small => 1,
        }
    }

    pub const fn lower(self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Big,
            PageLevel::Big  => PageLevel::Small,
            PageLevel::Small => PageLevel::Small,
        }
    }

    pub const fn higher(self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Huge,
            PageLevel::Big => PageLevel::Huge,
            PageLevel::Small => PageLevel::Big,
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
            0x200 => Some(Self::Big),
            0x40000 => Some(Self::Huge),
            _ => None
        }
    }
}

impl From<usize> for PageLevel {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Huge,
            1 => Self::Big,
            2 => Self::Small,
            _ => panic!("unsupport Page Level")
        }
    }
}


