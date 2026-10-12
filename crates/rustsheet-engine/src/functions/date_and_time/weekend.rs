//! WORKDAY.INTL / NETWORKDAYS.INTL weekend codes.

#[derive(Debug, Clone, Copy)]
pub(super) enum WeekendPattern {
    SatSun,
    SunMon,
    MonTue,
    TueWed,
    WedThu,
    ThuFri,
    FriSat,
    SunOnly,
    MonOnly,
    TueOnly,
    WedOnly,
    ThuOnly,
    FriOnly,
    SatOnly,
}

impl std::convert::TryFrom<i32> for WeekendPattern {
    type Error = ();
    fn try_from(code: i32) -> Result<Self, Self::Error> {
        Ok(match code {
            1 => Self::SatSun,
            2 => Self::SunMon,
            3 => Self::MonTue,
            4 => Self::TueWed,
            5 => Self::WedThu,
            6 => Self::ThuFri,
            7 => Self::FriSat,
            11 => Self::SunOnly,
            12 => Self::MonOnly,
            13 => Self::TueOnly,
            14 => Self::WedOnly,
            15 => Self::ThuOnly,
            16 => Self::FriOnly,
            17 => Self::SatOnly,
            _ => return Err(()),
        })
    }
}

impl WeekendPattern {
    pub(super) const fn to_mask(self) -> [bool; 7] {
        match self {
            Self::SatSun => [false, false, false, false, false, true, true],
            Self::SunMon => [true, false, false, false, false, false, true],
            Self::MonTue => [true, true, false, false, false, false, false],
            Self::TueWed => [false, true, true, false, false, false, false],
            Self::WedThu => [false, false, true, true, false, false, false],
            Self::ThuFri => [false, false, false, true, true, false, false],
            Self::FriSat => [false, false, false, false, true, true, false],
            Self::SunOnly => [false, false, false, false, false, false, true],
            Self::MonOnly => [true, false, false, false, false, false, false],
            Self::TueOnly => [false, true, false, false, false, false, false],
            Self::WedOnly => [false, false, true, false, false, false, false],
            Self::ThuOnly => [false, false, false, true, false, false, false],
            Self::FriOnly => [false, false, false, false, true, false, false],
            Self::SatOnly => [false, false, false, false, false, true, false],
        }
    }
}
