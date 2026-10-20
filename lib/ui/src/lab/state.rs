//! Per-viewer lab UI state: mode, panels, drafts and stage views.
use super::stage::view::StageView;
use bevy::prelude::Resource;
use engine_core::automata::channel::ChannelId;
use std::collections::HashMap;

/// Workbench (all panels) or presenter (stage and HUD only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabMode {
    /// Rail, tabs, inspector, transport and stage.
    #[default]
    Workbench,
    /// Full-window stage with HUD readouts.
    Presenter,
}

/// Drawer opened from the rail; at most one is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drawer {
    /// Specimen library.
    Library,
    /// Session comparison and exports.
    Compare,
    /// Project save and load.
    Project,
}

/// UI-only state; nothing here affects simulation.
#[derive(Resource, Debug)]
pub struct LabState {
    /// Workbench or presenter.
    pub mode: LabMode,
    /// Open rail drawer, if any.
    pub drawer: Option<Drawer>,
    /// Settings popover visibility.
    pub settings_open: bool,
    /// Inspector visibility (N).
    pub inspector_open: bool,
    /// Inspector width in logical points.
    pub inspector_width: f32,
    /// Library filter text.
    pub filter: String,
    /// Seconds since the pointer last moved.
    pub idle_secs: f32,
    /// Unsent symbolic input drafts.
    pub inputs: HashMap<ChannelId, String>,
    /// Zoom and pan per CPU channel.
    pub views: HashMap<ChannelId, StageView>,
    /// Message and remaining seconds.
    pub toast: Option<(String, f32)>,
}

impl Default for LabState {
    fn default() -> Self {
        Self {
            mode: LabMode::Workbench,
            drawer: Some(Drawer::Library), // the library opens on first launch
            settings_open: false,
            inspector_open: true,
            inspector_width: Self::DEFAULT_WIDTH,
            filter: String::new(),
            idle_secs: 0.0,
            inputs: HashMap::new(),
            views: HashMap::new(),
            toast: None,
        }
    }
}

impl LabState {
    /// Default inspector width in logical points.
    pub const DEFAULT_WIDTH: f32 = 260.0;
    /// Seconds without pointer motion before the presenter transport fades.
    pub const IDLE_FADE_SECS: f32 = 2.0;

    /// Inspector width within 220–420 points.
    pub fn clamp_width(width: f32) -> f32 {
        width.clamp(220.0, 420.0)
    }

    /// Open `drawer`, or close it when it is already the open one.
    pub fn toggle_drawer(&mut self, drawer: Drawer) {
        self.drawer = if self.drawer == Some(drawer) {
            None
        } else {
            Some(drawer)
        };
    }

    /// Track pointer idleness for the presenter transport.
    pub fn note_pointer(&mut self, moved: bool, dt: f32) {
        self.idle_secs = if moved { 0.0 } else { self.idle_secs + dt };
    }

    /// 1 while recently active, fading to 0 over 0.3 s after the idle threshold.
    ///
    /// Alphas below one 8-bit step are invisible, so they snap to 0; this absorbs
    /// f32 drift in the accumulated idle time and lets callers skip drawing.
    pub fn transport_alpha(&self) -> f32 {
        let alpha = (1.0 - (self.idle_secs - Self::IDLE_FADE_SECS) / 0.3).clamp(0.0, 1.0);
        if alpha < 1.0 / 255.0 { 0.0 } else { alpha }
    }

    /// Whether Escape belongs to the lab (closing a drawer or leaving presenter).
    pub fn consumes_escape(&self) -> bool {
        self.drawer.is_some() || self.mode == LabMode::Presenter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_width_is_clamped() {
        assert_eq!(LabState::clamp_width(100.0), 220.0);
        assert_eq!(LabState::clamp_width(300.0), 300.0);
        assert_eq!(LabState::clamp_width(900.0), 420.0);
        assert_eq!(LabState::default().inspector_width, 260.0);
    }

    #[test]
    fn presenter_transport_fades_after_two_idle_seconds() {
        let mut lab = LabState::default();
        lab.note_pointer(false, 1.9);
        assert_eq!(lab.transport_alpha(), 1.0);
        lab.note_pointer(false, 0.4);
        assert_eq!(lab.transport_alpha(), 0.0);
        lab.note_pointer(true, 0.016);
        assert_eq!(lab.transport_alpha(), 1.0);
    }

    #[test]
    fn library_opens_on_first_launch_and_drawers_toggle() {
        let mut lab = LabState::default();
        assert_eq!(lab.drawer, Some(Drawer::Library));
        lab.toggle_drawer(Drawer::Compare);
        assert_eq!(
            lab.drawer,
            Some(Drawer::Compare),
            "switching drawers keeps one open"
        );
        lab.toggle_drawer(Drawer::Compare);
        assert_eq!(lab.drawer, None);
    }
}
