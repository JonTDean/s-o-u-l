use super::*;
use crate::theme::widgets::{margin_section,note,pill};
use bevy_egui::egui;
use engine_core::automata::channel::{ChannelAction,ChannelCommand};
pub(super) fn body(ui:&mut Ui,state:&mut Comparison,data:&LabData,fx:&mut Effects) {
    let rows=&data.channels.rows;
    if rows.len()<2 {note(ui,"Open two channels to compare.",false);return}
    if state.left.is_none() && state.right.is_none() {
        state.left=Some(rows[0].id);state.right=Some(rows[1].id);
    }
    margin_section(ui,"compare",true,|ui| {
        for (label,side) in [("left",&mut state.left),("right",&mut state.right)] {
            let shown=rows.iter().find(|r|Some(r.id)==*side).map_or("choose channel".into(),|r|format!("{} · {}",r.id.0,r.kind));
            egui::ComboBox::from_id_salt(("lab.compare",label)).selected_text(shown).show_ui(ui,|ui| {
                for row in rows {ui.selectable_value(side,Some(row.id),format!("{} · {}",row.id.0,row.kind));}
            });
        }
        let Some((l,r))=state.left.zip(state.right).filter(|(l,r)|l!=r) else {note(ui,"Choose two different channels",true);return};
        let idle=state.capture.pending.is_none();
        if pill(ui,"Pause both",false,idle).clicked() {
            for id in [l,r] {fx.commands.push(ChannelCommand {id,action:ChannelAction::Pause(true)});}
        }
        let paused=[l,r].iter().all(|id|rows.iter().any(|r|r.id==*id && r.status==Status::Paused));
        if pill(ui,"Capture completed fields",false,idle && paused).clicked() {state.capture.request=Some(false);}
        if pill(ui,"Step both and capture",true,idle && paused).clicked() {state.capture.request=Some(true);}
        ui.columns(2,|columns| {
            for (column,id) in columns.iter_mut().zip([l,r]) {
                if let Some(session)=data.bench.sessions.iter().find(|s|s.channel==id) {thumbnail(column,session);}
                super::plots::trace(column,id,data,fx);
            }
        });
        if let Some(pair)=&state.capture.result {
            if pair[0].stamp.id==l && pair[1].stamp.id==r {super::plots::difference(ui,pair);}
        }
        super::export::show(ui,state,data);
        note(ui,&state.status,false);
    });
}
