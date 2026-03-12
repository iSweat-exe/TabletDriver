// use eframe::egui; // Unused import
use display_info::DisplayInfo;
use std::sync::Arc;
use std::time::Instant;

use crate::app::autoupdate::UpdateStatus;
use crate::drivers::TabletData;
use crate::engine::state::SharedState;
use crossbeam_channel::Receiver;

#[derive(PartialEq, Clone, Copy)]
pub enum AppTab {
    Output,
    Filters,
    PenSettings,
    Console,
    Settings,
    Support,
    Release,
}

pub struct TabletMapperApp {
    // Shared State
    pub shared: Arc<SharedState>,

    // UI Local State
    pub displays: Vec<DisplayInfo>,
    pub last_update: Instant,
    pub profile_name: String,
    pub active_tab: AppTab,

    // Event Receiver
    pub tablet_receiver: Receiver<TabletData>,
    pub update_receiver: Receiver<UpdateStatus>,
    pub update_status: UpdateStatus,

    // Filters UI State
    pub selected_filter: String,

    // Debugger UI State
    pub show_debugger: bool,
    pub displayed_hz: f32,
    pub last_hz_update: Instant,
    pub last_packet_count: u32,
}
