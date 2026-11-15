use super::{Plans, access, date, error, work};
use crate::household::{self, Shared};
use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::post,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Schedule {
    recurring: Vec<Recurring>,
    overrides: Vec<Override>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Recurring {
    member_id: String,
    weekdays: Vec<u8>,
    meals: Vec<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Override {
    date: String,
    meal: String,
    member_ids: Vec<String>,
}
pub(super) fn meal(value: &str) -> bool {
    matches!(value, "breakfast" | "lunch" | "snack" | "dinner")
}
impl Schedule {
    pub(super) fn validate(&self) -> Result<(), &'static str> {
        let mut people = BTreeSet::new();
        let mut dates = BTreeSet::new();
        if self.recurring.len() > 100 || self.overrides.len() > 366 {
            return Err("attendance_capacity");
        }
        for r in &self.recurring {
            if r.member_id.is_empty()
                || r.member_id.len() > 100
                || !people.insert(&r.member_id)
                || r.weekdays.len() > 7
                || r.weekdays.iter().any(|d| *d > 6)
                || r.weekdays.iter().collect::<BTreeSet<_>>().len() != r.weekdays.len()
                || r.meals.len() > 4
                || r.meals.iter().any(|m| !meal(m))
                || r.meals.iter().collect::<BTreeSet<_>>().len() != r.meals.len()
            {
                return Err("attendance_invalid");
            }
        }
        for o in &self.overrides {
            if !date(&o.date)
                || !meal(&o.meal)
                || !dates.insert((&o.date, &o.meal))
                || o.member_ids.len() > 100
                || o.member_ids
                    .iter()
                    .any(|id| id.is_empty() || id.len() > 100)
                || o.member_ids.iter().collect::<BTreeSet<_>>().len() != o.member_ids.len()
            {
                return Err("attendance_invalid");
            }
        }
        Ok(())
    }
    pub(super) fn available(&self, id: &str, date: &str, meal: &str) -> bool {
        if let Some(o) = self
            .overrides
            .iter()
            .find(|o| o.date == date && o.meal == meal)
        {
            return o.member_ids.iter().any(|m| m == id);
        }
        self.recurring
            .iter()
            .find(|r| r.member_id == id)
            .is_none_or(|r| {
                date.parse::<jiff::civil::Date>().is_ok_and(|d| {
                    u8::try_from(d.weekday().to_monday_zero_offset())
                        .is_ok_and(|d| r.weekdays.contains(&d))
                }) && r.meals.iter().any(|m| m == meal)
            })
    }
    fn references(&self) -> impl Iterator<Item = &String> {
        self.recurring
            .iter()
            .map(|r| &r.member_id)
            .chain(self.overrides.iter().flat_map(|o| &o.member_ids))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    expected_revision: u64,
    attendance: Schedule,
}
pub(super) fn routes() -> Router<Shared> {
    Router::new().route("/api/household/mealplan/attendance", post(save))
}
async fn save(
    State(state): State<Shared>,
    headers: HeaderMap,
    input: Result<Json<Input>, JsonRejection>,
) -> Response {
    if !access(&state, &headers, true) {
        return household::reply(
            StatusCode::FORBIDDEN,
            json!({"code":"household_write_required"}),
        );
    }
    let Ok(Json(input)) = input else {
        return error("attendance_invalid");
    };
    work(state, move |_, profiles, _, plans, _| {
        input.attendance.validate()?;
        if input.expected_revision != plans.data.revision {
            return Err("revision_conflict");
        }
        if input
            .attendance
            .references()
            .any(|id| !profiles.members.iter().any(|m| &m.id == id))
        {
            return Err("unknown_member");
        }
        let revision = plans
            .data
            .revision
            .checked_add(1)
            .ok_or("mealplan_capacity")?;
        let mut meals = plans.data.meals.clone();
        for plan in &mut meals {
            plan.reviewed_profile_revision = None;
        }
        plans.save(Plans {
            revision,
            attendance: input.attendance,
            meals,
            ..plans.data.clone()
        })?;
        Ok(json!({"revision":revision,"attendance":plans.data.attendance}))
    })
    .await
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn explicit_date_overrides_recurrence_on_civil_dst_and_leap_dates() {
        let schedule: Schedule = serde_json::from_value(json!({"recurring":[
            {"member_id":"one","weekdays":[6],"meals":["dinner"]}],
            "overrides":[{"date":"2026-10-25","meal":"dinner","member_ids":[]},
            {"date":"2028-02-29","meal":"lunch","member_ids":["one"]}]}))
        .unwrap();
        schedule.validate().unwrap();
        assert!(schedule.available("one", "2026-03-29", "dinner"));
        assert!(!schedule.available("one", "2026-10-25", "dinner"));
        assert!(schedule.available("one", "2028-02-29", "lunch"));
        assert!(!schedule.available("one", "2028-02-29", "dinner"));
        assert!(schedule.available("two", "2026-11-14", "lunch"));
    }
}
