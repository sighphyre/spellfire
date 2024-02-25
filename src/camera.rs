use bevy::{
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{component::Component, query::Without, system::Query},
    prelude::default,
    transform::components::Transform,
};

use crate::agent::human::HumanController;

#[derive(Component)]
pub struct CameraMarker;

pub type CameraBundle = (Camera2dBundle, CameraMarker);

pub fn spawn_camera() -> CameraBundle {
    (
        Camera2dBundle {
            transform: Transform::from_xyz(100.0, 200.0, 0.0),
            ..default()
        },
        CameraMarker,
    )
}

pub fn move_camera(
    player_query: Query<(&HumanController, &Transform), Without<CameraMarker>>,
    mut camera_query: Query<(&CameraMarker, &mut Transform), Without<HumanController>>,
) {
    for (_player, player_transform) in player_query.iter() {
        for (_camera, mut camera_transform) in camera_query.iter_mut() {
            camera_transform.translation = player_transform.translation;
        }
    }
}
