use bevy::{
    ecs::{
        component::Component,
        event::Event,
        system::{Query, Res},
    },
    math::Vec3,
    prelude::default,
    sprite::TextureAtlasSprite,
    text::{Text, Text2dBundle, TextAlignment, TextStyle},
    time::Time,
    transform::components::Transform,
};

use crate::{AnimationSet, AnimationTimer};

pub mod human;
pub mod npc;

#[repr(u8)]
#[derive(Default, Clone, Eq, PartialEq)]
pub enum Direction {
    #[default]
    W,
    NW,
    N,
    NE,
    E,
    SE,
    S,
    SW,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub enum Action {
    #[default]
    Running,
    Attacking,
    Idle,
}

#[derive(Component, PartialEq, Eq)]
pub struct CharacterState {
    pub action: Action,
    pub direction: Direction,
}

pub fn move_agent(mut query: Query<(&mut Transform, &mut CharacterState)>, time: Res<Time>) {
    for (mut player_transform, character_state) in &mut query {
        if character_state.action != Action::Running {
            continue;
        }

        let character_direction = character_state.direction.as_vec();
        let (x, y) = (character_direction.x, character_direction.y);

        player_transform.translation.x += x * 170f32 * time.delta_seconds();
        player_transform.translation.y += y * 170f32 * time.delta_seconds();
    }
}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationSet,
        &CharacterState,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (anim_set, action_state, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let direction = action_state.direction.clone();
            let action = action_state.action.clone();

            let (first, last) = {
                let indices = match action {
                    Action::Running => anim_set.running.clone(),
                    Action::Idle => anim_set.idle.clone(),
                    Action::Attacking => anim_set.hit.clone(),
                };

                let offset = (direction as usize) * 24;
                let first = indices.first + offset;
                let last = indices.last + offset;
                (first, last)
            };

            sprite.index = if sprite.index >= last || sprite.index < first {
                first
            } else {
                sprite.index + 1
            };
        }
    }
}

pub fn make_speech_bubble(text_style: TextStyle) -> Text2dBundle {
    Text2dBundle {
        text: Text::from_section("", text_style).with_alignment(TextAlignment::Center),
        transform: Transform {
            translation: Vec3::new(0.0, 50.0, 0.0),
            ..default()
        },
        ..default()
    }
}

#[derive(Event)]
pub struct Shout {
    pub message: String,
}
