//! Model-owned symbol positions projected as voxel entities, not text tiles.
use bevy::prelude::*;
use models::workbench::symbolic::machine::Symbolic;
use crate::lab::stage::painters::symbolic::tape_window;
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub symbol: char,
    pub position: Vec3,
    pub head: bool,
}
pub fn cells(machine: &Symbolic) -> Vec<Cell> {
    let (tape, head) = match machine {
        Symbolic::Finite(m) => (&m.input, m.cursor),
        Symbolic::Pushdown(m) => (&m.input, m.cursor),
        Symbolic::Bounded(m) => (&m.tape, m.head),
        Symbolic::Turing(m) => (&m.tape, m.head),
        Symbolic::Koch { .. } => return Vec::new(),
    };
    let window = tape_window(tape.len(), head, 20);
    let start = window.start;
    let centre = (window.len() - 1) as f32 / 2.0;
    let mut cells: Vec<_> = window.enumerate().map(|(slot, index)| Cell {
        symbol: tape.get(index).copied().unwrap_or(' '),
        position: Vec3::new((slot as f32 - centre) * 3.0, 0.0, 0.0),
        head: index == head,
    }).collect();
    if let Symbolic::Pushdown(m) = machine {
        cells.extend(m.stack.iter().rev().take(12).enumerate().map(|(depth, symbol)| Cell {
            symbol: *symbol,
            position: Vec3::new((head.saturating_sub(start) as f32 - centre) * 3.0, -3.0 * (depth + 1) as f32, 0.0),
            head: false,
        }));
    }
    cells
}
pub fn palette(symbol: char) -> (Color, &'static str) {
    let (rgb, name) = match symbol {
        'a' => ([0.88, 0.64, 0.27], "amber"),
        'b' => ([0.26, 0.66, 0.91], "blue"),
        'c' => ([0.40, 0.80, 0.52], "green"),
        'A' => ([0.89, 0.40, 0.34], "red"),
        'X' => ([0.73, 0.42, 0.87], "purple"),
        '(' => ([0.30, 0.82, 0.77], "teal"),
        ')' => ([0.93, 0.48, 0.69], "pink"),
        ' ' => ([0.24, 0.24, 0.28], "blank / charcoal"),
        _ => ([0.65, 0.70, 0.78], "silver"),
    };
    (Color::srgb(rgb[0], rgb[1], rgb[2]), name)
}
