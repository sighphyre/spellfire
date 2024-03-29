mod agent;
mod generator;
mod oracle;
mod spell;
mod terrain;
mod camera;

use agent::human::{new_human_agent_bundle, HumanAgentBundle, HumanController};
use agent::npc::{new_ai_agent_bundle, tick_ai, AiAgentBundle};
use agent::{
    animate_sprite, make_speech_bubble, move_agent, Action, CharacterState, Shout, SKELETON,
};

use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_ecs_tilemap::TilemapPlugin;
use camera::move_camera;
use oracle::{read_oracle, start_oracle, CompletionCallback, Oracle, OracleReaderConfig};
use spell::{
    animate_blob, create_spell, new_matter_blob_bundle, update_spell, MatterBlobBundleBundle,
};
use std::sync::mpsc::Sender;
use std::time::Duration;
use terrain::{TiledMap, TiledMapBundle, TiledMapPlugin};
use uuid::Uuid;

use crate::generator::CompletionQuery;

#[derive(Default, Debug, Eq, PartialEq)]
enum GameState {
    #[default]
    Loading,
    Playing,
    Typing,
}

#[derive(Resource)]
struct Game {
    game_state: GameState,
    asker: Sender<(Uuid, CompletionQuery)>,
    oracle: Oracle,
    entity_factory: Option<EntityFactory>,
}

impl Default for Game {
    fn default() -> Self {
        let (asker, oracle) = start_oracle();

        Game {
            game_state: GameState::Loading,
            asker,
            oracle,
            entity_factory: None,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Default)]
struct InputText;

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
    mut commands: Commands,
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
    } else if keyboard_input.just_pressed(KeyCode::L) {
        let ai_bundle = game
            .entity_factory
            .as_ref()
            .map(|factory| factory.make_ai());

        let Some((bundle, text)) = ai_bundle else {
            return;
        };

        commands.spawn(bundle).with_children(|parent| {
            parent.spawn(text);
        });
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

fn handle_mouse(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    buttons: Res<Input<MouseButton>>,
    q_sprites: Query<(Entity, &Transform, &CharacterState)>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera_query.single();

        let Some(cursor_position) = windows.single().cursor_position() else {
            return;
        };

        let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
            return;
        };

        for (_entity, transform, _sprite) in q_sprites.iter() {
            let min = transform.translation.truncate() - Vec2::new(128.0, 64.0);
            let max = transform.translation.truncate() + Vec2::new(128.0, 64.0);

            if point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y {
                println!("Clicked an entity");
            }
        }
    }
}

fn text_input(
    mut evr_char: EventReader<ReceivedCharacter>,
    game: ResMut<Game>,
    kbd: Res<Input<KeyCode>>,
    mut string: Local<String>,
    mut query: Query<(&InputText, &mut Text)>,
    mut event_writer: EventWriter<Shout>,
) {
    if game.game_state != GameState::Typing {
        return;
    }

    if kbd.just_pressed(KeyCode::Return) {
        let message = string.to_string();
        if message.trim().is_empty() {
            return;
        }
        event_writer.send(Shout { message });
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

    for (_input, mut text) in &mut query {
        text.sections[0].value = string.to_string();
    }
}

struct EntityFactory {
    constructed_assets: ConstructedAssets,
}

struct NamedAssets {
    pub character: Handle<Image>,
    pub fire: Handle<Image>,
    pub rock: Handle<Image>,
    pub font: Handle<Font>,
}

struct ConstructedAssets {
    pub character_atlas: Handle<TextureAtlas>,
    pub fire_atlas: Handle<TextureAtlas>,
    pub rock_atlas: Handle<TextureAtlas>,
    pub text_style: TextStyle,
}

impl EntityFactory {
    fn new(named_assets: NamedAssets, mut texture_assets: ResMut<Assets<TextureAtlas>>) -> Self {
        let character_atlas = TextureAtlas::from_grid(
            named_assets.character.clone(),
            Vec2::new(128.0, 128.0),
            24,
            8,
            None,
            None,
        );

        let character_atlas_handle = texture_assets.add(character_atlas);

        let fire_atlas = TextureAtlas::from_grid(
            named_assets.fire.clone(),
            Vec2::new(32.0, 32.0),
            3,
            1,
            None,
            None,
        );

        let fire_atlas_handle = texture_assets.add(fire_atlas);

        let rock_atlas = TextureAtlas::from_grid(
            named_assets.rock.clone(),
            Vec2::new(32.0, 32.0),
            5,
            1,
            None,
            None,
        );

        let rock_atlas_handle = texture_assets.add(rock_atlas);

        let constructed_assets = ConstructedAssets {
            character_atlas: character_atlas_handle,
            rock_atlas: rock_atlas_handle.clone(),
            fire_atlas: fire_atlas_handle,
            text_style: TextStyle {
                font: named_assets.font.clone(),
                font_size: 15.0,
                color: Color::RED,
            },
        };

        EntityFactory { constructed_assets }
    }

    fn make_human(&self) -> (HumanAgentBundle, Text2dBundle) {
        (
            new_human_agent_bundle(
                self.constructed_assets.character_atlas.clone(),
                SKELETON.clone(),
            ),
            make_speech_bubble(self.constructed_assets.text_style.clone()),
        )
    }

    fn make_ai(&self) -> (AiAgentBundle, Text2dBundle) {
        (
            new_ai_agent_bundle(
                self.constructed_assets.character_atlas.clone(),
                SKELETON.clone(),
            ),
            make_speech_bubble(self.constructed_assets.text_style.clone()),
        )
    }

    fn make_rock(&self) -> MatterBlobBundleBundle {
        new_matter_blob_bundle(self.constructed_assets.rock_atlas.clone(), SKELETON.clone())
    }

    fn make_fire(&self) -> MatterBlobBundleBundle {
        new_matter_blob_bundle(self.constructed_assets.fire_atlas.clone(), SKELETON.clone())
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<Game>,
    texture_assets: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(camera::spawn_camera());

    let map_handle: Handle<TiledMap> = asset_server.load("test.tmx");

    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });

    let character_handle = asset_server.load("skeleton_0.png");
    let fire_handle = asset_server.load("fireball.png");
    let rock_handle = asset_server.load("rock.png");
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let assets = NamedAssets {
        character: character_handle.clone(),
        fire: fire_handle.clone(),
        rock: rock_handle.clone(),
        font: font.clone(),
    };

    let entity_factory = EntityFactory::new(assets, texture_assets);

    let (human_bundle, text_bubble) = entity_factory.make_human();

    commands.spawn(human_bundle).with_children(|parent| {
        parent.spawn(text_bubble);
    });

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font: font.clone(),
                font_size: 30.0,
                ..default()
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        InputText,
    ));

    commands.spawn(create_spell());

    commands.insert_resource(OracleReaderConfig {
        timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
    });

    game.entity_factory = Some(entity_factory);

    game.game_state = GameState::Playing;
}

fn main() {
    App::new()
        .init_resource::<Game>()
        .add_event::<Shout>()
        .add_event::<CompletionCallback>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_sprite,
                update_spell,
                read_oracle,
                move_agent,
                tick_ai,
                (text_input, control_player, toggle_text_input),
                handle_mouse,
                move_camera,
                animate_blob,
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
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .run();
}
