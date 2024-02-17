use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        event::EventReader,
        system::{Query, Res},
    },
    hierarchy::Children,
    math::{Vec2, Vec3},
    prelude::default,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    text::Text,
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};
use uuid::Uuid;

use crate::{
    generator::{CompletionQuery, Conversation},
    oracle::CompletionCallback,
    AnimationTimer, Game,
};

use super::{Action, AnimationSet, CharacterState, Direction, Shout};

#[derive(Component)]
pub struct AiController {
    pub id: Uuid,
    pub ticks_since_last_action: f32,
    pub active_converstation: Option<Conversation>,
    ai_state: AiState,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum ConversationState {
    Opening,
    Respond(String),
    WaitingForCompleter,
    WaitingForPartner,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum AiState {
    Idle,
    Patrolling(Action, Direction),
    Talking(ConversationState, Conversation),
}

impl AiState {
    fn next_state(&self, time_since_change: f32, events: Option<Vec<EventType>>) -> Option<Self> {
        let (player_events, completer_events) = if let Some(events) = events {
            let (player_events, completer_events): (Vec<EventType>, Vec<EventType>) =
                events.into_iter().partition(|x| x.is_player());

            let player_events = if !player_events.is_empty() {
                Some(player_events)
            } else {
                None
            };

            let completer_events = if !completer_events.is_empty() {
                Some(completer_events)
            } else {
                None
            };
            (player_events, completer_events)
        } else {
            (None, None)
        };

        let player_messages: Option<Vec<String>> = player_events.map(|events| {
            events
                .iter()
                .filter_map(|event| match event {
                    EventType::PlayerShout(message) => Some(message.clone()),
                    _ => None,
                })
                .collect()
        });

        if let Some(messages) = player_messages {
            let mut convo = Conversation::new();
            for message in messages {
                convo.input_from_partner(message);
            }
            return Some(AiState::Talking(ConversationState::Opening, convo));
        }

        match self {
            AiState::Idle => {
                if time_since_change > 2.0 {
                    Some(AiState::Patrolling(Action::Running, Direction::W))
                } else {
                    None
                }
            }
            AiState::Patrolling(action, direction) => {
                if time_since_change > 4.0 {
                    let next_direction = match direction {
                        Direction::W => Direction::N,
                        Direction::N => Direction::E,
                        Direction::E => Direction::S,
                        Direction::S => Direction::W,
                        _ => Direction::N,
                    };
                    if *action == Action::Running {
                        Some(AiState::Patrolling(Action::Idle, next_direction))
                    } else {
                        Some(AiState::Patrolling(Action::Running, next_direction))
                    }
                } else {
                    None
                }
            }
            AiState::Talking(state, convo) => match &state {
                ConversationState::Opening => Some(AiState::Talking(
                    ConversationState::WaitingForCompleter,
                    convo.clone(),
                )),

                ConversationState::WaitingForCompleter => {
                    if time_since_change > 4.0 {
                        Some(AiState::Idle)
                    } else if let Some(completer_events) = completer_events {
                        let event = completer_events
                            .first()
                            .expect("Expected a completer event");
                        match event {
                            EventType::CompleterResponse(message) => {
                                let mut conv = convo.clone();

                                conv.input_from_partner(message.clone());

                                Some(AiState::Talking(
                                    ConversationState::Respond(message.clone()),
                                    conv,
                                ))
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                ConversationState::Respond(_) => Some(AiState::Talking(
                    ConversationState::WaitingForPartner,
                    convo.clone(),
                )),

                ConversationState::WaitingForPartner => {
                    if time_since_change > 4.0 {
                        Some(AiState::Idle)
                    } else {
                        None
                    }
                }
            },
        }
    }
}

#[derive(Clone, Debug)]
enum EventType {
    PlayerShout(String),
    CompleterResponse(String),
}

impl EventType {
    pub fn is_player(&self) -> bool {
        matches!(self, EventType::PlayerShout(_))
    }
}

pub fn tick_ai(
    mut query: Query<(&mut AiController, &mut CharacterState, &Children)>,
    time: Res<Time>,
    mut shouts: EventReader<Shout>,
    mut completion_handler: EventReader<CompletionCallback>,
    mut text_query: Query<&mut Text>,
    game_state: Res<Game>,
) {
    for (mut controller, mut state, children) in &mut query {
        let shout_events = shouts
            .read()
            .map(|event| EventType::PlayerShout(event.message.clone()));

        let completion_events = completion_handler
            .read()
            // .filter(|event| event.id == controller.id)
            .map(|event| EventType::CompleterResponse(event.message.clone()));

        let events: Vec<EventType> = shout_events.chain(completion_events).collect();

        let events = if !events.is_empty() {
            Some(events)
        } else {
            None
        };

        controller.ticks_since_last_action += time.delta_seconds();

        let current_ticks = controller.ticks_since_last_action;

        if let Some(new_state) = controller
            .ai_state
            .next_state(current_ticks, events.clone())
        {
            println!("AI state changed: {:#?}", new_state);
            controller.ai_state = new_state;
            controller.ticks_since_last_action = 0.0;
        }

        let (action, direction) = match &controller.ai_state {
            AiState::Idle => (Action::Idle, Direction::S),
            AiState::Patrolling(action, direction) => (*action, *direction),
            AiState::Talking(state, conversation) => {
                match state {
                    ConversationState::Opening => {
                        let next_message_prompt: CompletionQuery = conversation.clone().into();

                        let _ = &game_state
                            .asker
                            .send((controller.id, next_message_prompt))
                            .expect("Channel send failed");

                        for child in children.iter() {
                            let mut text = text_query.get_mut(*child).unwrap();
                            text.sections[0].value = "...".to_string();
                        }
                    }
                    ConversationState::Respond(message) => {
                        for child in children.iter() {
                            let mut text = text_query.get_mut(*child).unwrap();
                            text.sections[0].value = message.to_string();
                        }
                    }
                    _ => {}
                };
                (Action::Idle, Direction::S)
            }
        };

        state.action = action;
        state.direction = direction;
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
            id: Uuid::new_v4(),
            ticks_since_last_action: 0.0,
            active_converstation: None,
            ai_state: AiState::Patrolling(Action::Idle, Direction::N),
        },
    )
}
