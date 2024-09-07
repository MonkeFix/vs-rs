use bevy::prelude::*;

pub mod game_timer;
pub mod health_bar;

use game_timer::GameTimerPlugin;
use health_bar::HealthBarPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameTimerPlugin)
            .add_plugins(HealthBarPlugin);
    }
}
