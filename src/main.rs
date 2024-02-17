mod agent;
mod generator;
mod terrain;

use agent::human::{new_human_agent_bundle, HumanController};
use agent::npc::{control_ai, new_ai_agent_bundle};
use agent::{animate_sprite, make_speech_bubble, move_agent, Action, CharacterState};
use bevy::prelude::*;
use bevy::window::WindowMode;
use generator::{Completer, Conversation};
use openai_api_rust::{Auth, OpenAI};
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::time::Duration;
use terrain::TerrainGenerator;
use uuid::Uuid;

use crate::generator::CompletionQuery;

#[derive(Default, Debug)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Default, Resource)]
struct Game {
    game_state: GameState,
    asker: Option<Sender<CompletionQuery>>,
    oracle: Option<Oracle>,
    conversation: Conversation,
}

struct Oracle {
    completion_queue: Arc<RwLock<VecDeque<(Uuid, String)>>>,
}

impl Oracle {
    fn get_messages(&self) -> Option<Vec<(Uuid, String)>> {
        let mut lock = self.completion_queue.write().unwrap();
        if lock.is_empty() {
            None
        } else {
            let mut result = Vec::new();
            while let Some(item) = lock.pop_front() {
                result.push(item);
            }
            Some(result)
        }
    }
}

fn default_completer() -> (Sender<CompletionQuery>, Oracle) {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let completer = Completer { client: openai };

    let (send_ask, receive_ask): (Sender<CompletionQuery>, Receiver<CompletionQuery>) =
        mpsc::channel();

    let queue: VecDeque<(Uuid, String)> = VecDeque::default();
    let lock = Arc::new(std::sync::RwLock::new(queue));

    let write_lock = lock.clone();

    //spawn new thread first for our background processing, this is a terrible, terrible idea and needs fixing
    std::thread::spawn(move || loop {
        let query = receive_ask.recv().unwrap();
        println!("Oooo, got me a query");
        let character = completer.complete(query).expect("Ooops?");
        write_lock
            .write()
            .unwrap()
            .push_back((Uuid::new_v4(), character));
    });

    (
        send_ask,
        Oracle {
            completion_queue: lock,
        },
    )
}

#[derive(Component, Clone)]
struct AnimationSet {
    running: AnimationIndices,
    idle: AnimationIndices,
    hit: AnimationIndices,
}

#[derive(Component, Clone)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn keyboard_to_direction<'a>(
    key_events: impl ExactSizeIterator<Item = &'a KeyCode>,
) -> Option<agent::Direction> {
    let direction = key_events.fold(0, |acc, key| match key {
        KeyCode::A => acc | 0b0001,
        KeyCode::D => acc | 0b0010,
        KeyCode::W => acc | 0b0100,
        KeyCode::S => acc | 0b1000,
        _ => acc,
    });

    match direction {
        0b0001 => Some(agent::Direction::W),
        0b0010 => Some(agent::Direction::E),
        0b0100 => Some(agent::Direction::N),
        0b1000 => Some(agent::Direction::S),
        0b0101 => Some(agent::Direction::NW),
        0b0110 => Some(agent::Direction::NE),
        0b1001 => Some(agent::Direction::SW),
        0b1010 => Some(agent::Direction::SE),
        _ => None,
    }
}

fn control_player(
    keyboard_input: Res<Input<KeyCode>>,
    game_state: ResMut<Game>,
    mut query: Query<(&HumanController, &mut CharacterState, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    let (mut _controller, mut state, children) = query.single_mut();

    if let Some(direction) = keyboard_to_direction(keyboard_input.get_pressed()) {
        state.direction = direction;
        state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::Space) {
        state.action = Action::Attacking;
    } else if keyboard_input.pressed(KeyCode::Escape) {
        std::process::exit(0);
    } else if keyboard_input.just_pressed(KeyCode::Return) {
        for child in children.iter() {
            let mut text = text_query.get_mut(*child).unwrap();
            text.sections[0].value = "...".to_string();
            if let Some(asker) = &game_state.asker {
                let next_message_prompt: CompletionQuery = game_state.conversation.clone().into();

                asker
                    .send(next_message_prompt)
                    .expect("Channel send failed");
            };
        }
    } else {
        state.action = Action::Idle;
    }
}

#[derive(Resource)]
struct OracleReaderConfig {
    timer: Timer,
}

fn read_oracle(
    game: ResMut<Game>,
    time: Res<Time>,
    mut config: ResMut<OracleReaderConfig>,
    player_query: Query<(&HumanController, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    config.timer.tick(time.delta());

    if config.timer.finished() {
        if let Some(messages) = game.oracle.as_ref().unwrap().get_messages() {
            for message in messages.iter() {
                for (_player, children) in &player_query {
                    for child in children.iter() {
                        let mut text = text_query.get_mut(*child).unwrap();
                        text.sections[0].value = message.1.to_string();
                    }
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<Game>,
    mut texture_assets: ResMut<Assets<TextureAtlas>>,
) {
    let terrain_handle = asset_server.load("map.png");
    let terrain_atlas =
        TextureAtlas::from_grid(terrain_handle, Vec2::new(64.0, 32.0), 16, 2, None, None);
    let terrain_atlas_handle = texture_assets.add(terrain_atlas);

    let character_handle = asset_server.load("skeleton_0.png");

    let character_atlas =
        TextureAtlas::from_grid(character_handle, Vec2::new(128.0, 128.0), 24, 8, None, None);

    let character_atlas_handle = texture_assets.add(character_atlas);

    let idle_animation = AnimationIndices { first: 0, last: 3 };

    let running_animation = AnimationIndices { first: 4, last: 11 };
    let hit_animation = AnimationIndices {
        first: 12,
        last: 15,
    };

    let animation_set = AnimationSet {
        running: running_animation,
        idle: idle_animation,
        hit: hit_animation,
    };

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 15.0,
        color: Color::RED,
    };
    commands.spawn(Camera2dBundle::default());

    let mut terrain_gen = TerrainGenerator::new(10, 10, terrain_atlas_handle.clone());

    while let Some(tile) = terrain_gen.next() {
        commands.spawn(tile);
    }

    commands
        .spawn(new_human_agent_bundle(
            character_atlas_handle.clone(),
            animation_set.clone(),
        ))
        .with_children(|parent| {
            parent.spawn(make_speech_bubble(text_style.clone()));
        });

    commands
        .spawn(new_ai_agent_bundle(
            character_atlas_handle.clone(),
            animation_set.clone(),
        ))
        .with_children(|parent| {
            parent.spawn(make_speech_bubble(text_style.clone()));
        });

    commands.insert_resource(OracleReaderConfig {
        timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
    });

    game.game_state = GameState::Playing;
    let (asker, oracle) = default_completer();
    game.oracle = Some(oracle);
    game.asker = Some(asker);
}

fn main() {
    App::new()
        .init_resource::<Game>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_sprite,
                read_oracle,
                move_agent,
                control_player,
                control_ai,
            ),
        )
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: true,
                        mode: WindowMode::BorderlessFullscreen,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .run();
}
