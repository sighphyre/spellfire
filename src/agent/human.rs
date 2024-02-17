use bevy::{
    asset::Handle,
    ecs::component::Component,
    math::Vec3,
    prelude::default,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    time::{Timer, TimerMode},
    transform::components::Transform,
};

use crate::AnimationTimer;

use super::{Action, AnimationSet, CharacterState, Direction};

type HumanAgentBundle = (
    SpriteSheetBundle,
    AnimationSet,
    AnimationTimer,
    CharacterState,
    HumanController,
);

#[derive(Component)]
pub struct HumanController;

pub fn new_human_agent_bundle(
    character_atlas_handle: Handle<TextureAtlas>,
    animation_set: AnimationSet,
) -> HumanAgentBundle {
    (
        SpriteSheetBundle {
            texture_atlas: character_atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_scale(Vec3::splat(2.0))
                .with_translation(Vec3::new(400.0, 10.0, 10.00)),
            ..default()
        },
        animation_set.clone(),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        CharacterState {
            action: Action::Idle,
            direction: Direction::N,
        },
        HumanController {},
    )
}
