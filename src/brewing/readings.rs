use super::{
    model::{Calibration, Reading, ReadingInput, ReadingSource, ReadingUnit},
    validation,
};
use crate::cooking::{self, model::Data};

pub(super) fn input(
    value: &ReadingInput,
    data: &Data,
    run_id: &str,
    now: i64,
) -> Result<i64, &'static str> {
    let run = data.run(run_id)?;
    if !data.brewing.batches.contains_key(run_id)
        || value
            .phase_id
            .as_ref()
            .is_some_and(|id| !run.steps.iter().any(|s| &s.stage.id == id))
        || (!value.notes.is_empty() && !cooking::model::text(&value.notes, 2000))
    {
        return Err("brewing_reading_invalid");
    }
    validation::measurement(&value.measurement)?;
    let at = value.measured_at.resolve()?;
    if at > now {
        return Err("brewing_measurement_future");
    }
    if let ReadingSource::Instrument {
        device_id, name, ..
    } = &value.source
    {
        if !cooking::model::text(device_id, 120) || !cooking::model::text(name, 200) {
            return Err("brewing_reading_invalid");
        }
    }
    if let Calibration::Declared {
        details,
        calibrated_at,
        reference_temperature,
        source_url,
    } = &value.calibration
    {
        if !cooking::model::text(details, 2000)
            || calibrated_at
                .as_ref()
                .is_some_and(|t| t.resolve().is_err() || t.resolve().is_ok_and(|v| v > at))
            || source_url.as_ref().is_some_and(|url| {
                (crate::recipe_origin::Origin::Imported {
                    source_url: url.clone(),
                })
                .validate()
                .is_err()
            })
        {
            return Err("brewing_calibration_invalid");
        }
        if let Some(t) = reference_temperature {
            if !matches!(t.unit, ReadingUnit::Celsius | ReadingUnit::Fahrenheit) {
                return Err("brewing_calibration_invalid");
            }
            validation::measurement(t)?;
        }
    }
    Ok(at)
}
pub(super) fn append(
    data: &mut Data,
    run_id: &str,
    value: &ReadingInput,
    corrects: Option<&str>,
    reason: Option<&str>,
    actor: &str,
    now: i64,
) -> Result<String, &'static str> {
    let measured_instant = input(value, data, run_id, now)?;
    match (corrects, reason) {
        (Some(id), Some(reason)) if cooking::model::text(reason, 1000) => {
            if !data
                .brewing
                .readings
                .iter()
                .any(|r| r.id == id && r.run_id == run_id)
                || data
                    .brewing
                    .readings
                    .iter()
                    .any(|r| r.corrects.as_deref() == Some(id))
            {
                return Err("brewing_reading_correction_invalid");
            }
        }
        (None, None) => {}
        _ => return Err("brewing_reading_correction_invalid"),
    }
    let id = data.allocate("brew-reading")?;
    data.brewing.readings.push(Reading {
        id: id.clone(),
        run_id: run_id.to_owned(),
        input: value.clone(),
        measured_instant,
        actor: actor.to_owned(),
        recorded_at: now,
        corrects: corrects.map(str::to_owned),
        reason: reason.map(str::to_owned),
    });
    Ok(id)
}
pub(super) fn validate(data: &Data) -> Result<(), &'static str> {
    let mut ids = std::collections::BTreeSet::new();
    let mut corrected = std::collections::BTreeSet::new();
    for r in &data.brewing.readings {
        if !cooking::store::allocated(data, &r.id, "brew-reading-")
            || !cooking::actor_valid(&r.actor)
            || !cooking::time::valid(r.recorded_at)
            || input(&r.input, data, &r.run_id, r.recorded_at)? != r.measured_instant
        {
            return Err("brewing_data_invalid");
        }
        match (&r.corrects, &r.reason) {
            (None, None) => {}
            (Some(id), Some(reason))
                if cooking::model::text(reason, 1000)
                    && ids.contains(id)
                    && corrected.insert(id)
                    && data.brewing.readings.iter().any(|old| {
                        &old.id == id && old.run_id == r.run_id && old.recorded_at <= r.recorded_at
                    }) => {}
            _ => return Err("brewing_data_invalid"),
        }
        if !ids.insert(&r.id) {
            return Err("brewing_data_invalid");
        }
    }
    Ok(())
}
