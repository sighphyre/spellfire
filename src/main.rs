mod entity;
mod generator;
mod memory;

use benimator::Animation;
use bevy::prelude::*;
use generator::Completer;
use openai_api_rust::{Auth, OpenAI};
use rand::Rng;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::time::Duration;

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

#[derive(Default, Resource)]
struct Game {
    game_state: GameState,
    direction: Direction,
    x_position: f32,
    y_position: f32,
    asker: Option<Sender<CompletionQuery>>,
    oracle: Option<Oracle>,
}

struct Oracle {
    // completer: Sender<CompletionQuery>,
    completion_queue: Arc<RwLock<VecDeque<String>>>,
}

impl Oracle {
    fn poll(&self) {
        let mut lock = self.completion_queue.write().unwrap();
        // println!("Reading the queue");
        while let Some(item) = lock.pop_front() {
            println!("{item}");
        }
    }
}

fn default_completer() -> (Sender<CompletionQuery>, Oracle) {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let completer = Completer { client: openai };

    let (send_ask, receive_ask): (Sender<CompletionQuery>, Receiver<CompletionQuery>) =
        mpsc::channel();

    let queue: VecDeque<String> = VecDeque::default();
    let lock = Arc::new(std::sync::RwLock::new(queue));

    let write_lock = lock.clone();
    //spawn new thread first for our background processing, this is a terrible, terrible idea and needs fixing
    std::thread::spawn(move || loop {
        let query = receive_ask.recv().unwrap();
        println!("Oooo, got me a query");
        let character = completer.complete(query).expect("Ooops?");
        write_lock.write().unwrap().push_back(character);
        println!("Sent to the queue");
    });

    (
        send_ask,
        Oracle {
            // completer: send_ask,
            completion_queue: lock,
        },
    )
}

#[derive(Component)]
struct Terrain {}

fn main() {
    App::new()
        .init_resource::<Game>()
        // .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // keyboard_input_system,
                animate_sprite,
                requestor_system,
                read_oracle,
                slide_terrain,
                move_player,
                control_player,
            ),
        )
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .run();
}

// fn keyboard_input_system(mut game_state: ResMut<Game>, keyboard_input: Res<Input<KeyCode>>) {
//     if keyboard_input.pressed(KeyCode::A) {
//         game_state.x_position -= 1.0;
//         // info!("'A' currently pressed");
//     } else if keyboard_input.pressed(KeyCode::D) {
//         game_state.x_position += 1.0;
//         // info!("'D' currently pressed");
//     } else if keyboard_input.pressed(KeyCode::W) {
//         game_state.y_position += 1.0;
//         // info!("'W' currently pressed");
//     } else if keyboard_input.pressed(KeyCode::S) {
//         game_state.y_position -= 1.0;
//         // info!("'S' currently pressed");
//     }

//     if keyboard_input.just_pressed(KeyCode::A) {
//         // info!("'A' just pressed");
//     }

//     if keyboard_input.just_released(KeyCode::A) {
//         if let Some(asker) = &game_state.asker {
//             println!("WHEEEE");
//             let input: String = "a beautiful sorceress, dark hair, adept in ice magic".into();
//             let query = generator::query::<String, Character>(&input);
//             asker.send(query).expect("Channel send failed");
//             println!("Released");
//         };
//         info!("'A' just released");
//     }
// }

fn requestor_system(keyboard_input: Res<Input<KeyCode>>, mut game: ResMut<Game>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        game.game_state = GameState::Paused;
    }
}

fn slide_terrain(game: Res<Game>, mut tile_position: Query<(&mut Terrain, &mut Transform)>) {
    for (mut _terrain, mut transform) in &mut tile_position {
        // let x = time.x_position;
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

fn control_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&HumanController, &mut CharacterState)>,
    time: Res<Time>,
) {
    let (mut _controller, mut state) = query.single_mut();

    let mut direction = state.direction.clone();

    if keyboard_input.pressed(KeyCode::A) && keyboard_input.pressed(KeyCode::W) {
        direction = Direction::NW;
        state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::D) && keyboard_input.pressed(KeyCode::W) {
        direction = Direction::NE;
        state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::A) && keyboard_input.pressed(KeyCode::S) {
        direction = Direction::SW;
        state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::D) && keyboard_input.pressed(KeyCode::S) {
        direction = Direction::SE;
        state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::A) {
        direction = Direction::W;
        state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::D) {
        direction = Direction::E;
        state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::W) {
        direction = Direction::N;
        state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::S) {
        direction = Direction::S;
        state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::Space) {
        state.action = Action::Attacking;
    } else if keyboard_input.pressed(KeyCode::Q) {
        state.action = Action::Dying;
    } else {
        state.action = Action::Idle;
    }

    state.direction = direction;
}

fn move_player(mut query: Query<(&mut Transform, &CharacterState)>, time: Res<Time>) {
    let (mut player_transform, character_state) = query.single_mut();

    if character_state.action != Action::Running {
        return;
    }

    let x = match character_state.direction {
        Direction::W => -1.0,
        Direction::NW => -0.7,
        Direction::N => 0.0,
        Direction::NE => 0.7,
        Direction::E => 1.0,
        Direction::SE => 0.7,
        Direction::S => 0.0,
        Direction::SW => -0.7,
    };

    let y = match character_state.direction {
        Direction::W => 0.0,
        Direction::NW => 0.7,
        Direction::N => 1.0,
        Direction::NE => 0.7,
        Direction::E => 0.0,
        Direction::SE => -0.7,
        Direction::S => -1.0,
        Direction::SW => -0.7,
    };

    player_transform.translation.x =
        player_transform.translation.x + x * 100f32 * time.delta_seconds();
    player_transform.translation.y =
        player_transform.translation.y + y * 100f32 * time.delta_seconds();

    // player_transform.translation.x += 1.0;
    // let mut direction = 1.0;

    // if keyboard_input.pressed(KeyCode::A) {
    //     direction -= 1.0;
    // }

    // if keyboard_input.pressed(KeyCode::B) {
    //     direction += 1.0;
    // }

    // // // Calculate the new horizontal paddle position based on player input
    // let new_player_position =
    // player_transform.translation.x + direction * 500f32 * time.delta_seconds();

    // // // Update the paddle position,
    // // // making sure it doesn't cause the paddle to leave the arena
    // // let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    // // let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    // player_transform.translation.x = new_player_position;
}

#[derive(Component)]
struct HumanController;

#[derive(Component, PartialEq, Eq)]
struct CharacterState {
    action: Action,
    direction: Direction,
}

#[derive(Resource)]
struct OracleReaderConfig {
    timer: Timer,
}

fn read_oracle(game: ResMut<Game>, time: Res<Time>, mut config: ResMut<OracleReaderConfig>) {
    // tick the timer
    config.timer.tick(time.delta());

    if config.timer.finished() {
        game.oracle.as_ref().unwrap().poll();
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

    // let character_handle = asset_server.load("horse_paint.png");
    // let character_atlas =
    // TextureAtlas::from_grid(character_handle, Vec2::new(128.0, 128.0), 24, 8, None, None);

    // let character_atlas_handle = texture_assets.add(character_atlas);

    // let idle_animation = AnimationIndices { first: 0, last: 3 };
    // let walking_animation = AnimationIndices { first: 4, last: 7 };
    // let running_animation = AnimationIndices { first: 8, last: 11 };
    // let hit_animation = AnimationIndices {
    //     first: 12,
    //     last: 15,
    // };
    // let dying_animation = AnimationIndices {
    //     first: 16,
    //     last: 23,
    // };

    let animation_set = AnimationSet {
        walking: walking_animation,
        running: running_animation,
        idle: idle_animation,
        hit: hit_animation,
        dying: dying_animation,
        attack: attack_animation,
    };

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

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: character_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_scale(Vec3::splat(2.0))
                .with_translation(Vec3::new(400.0, 10.0, 10.00)),
            ..default()
        },
        animation_set,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        CharacterState {
            action: Action::Idle,
            direction: Direction::N,
        },
        HumanController {},
    ));

    commands.insert_resource(OracleReaderConfig {
        // create the repeating timer
        timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
    });

    game.game_state = GameState::Playing;
    let (asker, oracle) = default_completer();
    game.oracle = Some(oracle);
    game.asker = Some(asker);
}
