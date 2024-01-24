use bevy::prelude::*;
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::steering::SteeringHost;

#[cfg(debug_assertions)]
pub struct DebugPlugin;

#[cfg(debug_assertions)]
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SteeringHost>();
        app.add_plugins(WorldInspectorPlugin::new());
        app.add_systems(Update, bevy::window::close_on_esc);
    }
}
