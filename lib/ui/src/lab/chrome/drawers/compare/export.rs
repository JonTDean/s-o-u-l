//! Immutable CSV bundles retain sample epochs, ticks, units and exact captured fields.
use super::*;
use crate::theme::widgets::pill;
use std::{fmt::Write, fs, path::Path};
pub(super) fn show(ui: &mut Ui, state: &mut Comparison, data: &LabData) {
    if !pill(
        ui,
        "Export metrics and difference CSV",
        false,
        state.capture.pending.is_none(),
    )
    .clicked()
    {
        return;
    }
    state.status =
        write(state, data).map_or_else(|e| e, |()| "CSV bundle exported to a new folder".into());
}
fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
fn write(state: &Comparison, data: &LabData) -> Result<(), String> {
    let pair = state
        .left
        .zip(state.right)
        .filter(|(l, r)| l != r)
        .ok_or("Choose two different channels")?;
    let mut metrics = String::from("channel,kind,metric,unit,epoch,tick,value\n");
    for id in [pair.0, pair.1] {
        let row = data.channels.get(id).ok_or("Compared channel closed")?;
        if let Some(metric) = data.metrics.get(id) {
            for sample in metric.trace.samples() {
                writeln!(
                    metrics,
                    "{},{},{},{},{},{},{}",
                    id.0,
                    quoted(&row.kind),
                    quoted(metric.id.name),
                    quoted(metric.id.unit),
                    sample.epoch,
                    sample.tick,
                    sample.value
                )
                .unwrap();
            }
        }
    }
    let mut fields = String::from(
        "left_channel,left_epoch,left_tick,right_channel,right_epoch,right_tick,unit,x,y,z,left,right,absolute_difference\n",
    );
    if let Some(frames) = &state.capture.result {
        let [a, b] = frames;
        if (a.stamp.id, b.stamp.id) == pair {
            a.field.compare(&b.field)?;
            let [w, h, _] = a.field.size;
            for (i, (l, r)) in a.field.values.iter().zip(&b.field.values).enumerate() {
                let i = i as u32;
                writeln!(
                    fields,
                    "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    a.stamp.id.0,
                    a.stamp.epoch,
                    a.stamp.tick,
                    b.stamp.id.0,
                    b.stamp.epoch,
                    b.stamp.tick,
                    quoted(a.field.units),
                    i % w,
                    (i / w) % h,
                    i / (w * h),
                    l,
                    r,
                    (l - r).abs()
                )
                .unwrap();
            }
        }
    }
    let destination = Path::new(&state.folder);
    if destination.exists() {
        return Err("Export folder exists; choose a new folder below".into());
    }
    let temp = destination.with_extension(format!("csv-tmp-{}", std::process::id()));
    fs::create_dir(&temp).map_err(|e| e.to_string())?;
    let result=(|| {
        fs::write(temp.join("metrics.csv"),metrics)?;
        fs::write(temp.join("differences.csv"),fields)?;
        fs::write(temp.join("README.txt"),"Metric traces contain up to 200 actual samples per current epoch. Fields are completed captured states; XYZ error projects by maximum absolute value in the viewer. Empty differences means no captured field pair. Channel IDs are run-local.\n")?;
        fs::rename(&temp,destination)
    })().map_err(|e:std::io::Error|e.to_string());
    if result.is_err() {
        let _ = fs::remove_dir_all(&temp);
    }
    result
}
