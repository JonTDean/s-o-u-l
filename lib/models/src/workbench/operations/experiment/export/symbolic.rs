use super::raster::{INK, dot};
use crate::workbench::symbolic::machine::Symbolic;
use image::{Rgba, RgbaImage};
pub(super) fn draw(image: &mut RgbaImage, machine: &Symbolic) {
    let (tape, head) = match machine {
        Symbolic::Finite(m) => (m.input.clone(), m.cursor),
        Symbolic::Pushdown(m) => (m.input.clone(), m.cursor),
        Symbolic::Bounded(m) => (m.tape.clone(), m.head),
        Symbolic::Turing(m) => (m.tape.clone(), m.head),
        Symbolic::Koch { word, .. } => {
            koch(image, word);
            return;
        }
    };
    for (i, symbol) in tape
        .iter()
        .chain(std::iter::once(&' '))
        .take(256)
        .enumerate()
    {
        let x = (i % 16) * 16;
        let y = (i / 16) * 16;
        if i == head {
            for dx in 0..14 {
                image.put_pixel((x + dx) as u32, (y + 14) as u32, Rgba([255, 180, 40, 255]));
            }
        }
        let rows = glyph(*symbol);
        for (dy, bits) in rows.iter().enumerate() {
            for dx in 0..5 {
                if bits & (1 << (4 - dx)) != 0 {
                    image.put_pixel((x + dx + 3) as u32, (y + dy + 3) as u32, INK);
                }
            }
        }
    }
}
fn glyph(symbol: char) -> [u8; 7] {
    match symbol {
        'a' => [0, 0, 14, 1, 15, 17, 15],
        'b' => [16, 16, 30, 17, 17, 17, 30],
        'c' => [0, 0, 14, 17, 16, 17, 14],
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        ' ' => [0; 7],
        _ => [31, 17, 1, 2, 4, 0, 4],
    }
}
fn koch(image: &mut RgbaImage, word: &str) {
    let mut p = glam::Vec2::ZERO;
    let mut angle = 0.0f32;
    let mut points = vec![p];
    for c in word.chars() {
        match c {
            'F' => {
                p += glam::Vec2::from_angle(angle);
                points.push(p);
            }
            '+' => angle += std::f32::consts::PI / 3.0,
            '-' => angle -= std::f32::consts::PI / 3.0,
            _ => {}
        }
    }
    let min = points
        .iter()
        .fold(glam::Vec2::splat(f32::INFINITY), |a, p| a.min(*p));
    let max = points
        .iter()
        .fold(glam::Vec2::splat(f32::NEG_INFINITY), |a, p| a.max(*p));
    let scale = 58.0 / (max - min).max_element().max(1.0);
    for pair in points.windows(2) {
        let a = (pair[0] - min) * scale + glam::Vec2::splat(2.0);
        let b = (pair[1] - min) * scale + glam::Vec2::splat(2.0);
        let steps = ((b - a).length() * 4.0).ceil() as usize;
        for i in 0..=steps {
            let p = a.lerp(b, i as f32 / steps.max(1) as f32);
            dot(image, p.x, p.y, INK);
        }
    }
}
