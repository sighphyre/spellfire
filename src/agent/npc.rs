use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    math::{Vec2, Vec3},
    prelude::default,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};

use crate::{generator::Conversation, AnimationSet, AnimationTimer};

use super::{Action, CharacterState, Direction};


#[derive(Component)]
pub struct AiController {
    pub ticks_since_last_action: f32,
    pub active_converstation: Option<Conversation>,
}

pub fn control_ai(mut query: Query<(&mut AiController, &mut CharacterState)>, time: Res<Time>) {
    for (mut controller, mut state) in &mut query {
        controller.ticks_since_last_action += time.delta_seconds();

        let current_ticks = controller.ticks_since_last_action;

        if state.action == Action::Idle && state.direction == Direction::N && current_ticks > 4.0 {
            state.action = Action::Running;
            state.direction = Direction::W;
            controller.ticks_since_last_action = 0.0;
            continue;
        } else if state.action == Action::Running
            && state.direction == Direction::W
            && current_ticks > 5.0
        {
            state.action = Action::Idle;
            state.direction = Direction::S;
            controller.ticks_since_last_action = 0.0;
            continue;
        }
        if state.action == Action::Idle && state.direction == Direction::S && current_ticks > 4.0 {
            state.action = Action::Running;
            state.direction = Direction::E;
            controller.ticks_since_last_action = 0.0;
            continue;
        } else if state.action == Action::Running
            && state.direction == Direction::E
            && current_ticks > 5.0
        {
            state.action = Action::Idle;
            state.direction = Direction::N;
            controller.ticks_since_last_action = 0.0;
            continue;
        }
    }
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

type AiAgentBundle = (
    SpriteSheetBundle,
    AnimationSet,
    AnimationTimer,
    CharacterState,
    AiController,
);

pub fn new_ai_agent_bundle(
    character_atlas_handle: Handle<TextureAtlas>,
    animation_set: AnimationSet,
) -> AiAgentBundle {
    (
        SpriteSheetBundle {
            texture_atlas: character_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_scale(Vec3::splat(2.0))
                .with_translation(Vec3::new(300.0, 10.0, 10.00)),
            ..default()
        },
        animation_set,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        CharacterState {
            action: Action::Idle,
            direction: Direction::N,
        },
        AiController {
            ticks_since_last_action: 0.0,
            active_converstation: None,
        },
    )
}