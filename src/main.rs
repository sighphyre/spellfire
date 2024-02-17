mod agent;
mod generator;
mod oracle;
mod terrain;

use agent::human::{new_human_agent_bundle, HumanController};
use agent::npc::{control_ai, new_ai_agent_bundle};
use agent::{
    animate_sprite, make_speech_bubble, move_agent, Action, CharacterState, Shout, SKELETON,
};
use bevy::prelude::*;
use bevy::window::WindowMode;
use oracle::{read_oracle, start_oracle, Oracle, OracleReaderConfig};
use std::sync::mpsc::Sender;
use std::time::Duration;
use terrain::TerrainGenerator;
use uuid::Uuid;

use crate::generator::CompletionQuery;

#[derive(Default, Debug, Eq, PartialEq)]
enum GameState {
    #[default]
    Loading,
    Playing,
    Typing,
}

#[derive(Default, Resource)]
struct Game {
    game_state: GameState,
    asker: Option<Sender<(Uuid, CompletionQuery)>>,
    oracle: Option<Oracle>,
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
    game: Res<Game>,
    mut query: Query<(&HumanController, &mut CharacterState)>,
) {
    if game.game_state != GameState::Playing {
        return;
    }

    let (mut _controller, mut character_state) = query.single_mut();

    if let Some(direction) = keyboard_to_direction(keyboard_input.get_pressed()) {
        character_state.direction = direction;
        character_state.action = Action::Running;
    } else if keyboard_input.pressed(KeyCode::Space) {
        character_state.action = Action::Attacking;
    } else if keyboard_input.pressed(KeyCode::Escape) {
        std::process::exit(0);
    } else {
        character_state.action = Action::Idle;
    }
}

fn toggle_text_input(mut game: ResMut<Game>, kbd: Res<Input<KeyCode>>) {
    if kbd.just_pressed(KeyCode::Return) {
        if game.game_state == GameState::Typing {
            game.game_state = GameState::Playing;
        } else {
            game.game_state = GameState::Typing;
        }
    }
}

fn text_input(
    mut evr_char: EventReader<ReceivedCharacter>,
    game: ResMut<Game>,
    kbd: Res<Input<KeyCode>>,
    mut string: Local<String>,
    mut megaphone: EventWriter<Shout>,
) {
    if game.game_state != GameState::Typing {
        return;
    }

    if kbd.just_pressed(KeyCode::Return) {
        megaphone.send(Shout {
            message: string.to_string(),
        });
        string.clear();
    }
    if kbd.just_pressed(KeyCode::Back) {
        string.pop();
    }
    for ev in evr_char.read() {
        if !ev.char.is_control() {
            string.push(ev.char);
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

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 15.0,
        color: Color::RED,
    };
    commands.spawn(Camera2dBundle::default());

    let terrain_gen = TerrainGenerator::new(10, 10, terrain_atlas_handle.clone());

    for tile in terrain_gen {
        commands.spawn(tile);
    }

    commands
        .spawn(new_human_agent_bundle(
            character_atlas_handle.clone(),
            SKELETON.clone(),
        ))
        .with_children(|parent| {
            parent.spawn(make_speech_bubble(text_style.clone()));
        });

    commands
        .spawn(new_ai_agent_bundle(
            character_atlas_handle.clone(),
            SKELETON.clone(),
        ))
        .with_children(|parent| {
            parent.spawn(make_speech_bubble(text_style.clone()));
        });

    commands.insert_resource(OracleReaderConfig {
        timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
    });

    game.game_state = GameState::Playing;
    let (asker, oracle) = start_oracle();
    game.oracle = Some(oracle);
    game.asker = Some(asker);
}

fn main() {
    App::new()
        .init_resource::<Game>()
        .add_event::<Shout>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_sprite,
                read_oracle,
                move_agent,
                control_player,
                control_ai,
                toggle_text_input,
                text_input,
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
