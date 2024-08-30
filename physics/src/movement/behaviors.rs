use super::steering::{PhysicalParams, SteeringHost, SteeringTarget};
use bevy::prelude::*;

/// Seeks the specified target moving directly towards it.
pub struct SteerSeek;

impl SteerSeek {
    pub fn steer(
        &self,
        position: &Transform,
        host: &SteeringHost,
        params: &PhysicalParams,
        target: &impl SteeringTarget,
    ) -> Vec2 {
        let dv = target.position() - position.translation.xy();
        let dv = dv.normalize_or_zero();

        dv * params.max_velocity - host.velocity
    }
}
