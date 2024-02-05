mod entity;
mod generator;

use bevy::prelude::*;
use bevy::utils::Uuid;
use generator::Completer;
use openai_api_rust::{Auth, OpenAI};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use crate::entity::character::Character;
use crate::generator::CompletionQuery;

#[derive(Default)]
struct Player {
    x: f32,
    y: f32,
    speed: f32,
    direction: Vec2,
}

#[derive(Default, Debug)]
enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}

#[derive(Default, Resource)]
struct Game {
    player: Player,
    game_state: GameState,
    oracle: Option<Oracle>,
}

struct Oracle {
    completer: Sender<CompletionQuery>,
}

impl Oracle {
    fn ask<Character>(&self, input: CompletionQuery) {
        let _ = self.completer.send(input);
    }
}

fn default_completer() -> Oracle {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let completer = Completer { client: openai };

    //spawn new thread first for our background processing

    let (send_ask, receive_ask): (Sender<CompletionQuery>, Receiver<CompletionQuery>) = mpsc::channel();

    std::thread::spawn(move || {
        while (true) {
            let query = receive_ask.recv().unwrap();
            let character = completer.complete_as::<Character>(query).expect("Ooops?");
            println!("Got the following response {character:?}");
        }
    });

    Oracle {
        completer: send_ask,
    }

    // let input: String = "a beautiful sorceress, dark hair, adept in fire magic".into();
    // let thing = completer.materialize::<String, Character>(&input);

    // println!("Got the following response {thing:?}");
}

fn main() {
    App::new()
        .init_resource::<Game>()
        // .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (keyboard_input_system, animate_sprite, requestor_system),
        )
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .run();
}

fn keyboard_input_system(game_state: Res<Game>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::A) {
        // info!("'A' currently pressed");
    }

    if keyboard_input.just_pressed(KeyCode::A) {
        // info!("'A' just pressed");
    }

    if keyboard_input.just_released(KeyCode::A) {
        if let Some(oracle) = &game_state.oracle {
            let input: String = "a beautiful sorceress, dark hair, adept in ice magic".into();
            let thing = generator::query::<String, Character>(&input);
            oracle.ask::<Character>(thing);

            // println!("Got the following response {thing:?}");
        };
        info!("'A' just released");
    }
}

fn requestor_system(keyboard_input: Res<Input<KeyCode>>, mut game: ResMut<Game>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        game.game_state = GameState::Paused;
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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<Game>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("gabe-idle-run.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

    game.game_state = GameState::Playing;
    game.oracle = Some(default_completer());
}
