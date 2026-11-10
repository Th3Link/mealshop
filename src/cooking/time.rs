use jiff::{
    Timestamp,
    civil::DateTime,
    tz::{AmbiguousOffset, TimeZone},
};
use serde::{Deserialize, Serialize};

pub(crate) const MIN_TIME: i64 = 1_577_836_800;
pub(crate) const MAX_TIME: i64 = 4_133_980_799;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Fold {
    #[default]
    Reject,
    Earlier,
    Later,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LocalTime {
    pub(crate) local: String,
    pub(crate) timezone: String,
    #[serde(default)]
    pub(crate) fold: Fold,
}
pub(crate) fn zone(name: &str) -> Result<TimeZone, &'static str> {
    if name.len() > 100
        || name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/_+-".contains(&b))
    {
        return Err("cooking_timezone_invalid");
    }
    TimeZone::get(name).map_err(|_| "cooking_timezone_invalid")
}
pub(crate) fn valid(value: i64) -> bool {
    (MIN_TIME..=MAX_TIME).contains(&value)
}
pub(crate) fn now() -> Result<i64, &'static str> {
    let value = Timestamp::now().as_second();
    if valid(value) {
        Ok(value)
    } else {
        Err("cooking_clock_invalid")
    }
}
pub(crate) fn add(time: i64, seconds: i64) -> Result<i64, &'static str> {
    time.checked_add(seconds)
        .filter(|value| valid(*value))
        .ok_or("cooking_time_invalid")
}
impl LocalTime {
    pub(crate) fn resolve(&self) -> Result<i64, &'static str> {
        if !matches!(self.local.len(), 16 | 19) || self.local.as_bytes().get(10) != Some(&b'T') {
            return Err("cooking_time_invalid");
        }
        let datetime: DateTime = self.local.parse().map_err(|_| "cooking_time_invalid")?;
        let ambiguous = zone(&self.timezone)?.to_ambiguous_zoned(datetime);
        let zoned = match ambiguous.offset() {
            AmbiguousOffset::Gap { .. } => return Err("cooking_time_gap"),
            AmbiguousOffset::Fold { .. } => match self.fold {
                Fold::Reject => return Err("cooking_time_fold"),
                Fold::Earlier => ambiguous.earlier(),
                Fold::Later => ambiguous.later(),
            },
            AmbiguousOffset::Unambiguous { .. } => ambiguous.unambiguous(),
        }
        .map_err(|_| "cooking_time_invalid")?;
        let value = zoned.timestamp().as_second();
        if valid(value) {
            Ok(value)
        } else {
            Err("cooking_time_invalid")
        }
    }
}
pub(crate) fn display(time: i64, timezone: &str) -> Result<String, &'static str> {
    Ok(Timestamp::from_second(time)
        .map_err(|_| "cooking_time_invalid")?
        .to_zoned(zone(timezone)?)
        .to_string())
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn civil_time_gaps_are_rejected_and_folds_require_a_choice() {
        let mut input = LocalTime {
            local: "2026-03-29T02:30".to_owned(),
            timezone: "Europe/Berlin".to_owned(),
            fold: Fold::Earlier,
        };
        assert_eq!(input.resolve(), Err("cooking_time_gap"));
        input.local = "2026-10-25T02:30".to_owned();
        input.fold = Fold::Reject;
        assert_eq!(input.resolve(), Err("cooking_time_fold"));
        input.fold = Fold::Earlier;
        let earlier = input.resolve().unwrap();
        input.fold = Fold::Later;
        let later = input.resolve().unwrap();
        assert_eq!(later - earlier, 3600);
        assert!(
            display(earlier, "Europe/Berlin")
                .unwrap()
                .contains("+02:00")
        );
        assert!(display(later, "Europe/Berlin").unwrap().contains("+01:00"));
        assert_eq!(add(earlier, 172_800).unwrap() - earlier, 172_800);
    }
}
