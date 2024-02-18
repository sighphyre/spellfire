use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        event::EventReader,
        system::{Query, Res},
    },
    hierarchy::Children,
    math::Vec3,
    prelude::default,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    text::Text,
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};
use uuid::Uuid;

use crate::{generator::Conversation, oracle::CompletionCallback, AnimationTimer, Game};

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
    WaitingForCompleter,
    WaitingForPartner,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum AiState {
    Idle,
    Patrolling(Action, Direction),
    Talking(ConversationState),
}

impl AiState {
    fn next_state(
        &self,
        time_since_change: f32,
        interrupts: Option<Vec<EventType>>,
        callbacks: Option<Vec<Callback>>,
    ) -> Option<Self> {
        let player_messages: Option<Vec<String>> = interrupts.map(|events| {
            events
                .iter()
                .map(|event| match event {
                    EventType::PlayerShout(message) => message.clone(),
                })
                .collect()
        });

        if let Some(messages) = player_messages {
            let mut convo = Conversation::new();
            for message in messages {
                convo.input_from_partner(message);
            }
            return Some(AiState::Talking(ConversationState::WaitingForCompleter));
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
            AiState::Talking(state) => match &state {
                ConversationState::WaitingForCompleter => {
                    if time_since_change > 4.0 {
                        Some(AiState::Idle)
                    } else if let Some(completer_events) = callbacks {
                        let event = completer_events
                            .first()
                            .expect("Expected a completer event");
                        match event {
                            Callback::CompleterResponse(_message) => {
                                Some(AiState::Talking(ConversationState::WaitingForPartner))
                            }
                        }
                    } else {
                        None
                    }
                }

                ConversationState::WaitingForPartner => {
                    if time_since_change > 2.0 {
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
}

#[derive(Clone, Debug)]
enum Callback {
    CompleterResponse(String),
}

fn to_option<T>(vec: Vec<T>) -> Option<Vec<T>> {
    if vec.is_empty() {
        None
    } else {
        Some(vec)
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
            .map(|event| EventType::PlayerShout(event.message.clone()))
            .collect::<Vec<EventType>>();

        let completion_events = completion_handler
            .read()
            .filter(|event| event.id == controller.id)
            .map(|event| {
                println!("Completions: {:#?} and id {:#?}", event, controller.id);
                Callback::CompleterResponse(event.message.clone())
            })
            .collect::<Vec<Callback>>();

        controller.ticks_since_last_action += time.delta_seconds();

        let current_ticks = controller.ticks_since_last_action;

        if let Some(new_state) = controller.ai_state.next_state(
            current_ticks,
            to_option(shout_events.clone()),
            to_option(completion_events.clone()),
        ) {
            println!("AI state changed: {:#?}", new_state);
            controller.ai_state = new_state;
            controller.ticks_since_last_action = 0.0;

            let (action, direction) = match &controller.ai_state {
                AiState::Idle => (Action::Idle, Direction::S),
                AiState::Patrolling(action, direction) => (*action, *direction),
                AiState::Talking(state) => {
                    let mut conversation =
                        if let Some(conversation) = &controller.active_converstation {
                            conversation.clone()
                        } else {
                            Conversation::new()
                        };

                    let character_float_text = match state {
                        ConversationState::WaitingForCompleter => {
                            for event in shout_events {
                                match event {
                                    EventType::PlayerShout(message) => {
                                        conversation.input_from_partner(message);
                                    }
                                }
                            }

                            let next_message_prompt = conversation.clone().into();

                            let _ = &game_state
                                .asker
                                .send((controller.id, next_message_prompt))
                                .expect("Channel send failed");

                            "...".to_string()
                        }
                        ConversationState::WaitingForPartner => {
                            let mut last_message = "".to_string();
                            for event in completion_events {
                                match event {
                                    Callback::CompleterResponse(message) => {
                                        last_message = message.clone();
                                        conversation.input_from_self(message);
                                    }
                                }
                            }
                            last_message.to_string()
                        }
                    };

                    println!("AI: {:#?}", conversation.messages);

                    controller.active_converstation = Some(conversation);

                    for child in children.iter() {
                        let mut text = text_query.get_mut(*child).unwrap();
                        text.sections[0].value = character_float_text.clone();
                    }

                    (Action::Idle, Direction::S)
                }
            };

            state.action = action;
            state.direction = direction;
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
