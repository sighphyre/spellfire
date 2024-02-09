mod entity;
mod generator;

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

#[derive(Default, Resource)]
struct Game {
    game_state: GameState,
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
                keyboard_input_system,
                animate_sprite,
                requestor_system,
                read_oracle,
                slide_terrain,
            ),
        )
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .run();
}

fn keyboard_input_system(mut game_state: ResMut<Game>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::A) {
        game_state.x_position -= 1.0;
        // info!("'A' currently pressed");
    } else if keyboard_input.pressed(KeyCode::D) {
        game_state.x_position += 1.0;
        // info!("'D' currently pressed");
    } else if keyboard_input.pressed(KeyCode::W) {
        game_state.y_position += 1.0;
        // info!("'W' currently pressed");
    } else if keyboard_input.pressed(KeyCode::S) {
        game_state.y_position -= 1.0;
        // info!("'S' currently pressed");
    }

    if keyboard_input.just_pressed(KeyCode::A) {
        // info!("'A' just pressed");
    }

    if keyboard_input.just_released(KeyCode::A) {
        if let Some(asker) = &game_state.asker {
            println!("WHEEEE");
            let input: String = "a beautiful sorceress, dark hair, adept in ice magic".into();
            let query = generator::query::<String, Character>(&input);
            asker.send(query).expect("Channel send failed");
            println!("Released");
        };
        info!("'A' just released");
    }
}

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

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
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
    let character_handle = asset_server.load("gabe-idle-run.png");
    let terrain_handle = asset_server.load("map.png");

    let character_atlas =
        TextureAtlas::from_grid(character_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);

    let terrain_atlas =
        TextureAtlas::from_grid(terrain_handle, Vec2::new(64.0, 32.0), 16, 2, None, None);

    let terrain_atlas_handle = texture_assets.add(terrain_atlas);
    let character_atlas_handle = texture_assets.add(character_atlas);

    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };
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
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(6.0))
                .with_translation(Vec3::new(400.0, 10.0, 10.00)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
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
