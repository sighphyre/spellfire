use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    math::Vec3,
    prelude::default,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};

use crate::{agent::AnimationSet, AnimationTimer, EntityFactory, Game};

#[derive(Component)]
pub enum BlobState {
    Floating,
}

pub type MatterBlobBundleBundle = (SpriteSheetBundle, AnimationSet, AnimationTimer, BlobState);

pub fn new_matter_blob_bundle(
    atlas_handle: Handle<TextureAtlas>,
    animation_set: AnimationSet,
) -> MatterBlobBundleBundle {
    (
        SpriteSheetBundle {
            texture_atlas: atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_scale(Vec3::splat(2.0))
                .with_translation(Vec3::new(400.0, 10.0, 10.00)),
            ..default()
        },
        animation_set,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        BlobState::Floating,
    )
}

pub fn animate_blob(
    time: Res<Time>,
    game: Res<Game>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &BlobState,
        &AnimationSet,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (entity, anim_set, action_state, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let (first, last) = (0, 2);

            sprite.index = if sprite.index >= last || sprite.index < first {
                first
            } else {
                sprite.index + 1
            };
        }
    }
}

enum SpellEffect {
    Lift,
    Burn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SpellState {
    Idle,
    Casting,
    Active,
    Finished,
    SpawnedEffect(Entity),
}

impl SpellEffect {
    fn try_apply(
        &mut self,
        commands: &mut Commands,
        entity_factory: &EntityFactory,
    ) -> Option<SpellState> {
        match self {
            SpellEffect::Lift => {
                let blob = entity_factory.make_rock();

                let entity_id = commands.spawn(blob).id();

                Some(SpellState::SpawnedEffect(entity_id))
            }
            SpellEffect::Burn => {
                let blob = entity_factory.make_fire();

                let entity_id = commands.spawn(blob).id();
                Some(SpellState::SpawnedEffect(entity_id))
            }
        }
    }
}

struct SpellTarget {}

struct SpellCommand {
    effect: SpellEffect,
    target: Option<SpellTarget>,
}

impl SpellCommand {
    fn take_target(&mut self, target: SpellTarget) {
        self.target = Some(target);
    }
}

#[derive(Component)]
pub struct Spell {
    command_index: usize,
    spell_commands: Vec<SpellCommand>,
}

impl Spell {
    fn take_target(&mut self, target: SpellTarget) {
        let Some(active_command) = self.spell_commands.get_mut(self.command_index) else {
            return;
        };
        active_command.take_target(target);
    }

    fn update(
        &mut self,
        commands: &mut Commands,
        entity_factory: &EntityFactory,
    ) -> Option<SpellState> {
        let len = self.spell_commands.len();
        if len <= self.command_index {
            return Some(SpellState::Finished);
        }

        let Some(active_command) = self.spell_commands.get_mut(self.command_index) else {
            return Some(SpellState::Idle);
        };

        let Some(new_state) = active_command.effect.try_apply(commands, entity_factory) else {
            return Some(SpellState::Idle);
        };

        self.command_index += 1;
        Some(new_state)
    }
}

pub fn create_spell() -> Spell {
    Spell {
        command_index: 0,
        spell_commands: vec![
            SpellCommand {
                effect: SpellEffect::Lift,
                target: Some(SpellTarget {}),
            },
            SpellCommand {
                effect: SpellEffect::Burn,
                target: Some(SpellTarget {}),
            },
        ],
    }
}

pub fn update_spell(
    game: Res<Game>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Spell)>,
) {
    let Some(entity_factory) = game.entity_factory.as_ref() else {
        return;
    };

    for (_entity, mut spell) in &mut query {
        spell.update(&mut commands, entity_factory);
    }
}
