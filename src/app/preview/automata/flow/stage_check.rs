//! Screenshot assertions: specimen pixels inside the stage, and no panel over the stage.
use bevy::{prelude::*, window::PrimaryWindow};
use engine_common::controls::camera::manager::stage::{StageRect, physical_viewport};
use std::path::Path;

/// Stage rectangle `(x, y, w, h)` in physical pixels.
pub(in crate::app::preview) type Stage = (u32, u32, u32, u32);

/// The lab's current stage in physical pixels, if one is published.
pub(in crate::app::preview) fn stage_pixels(world: &mut World) -> Option<Stage> {
    let rect = *world.get_resource::<StageRect>()?;
    let mut windows = world.query_filtered::<&Window, With<PrimaryWindow>>();
    let size = windows.single(world).ok()?.physical_size();
    let v = physical_viewport(&rect, size)?;
    let (min, size) = (v.physical_position, v.physical_size);
    Some((min.x, min.y, size.x, size.y))
}

fn count(rgba: &[u8], width: u32, stage: Stage, keep: impl Fn(&[u8]) -> bool) -> u64 {
    let (x0, y0, w, h) = stage;
    let mut count = 0;
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let i = ((y * width + x) * 4) as usize;
            if rgba.get(i..i + 3).is_some_and(&keep) {
                count += 1;
            }
        }
    }
    count
}

/// Pixels within `tolerance` per channel of `target`, inside `stage` of an RGBA8 image.
pub(in crate::app::preview) fn count_signal(
    rgba: &[u8],
    width: u32,
    stage: Stage,
    target: [u8; 3],
    tolerance: u8,
) -> u64 {
    count(rgba, width, stage, |px| {
        px.iter()
            .zip(target)
            .all(|(a, b)| a.abs_diff(b) <= tolerance)
    })
}

/// Warm, saturated pixels (shaded amber); dim ink text (r − b ≈ 20) does not count.
pub(in crate::app::preview) fn count_warm(rgba: &[u8], width: u32, stage: Stage) -> u64 {
    count(rgba, width, stage, |px| {
        px[0] > px[1] && px[1] > px[2] && px[0] > 80 && px[0] - px[2] > 48
    })
}

/// True when the 2 px band inside the stage edge has no panel-coloured (#121415) pixels.
pub(in crate::app::preview) fn panels_clear(rgba: &[u8], width: u32, stage: Stage) -> bool {
    let (x0, y0, w, h) = stage;
    let panel = [0x12, 0x14, 0x15];
    let band = |x: u32, y: u32| x < x0 + 2 || y < y0 + 2 || x + 2 >= x0 + w || y + 2 >= y0 + h;
    (y0..y0 + h).all(|y| {
        (x0..x0 + w).filter(|x| band(*x, y)).all(|x| {
            let i = ((y * width + x) * 4) as usize;
            rgba.get(i..i + 3).is_none_or(|px| px != panel)
        })
    })
}

/// Measure a captured screenshot and append its row to `<output>/image-checks.json`.
pub(in crate::app::preview) fn record(
    output: &Path,
    name: &str,
    image: &Image,
    stage: Stage,
) -> serde_json::Value {
    let rgba = image
        .clone()
        .try_into_dynamic()
        .expect("RGBA screenshot")
        .to_rgba8();
    let (width, height) = rgba.dimensions();
    let raw = rgba.as_raw();
    let row = serde_json::json!({
        "image": name,
        "width": width,
        "height": height,
        "stage": [stage.0, stage.1, stage.2, stage.3],
        "signal_pixels": count_signal(raw, width, stage, [0xff, 0xb5, 0x47], 24),
        "warm_pixels": count_warm(raw, width, stage),
        "specimen_pixels": count(raw, width, stage, |px| {
            let max = *px.iter().max().unwrap();
            let min = *px.iter().min().unwrap();
            max > 80 && max - min > 48
        }),
        "panels_clear": panels_clear(raw, width, stage),
    });
    let path = output.join("image-checks.json");
    let mut rows: Vec<serde_json::Value> = std::fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    rows.push(row.clone());
    std::fs::write(&path, serde_json::to_vec_pretty(&rows).unwrap()).expect("image checks");
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(w: u32, h: u32, fill: [u8; 3]) -> Vec<u8> {
        (0..w * h)
            .flat_map(|_| [fill[0], fill[1], fill[2], 255])
            .collect()
    }

    fn paint(rgba: &mut [u8], width: u32, at: (u32, u32), px: [u8; 3]) {
        let i = ((at.1 * width + at.0) * 4) as usize;
        rgba[i..i + 3].copy_from_slice(&px);
    }

    #[test]
    fn counts_only_matching_pixels_inside_the_stage() {
        let mut rgba = image(10, 10, [13, 15, 16]);
        for at in [(2, 2), (3, 2), (9, 9)] {
            paint(&mut rgba, 10, at, [255, 181, 71]);
        }
        assert_eq!(count_signal(&rgba, 10, (0, 0, 5, 5), [255, 181, 71], 24), 2);
        assert_eq!(
            count_signal(&rgba, 10, (0, 0, 10, 10), [255, 181, 71], 24),
            3
        );
    }

    #[test]
    fn warm_pixels_need_hue_and_saturation() {
        let mut rgba = image(4, 1, [13, 15, 16]);
        paint(&mut rgba, 4, (0, 0), [200, 120, 40]);
        paint(&mut rgba, 4, (1, 0), [40, 120, 200]);
        paint(&mut rgba, 4, (2, 0), [0x8a, 0x84, 0x76]); // ink_dim text is not a specimen
        paint(&mut rgba, 4, (3, 0), [110, 64, 22]); // shaded amber voxel
        assert_eq!(count_warm(&rgba, 4, (0, 0, 4, 1)), 2);
    }

    #[test]
    fn detects_panel_fill_inside_the_stage_edge() {
        let rgba = image(10, 10, [13, 15, 16]);
        assert!(panels_clear(&rgba, 10, (1, 1, 8, 8)));
        let panel = image(10, 10, [18, 20, 21]);
        assert!(!panels_clear(&panel, 10, (1, 1, 8, 8)));
    }
}
