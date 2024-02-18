use bevy::{
    ecs::{
        component::Component,
        event::Event,
        system::{Query, Res},
    },
    math::{Vec2, Vec3},
    prelude::default,
    sprite::TextureAtlasSprite,
    text::{Text, Text2dBundle, TextAlignment, TextStyle},
    time::Time,
    transform::components::Transform,
};

use crate::AnimationTimer;

pub mod human;
pub mod npc;

pub const SKELETON: AnimationSet = AnimationSet {
    running: AnimationIndices { first: 4, last: 11 },
    idle: AnimationIndices { first: 0, last: 3 },
    hit: AnimationIndices {
        first: 12,
        last: 15,
    },
};

#[repr(u8)]
#[derive(Default, Clone, Copy, Eq, PartialEq, Debug)]
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

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug)]
pub enum Action {
    #[default]
    Running,
    Attacking,
    Idle,
}

#[derive(Component, Clone)]
pub struct AnimationSet {
    running: AnimationIndices,
    idle: AnimationIndices,
    hit: AnimationIndices,
}

#[derive(Component, Clone)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, PartialEq, Eq)]
pub struct CharacterState {
    pub action: Action,
    pub direction: Direction,
}

impl Direction {
    pub fn as_vec(&self) -> Vec2 {
        match self {
            Direction::W => Vec2::new(-1.0, 0.0),
            Direction::NW => Vec2::new(-0.7, 0.7),
            Direction::N => Vec2::new(0.0, 1.0),
            Direction::NE => Vec2::new(0.7, 0.7),
            Direction::E => Vec2::new(1.0, 0.0),
            Direction::SE => Vec2::new(0.7, -0.7),
            Direction::S => Vec2::new(0.0, -1.0),
            Direction::SW => Vec2::new(-0.7, -0.7),
        }
    }
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
            let direction = action_state.direction;
            let action = action_state.action;

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
