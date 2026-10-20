//! Stage authoring uses channel commands and the same fitted coordinates as painting.
use super::view::Fit;
use crate::lab::Effects;
use bevy_egui::egui::{Response, Ui};
use engine_core::automata::channel::{ChannelAction, ChannelCommand, ChannelId};
use models::workbench::{
    catalog::Kind,
    session::{Model, Session},
};
mod controls;
pub use controls::show;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Navigate,
    Paint,
    Erase,
}
#[derive(Debug, Default)]
pub struct Editor {
    pub tool: Tool,
    pub gas: u8,
    stroke: Option<(ChannelId, [i32; 2])>,
}
impl Editor {
    pub fn gesture(
        &mut self,
        ui: &Ui,
        response: &Response,
        fit: &Fit,
        session: &Session,
        fx: &mut Effects,
        enabled: bool,
    ) {
        let down = response.is_pointer_button_down_on();
        if !enabled || !down || !session.paused || self.tool == Tool::Navigate {
            self.stroke = None;
            return;
        }
        let Model::Grid(grid) = &session.model else {
            return;
        };
        let Some((x, y)) = response.interact_pointer_pos().and_then(|p| fit.to_cell(p)) else {
            return;
        };
        // Elementary editing targets the current row, not historic generations.
        let point = [x, if grid.grid.size.y == 1 { 0 } else { y }];
        let previous = self
            .stroke
            .filter(|(id, _)| *id == session.channel)
            .map(|(_, p)| p);
        if previous == Some(point) {
            return;
        }
        let value = if self.tool == Tool::Erase {
            0
        } else if session.kind == Kind::Hpp {
            self.gas.clamp(1, 15)
        } else {
            255
        };
        fx.commands.push(ChannelCommand {
            id: session.channel,
            action: ChannelAction::Paint {
                points: line(previous.unwrap_or(point), point),
                value,
                start: previous.is_none(),
            },
        });
        self.stroke = Some((session.channel, point));
        ui.ctx().request_repaint();
    }
}
/// Rasterize every crossed cell so fast pointer movement leaves no gaps.
pub fn line(a: [i32; 2], b: [i32; 2]) -> Vec<[i32; 2]> {
    let n = (b[0] - a[0]).abs().max((b[1] - a[1]).abs()).max(1);
    (0..=n)
        .map(|i| {
            [
                a[0] + ((b[0] - a[0]) as f32 * i as f32 / n as f32).round() as i32,
                a[1] + ((b[1] - a[1]) as f32 * i as f32 / n as f32).round() as i32,
            ]
        })
        .collect()
}
#[cfg(test)]
mod tests;
