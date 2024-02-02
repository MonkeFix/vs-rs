use bevy::prelude::*;
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::collisions::colliders::Collider;
use crate::movement::SteeringHost;

#[cfg(debug_assertions)]
pub struct DebugPlugin;

#[cfg(debug_assertions)]
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SteeringHost>();
        app.register_type::<Collider>();
        app.add_plugins(WorldInspectorPlugin::new());
        app.add_systems(Update, bevy::window::close_on_esc);
    }
}
