use bevy::prelude::*;

pub mod game_timer;
// pub mod hp_bar;

use game_timer::GameTimerPlugin;
// use hp_bar::HpBarPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameTimerPlugin);
            // .add_plugins(HpBarPlugin);
    }
}
