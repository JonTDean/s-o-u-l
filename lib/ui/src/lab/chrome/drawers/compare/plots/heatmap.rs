use super::super::capture::Frame;
use super::*;
use bevy_egui::egui::Color32;
pub(crate) fn difference(
    ui: &mut Ui,
    pair: &[Frame; 2],
    texture: &mut Option<egui::TextureHandle>,
) {
    note(
        ui,
        &format!(
            "Captured: L epoch {} tick {}; R epoch {} tick {}",
            pair[0].stamp.epoch, pair[0].stamp.tick, pair[1].stamp.epoch, pair[1].stamp.tick
        ),
        false,
    );
    if pair[0].stamp.tick != pair[1].stamp.tick {
        note(
            ui,
            "Different absolute ticks; this compares the displayed states.",
            true,
        );
    }
    let d = match pair[0].field.compare(&pair[1].field) {
        Ok(d) => d,
        Err(e) => {
            note(ui, &e, true);
            return;
        }
    };
    note(
        ui,
        &format!(
            "MAE {:.6} · RMS {:.6}\nmax {:.6} · changed {}\n{} · XY maximum absolute difference",
            d.mean_absolute, d.root_mean_square, d.maximum, d.changed, pair[0].field.units
        ),
        false,
    );
    let pixels: Vec<u8> = d
        .projection
        .iter()
        .flat_map(|v| {
            let t = if d.maximum > 0.0 { *v / d.maximum } else { 0.0 };
            [(20.0 + t * 235.0) as u8, (20.0 + t * 150.0) as u8, 25, 255]
        })
        .collect();
    let size = pair[0].field.size;
    let image =
        egui::ColorImage::from_rgba_unmultiplied([size[0] as usize, size[1] as usize], &pixels);
    match texture {
        Some(handle) => handle.set(image, egui::TextureOptions::NEAREST),
        None => {
            *texture = Some(ui.ctx().load_texture(
                "comparison difference",
                image,
                egui::TextureOptions::NEAREST,
            ))
        }
    }
    let texture = texture.as_ref().expect("heatmap texture");
    let width = ui.available_width().min(256.0);
    ui.image((
        texture.id(),
        egui::vec2(width, width * size[1] as f32 / size[0] as f32),
    ));
    ui.colored_label(
        Color32::from_rgb(255, 170, 25),
        format!("Heat scale: 0 to {:.6}", d.maximum),
    );
}
