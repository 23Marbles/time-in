use std::{
    fmt::Display,
    num::{IntErrorKind, ParseIntError},
};

use num_integer::Integer;
use thiserror::Error;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeDeltaFormatter {
    secs: i64,
}

impl Display for TimeDeltaFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn with_suffix_if_not_zero(num: i64, suffix: &str) -> Option<String> {
            if num == 0 {
                return None;
            }

            Some(format!("{num} {suffix}{}", if num == 1 {
                ""
            } else {
                "s"
            }))
        }

        let rem = self.secs;

        #[cfg(feature = "show_weeks")]
        let (weeks, rem) = rem.div_rem(&WEEK_IN_SECS);
        let (days, rem) = rem.div_rem(&DAY_IN_SECS);
        let (hours, rem) = rem.div_rem(&HOUR_IN_SECS);
        let (mins, secs) = rem.div_rem(&MIN_IN_SECS);

        let iter = [
            #[cfg(feature = "show_weeks")]
            with_suffix_if_not_zero(weeks, "week"),
            with_suffix_if_not_zero(days, "day"),
            with_suffix_if_not_zero(hours, "hour"),
            with_suffix_if_not_zero(mins, "minute"),
            with_suffix_if_not_zero(secs, "second"),
        ];

        let vec: Vec<String> = iter.into_iter().flatten().collect();

        match vec.len() {
            0 => write!(f, "0 seconds"),
            1 => write!(f, "{}", vec[0]),
            2 => write!(f, "{} and {}", vec[0], vec[1]),
            len => {
                write!(f, "{}", vec[0])?;

                for (i, entry) in vec.into_iter().enumerate().skip(1) {
                    if i == len - 1 {
                        write!(f, " and ")?;
                    } else {
                        write!(f, ", ")?;
                    }

                    write!(f, "{entry}")?;
                }

                Ok(())
            }
        }
    }
}

impl TimeDeltaFormatter {
    pub fn seconds(secs: i64) -> Self {
        Self { secs }
    }
}

pub const WEEK_IN_SECS: i64 = DAY_IN_SECS * 7;
pub const DAY_IN_SECS: i64 = HOUR_IN_SECS * 24;
pub const HOUR_IN_SECS: i64 = MIN_IN_SECS * 60;
pub const MIN_IN_SECS: i64 = 60;

#[derive(Debug, Clone, Error)]
pub enum ArgParseError {
    #[error("empty string inputted")]
    EmptyStr,
    #[error("no suffix part found in the snippet `0`")]
    NoSuffixPart(String),
    #[error("suffix `{suffix}` not recognized in snippet `{snippet}`{}", if suggestions.is_empty() {
        String::new()
    } else {
        format!("\nmaybe you meant one of:\n{}", suggestions.join(",\n - "))
    })]
    UnknownSuffix {
        suffix: String,
        snippet: String,
        suggestions: Vec<String>,
    },
    #[error("no digit part found in snippet `{0}`")]
    NoDigitPart(String),
    #[error("digit part specifies a number too large (`{0}`)")]
    DigitTooLarge(String),
}

pub enum TimeKind {
    Secs,
    Mins,
    Hours,
    Days,
    Weeks,
}

pub fn split_snippet(snippet: &str) -> Result<(TimeKind, i64), ArgParseError> {
    if snippet.is_empty() {
        return Err(ArgParseError::EmptyStr);
    }

    let split = snippet
        .find(|ch: char| !ch.is_ascii_digit())
        .ok_or(ArgParseError::NoSuffixPart(snippet.to_owned()))?;

    let (prefix, suffix) = snippet.split_at(split);

    let count: i64 = prefix.parse().map_err(|e: ParseIntError| match e.kind() {
        IntErrorKind::Empty => ArgParseError::NoDigitPart(snippet.to_owned()),
        IntErrorKind::InvalidDigit => unreachable!("must be ascii digit"),
        IntErrorKind::PosOverflow => ArgParseError::DigitTooLarge(prefix.to_owned()),
        IntErrorKind::NegOverflow => unreachable!("does not have a negative part"),
        IntErrorKind::Zero => unreachable!("is not a NonZero<T>"),
        _ => unreachable!(),
    })?;

    Ok((
        match suffix {
            "w" | "weeks" => TimeKind::Weeks,
            "d" | "days" => TimeKind::Days,
            "h" | "hours" => TimeKind::Hours,
            "m" | "mins" | "minutes" => TimeKind::Mins,
            "s" | "secs" | "seconds" => TimeKind::Secs,
            s => {
                return Err(ArgParseError::UnknownSuffix {
                    suffix: s.to_owned(),
                    snippet: snippet.to_owned(),
                    suggestions: {
                        let mut vec = Vec::new();

                        if s.contains('w') {
                            vec.push("weeks".to_string());
                        }

                        if s.contains('d') {
                            vec.push("days".to_string());
                        }

                        if s.contains('h') {
                            vec.push("hours".to_string());
                        }

                        if s.contains('m') {
                            vec.push("mins".to_string());
                        }

                        let trimmed_last = &s[..s
                            .char_indices()
                            .next_back()
                            .ok_or(ArgParseError::NoSuffixPart(snippet.to_owned()))?
                            .0];

                        if trimmed_last.contains('s') {
                            vec.push("secs".to_owned());
                        }

                        vec
                    },
                });
            }
        },
        count,
    ))
}
