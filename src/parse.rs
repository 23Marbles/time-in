use std::{
    num::{IntErrorKind, ParseIntError},
    str::FromStr,
};

use chrono::{NaiveDate, NaiveTime};
use thiserror::Error;

use crate::time::{TimeKind, TimeUnitDur};

#[derive(Debug, Clone, Copy)]
pub struct DateParser(pub NaiveDate);

impl FromStr for DateParser {
    type Err = ArgParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let is_locale = s.contains('/');

        if is_locale {
            todo!("Support locale specific formatting, use yyyy-mm-dd for now")
        } else {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(DateParser)
                .map_err(chrono::ParseError::into)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeParser(pub NaiveTime);

impl FromStr for TimeParser {
    type Err = ArgParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for format in ["%-H:%-M:%-S%.f", "%-H:%-M:%-S", "%-H:%-M"] {
            if let Ok(t) = NaiveTime::parse_from_str(s, format).map(TimeParser) {
                return Ok(t);
            }
        }

        // Return the error from the most specific format
        NaiveTime::parse_from_str(s, "%-H:%-M:%-S%.f")
            .map(TimeParser)
            .map_err(chrono::ParseError::into)
    }
}

#[derive(Debug, Clone, Error)]
pub enum ArgParseError {
    #[error("empty string inputted")]
    EmptyStr,
    #[error("no suffix part found in the snippet `0`")]
    NoSuffixPart(String),
    #[error("suffix `{suffix}` not recognized in snippet `{snippet}`{}", if suggestions.is_empty() {
        String::new()
    } else {
        format!("\nmaybe you meant one of:\n - {},", suggestions.join(",\n - "))
    })]
    UnknownSuffix {
        suffix: String,
        snippet: String,
        suggestions: Vec<&'static str>,
    },
    #[error("no digit part found in snippet `{0}`")]
    NoDigitPart(String),
    #[error("digit part specifies a number too large (`{0}`)")]
    DigitTooLarge(String),
    #[error(transparent)]
    Chrono(#[from] chrono::ParseError),
}

/// # Errors
/// Can Error if the snippet cannot be parsed
pub fn split_snippet(snippet: &str) -> Result<(TimeKind, i64), ArgParseError> {
    if snippet.is_empty() {
        return Err(ArgParseError::EmptyStr);
    }

    let split = if let Some(s) = snippet.strip_prefix('-') {
        s.find(|ch: char| !ch.is_ascii_digit())
            .map(|n| if n != 0 { n + '-'.len_utf8() } else { 0 })
    } else {
        snippet.find(|ch: char| !ch.is_ascii_digit())
    }
    .ok_or(ArgParseError::NoSuffixPart(snippet.to_owned()))?;

    let (prefix, suffix) = snippet.split_at(split);

    let count: i64 = prefix.parse().map_err(|e: ParseIntError| match e.kind() {
        IntErrorKind::Empty => ArgParseError::NoDigitPart(snippet.to_owned()),
        IntErrorKind::InvalidDigit => {
            unreachable!("must be ascii digit optionally with a single '-' at the start")
        }
        IntErrorKind::PosOverflow => ArgParseError::DigitTooLarge(prefix.to_owned()),
        IntErrorKind::NegOverflow => unreachable!("does not have a negative part"),
        IntErrorKind::Zero => unreachable!("is not a NonZero<T>"),
        _ => unreachable!(),
    })?;

    Ok((
        match suffix {
            "y" | "years" | "year" => TimeKind::Years,
            "mo" | "months" | "month" => TimeKind::Months,
            "w" | "weeks" | "week" => TimeKind::Weeks,
            "d" | "days" | "day" => TimeKind::Days,
            "h" | "hours" | "hour" => TimeKind::Hours,
            "m" | "mins" | "minutes" | "min" | "minute" => TimeKind::Mins,
            "s" | "secs" | "seconds" | "sec" | "second" => TimeKind::Secs,
            s => {
                return Err(ArgParseError::UnknownSuffix {
                    suffix: s.to_owned(),
                    snippet: snippet.to_owned(),
                    suggestions: {
                        let mut vec = Vec::new();

                        if s.contains('y') {
                            vec.push("years");
                        }

                        if s.contains('w') {
                            vec.push("weeks");
                        }

                        if s.contains('d') {
                            vec.push("days");
                        }

                        if s.contains('h') {
                            vec.push("hours");
                        }

                        if s.contains('m') {
                            vec.push("mins");
                            vec.push("months");
                        }

                        let trimmed_last = &s[..s
                            .char_indices()
                            .next_back()
                            .ok_or(ArgParseError::NoSuffixPart(snippet.to_owned()))?
                            .0];

                        if trimmed_last.contains('s') {
                            vec.push("secs");
                        }

                        vec
                    },
                });
            }
        },
        count,
    ))
}

impl FromStr for TimeUnitDur {
    type Err = ArgParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        split_snippet(s).map(|(time_kind, duration)| Self {
            duration,
            time_kind,
        })
    }
}
