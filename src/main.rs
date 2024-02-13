mod entity;
mod generator;
mod memory;

use bevy::window::WindowMode;
use bevy::{prelude::*, text, transform};
use generator::Completer;
use openai_api_rust::{Auth, OpenAI};
use rand::Rng;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::time::Duration;
use uuid::Uuid;

use crate::entity::character::Character;
use crate::generator::CompletionQuery;

#[derive(Default, Debug)]
enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}

#[repr(u8)]
#[derive(Default, Clone, Eq, PartialEq)]
enum Direction {
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

impl Direction {
    fn as_vec(&self) -> Vec2 {
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

#[derive(Default, Resource)]
struct Game {
    game_state: GameState,
    x_position: f32,
    y_position: f32,
    asker: Option<Sender<CompletionQuery>>,
    oracle: Option<Oracle>,
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

#[derive(Component)]
struct Terrain {}

fn requestor_system(keyboard_input: Res<Input<KeyCode>>, mut game: ResMut<Game>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        game.game_state = GameState::Paused;
    }
}

fn slide_terrain(game: Res<Game>, mut tile_position: Query<(&mut Terrain, &mut Transform)>) {
    for (mut _terrain, mut transform) in &mut tile_position {
        transform.translation.x += game.x_position;
        transform.translation.y += game.y_position;
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
enum Action {
    #[default]
    Running,
    Dying,
    Attacking,
    Idle,
}

#[derive(Component, Clone)]
struct AnimationSet {
    walking: AnimationIndices,
    running: AnimationIndices,
    idle: AnimationIndices,
    hit: AnimationIndices,
    attack: AnimationIndices,
    dying: AnimationIndices,
}

#[derive(Component, Clone)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
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
                    Action::Dying => anim_set.dying.clone(),
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

fn keyboard_to_direction<'a>(
    key_events: impl ExactSizeIterator<Item = &'a KeyCode>,
) -> Option<Direction> {
    let direction = key_events.fold(0, |acc, key| match key {
        KeyCode::A => acc | 0b0001,
        KeyCode::D => acc | 0b0010,
        KeyCode::W => acc | 0b0100,
        KeyCode::S => acc | 0b1000,
        _ => acc,
    });

    match direction {
        0b0001 => Some(Direction::W),
        0b0010 => Some(Direction::E),
        0b0100 => Some(Direction::N),
        0b1000 => Some(Direction::S),
        0b0101 => Some(Direction::NW),
        0b0110 => Some(Direction::NE),
        0b1001 => Some(Direction::SW),
        0b1010 => Some(Direction::SE),
        _ => None,
    }
}

fn control_player(
    keyboard_input: Res<Input<KeyCode>>,
    game_state: Res<Game>,
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
                let input: String = "a beautiful sorceress, dark hair, adept in ice magic".into();
                let query = generator::query::<String, Character>(&input);
                asker.send(query).expect("Channel send failed");
            };
        }
    } else {
        state.action = Action::Idle;
    }
}

fn control_ai(mut query: Query<(&mut AiController, &mut CharacterState)>, time: Res<Time>) {
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

fn speak_system(
    game_state: Res<Game>,
    keyboard_input: Res<Input<KeyCode>>,
    player_query: Query<(&HumanController, &Children)>,
    mut text_query: Query<&mut Text>,
) {
}

fn move_character(mut query: Query<(&mut Transform, &mut CharacterState)>, time: Res<Time>) {
    // let (mut player_transform, character_state) = query.single_mut();

    for (mut player_transform, character_state) in &mut query {
        if character_state.action != Action::Running {
            continue;
        }

        let character_direction = character_state.direction.as_vec();
        let (x, y) = (character_direction.x, character_direction.y);

        player_transform.translation.x =
            player_transform.translation.x + x * 170f32 * time.delta_seconds();
        player_transform.translation.y =
            player_transform.translation.y + y * 170f32 * time.delta_seconds();
    }
}

#[derive(Component)]
struct HumanController;

#[derive(Component)]
struct AiController {
    ticks_since_last_action: f32,
}

#[derive(Component)]
struct Speaker {
    last_request: Option<Uuid>,
}

#[derive(Component, PartialEq, Eq)]
struct CharacterState {
    action: Action,
    direction: Direction,
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
    let walking_animation = AnimationIndices { first: 4, last: 19 };
    let running_animation = AnimationIndices { first: 4, last: 11 };
    let hit_animation = AnimationIndices {
        first: 12,
        last: 15,
    };
    let dying_animation = AnimationIndices {
        first: 16,
        last: 23,
    };
    let attack_animation = AnimationIndices {
        first: 12,
        last: 15,
    };

    let animation_set = AnimationSet {
        walking: walking_animation,
        running: running_animation,
        idle: idle_animation,
        hit: hit_animation,
        dying: dying_animation,
        attack: attack_animation,
    };

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 15.0,
        color: Color::RED,
    };
    let text_alignment = TextAlignment::Center;

    commands.spawn(Camera2dBundle::default());

    let scale_factor = 3;

    for x in 0..10 {
        for y in 0..10 {
            let rand = rand::thread_rng().sample(rand::distributions::Uniform::new(0, 16));

            let scale = Vec3::new(scale_factor as f32, scale_factor as f32, 2f32);
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: terrain_atlas_handle.clone(),
                    sprite: TextureAtlasSprite::new(rand),
                    transform: Transform::from_translation(Vec3::new(
                        ((y - x) * 32 * scale_factor) as f32,
                        ((x + y) * 16 * scale_factor) as f32,
                        10.0,
                    ))
                    .with_scale(scale),
                    ..default()
                },
                Terrain {},
            ));
        }
    }

    commands
        .spawn((
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
        ))
        .with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text::from_section("", text_style.clone()).with_alignment(text_alignment),
                transform: Transform {
                    translation: Vec3::new(0.0, 50.0, 0.0),
                    ..default()
                },
                ..default()
            });
        });

    commands
        .spawn((
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
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2dBundle {
                    text: Text::from_section("", text_style).with_alignment(text_alignment),
                    transform: Transform {
                        translation: Vec3::new(0.0, 50.0, 0.0),
                        ..default()
                    },
                    ..default()
                },
                Speaker { last_request: None },
            ));
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
                requestor_system,
                read_oracle,
                slide_terrain,
                move_character,
                control_player,
                control_ai,
                speak_system,
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
