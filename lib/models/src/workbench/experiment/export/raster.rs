use crate::workbench::{project::Project, session::Model};
use image::{Rgba, RgbaImage};
pub(super) const INK: Rgba<u8> = Rgba([80, 210, 205, 255]);
pub fn render(project: &Project) -> RgbaImage {
    let panels = (project.sessions.len() + project.lenia.len()).max(1);
    let mut image = RgbaImage::from_pixel(256 * panels as u32, 256, Rgba([8, 15, 22, 255]));
    for (index, session) in project.sessions.iter().enumerate() {
        let mut panel = RgbaImage::from_pixel(256, 256, Rgba([8, 15, 22, 255]));
        match &session.model {
            Model::Grid(g) => {
                let bytes = if g.grid.size.y == 1 {
                    g.history.iter().flatten().copied().collect()
                } else {
                    g.bytes()
                };
                for (i, v) in bytes.iter().enumerate() {
                    if *v != 0 {
                        dot(&mut panel, (i % 64) as f32, (i / 64) as f32, INK);
                    }
                }
            }
            Model::Boids(f) => {
                for b in &f.boids {
                    dot(&mut panel, b.position.x, b.position.y, INK);
                }
            }
            Model::Ants(a) => {
                for (p, v) in &a.colony.pheromone {
                    dot(
                        &mut panel,
                        p.x as f32,
                        p.y as f32,
                        Rgba([20, (v * 30.0).clamp(20.0, 180.0) as u8, 50, 255]),
                    );
                }
                for ant in &a.colony.ants {
                    dot(
                        &mut panel,
                        ant.pos.x as f32,
                        ant.pos.y as f32,
                        if ant.carrying_food {
                            Rgba([255, 220, 40, 255])
                        } else {
                            INK
                        },
                    );
                }
                for p in a
                    .colony
                    .food_sources
                    .iter()
                    .chain(std::iter::once(&a.colony.home))
                {
                    dot(
                        &mut panel,
                        p.x as f32,
                        p.y as f32,
                        Rgba([255, 255, 255, 255]),
                    );
                }
            }
            Model::Symbolic(m) => super::symbolic::draw(&mut panel, m),
        }
        image::imageops::replace(&mut image, &panel, index as i64 * 256, 0);
    }
    for (index, saved) in project.lenia.iter().enumerate() {
        // Maximum density projection across the independent XY planes.
        for y in 0..64 {
            for x in 0..64 {
                let v = (0..8)
                    .map(|z| saved.densities[z * 4096 + y * 64 + x])
                    .max()
                    .unwrap_or(0);
                for dy in 0..4 {
                    for dx in 0..4 {
                        image.put_pixel(
                            (project.sessions.len() + index) as u32 * 256 + x as u32 * 4 + dx,
                            y as u32 * 4 + dy,
                            Rgba([v, v, v, 255]),
                        );
                    }
                }
            }
        }
    }
    image
}
pub(super) fn dot(image: &mut RgbaImage, x: f32, y: f32, color: Rgba<u8>) {
    for dy in 0..4 {
        for dx in 0..4 {
            let px = (x * 4.0) as i32 + dx;
            let py = (y * 4.0) as i32 + dy;
            if (0..256).contains(&px) && (0..256).contains(&py) {
                image.put_pixel(px as u32, py as u32, color);
            }
        }
    }
}
