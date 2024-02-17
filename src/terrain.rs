use bevy::{
    asset::Handle,
    ecs::component::Component,
    math::Vec3,
    prelude::default,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    transform::components::Transform,
};
use rand::Rng;

#[derive(Component)]
pub struct Terrain {}

type TerrainBundle = (SpriteSheetBundle, Terrain);

pub struct TerrainGenerator {
    x: i32,
    y: i32,
    internal_x: i32,
    internal_y: i32,
    atlas_handle: Handle<TextureAtlas>,
}

impl TerrainGenerator {
    pub fn new(x: i32, y: i32, atlas: Handle<TextureAtlas>) -> TerrainGenerator {
        TerrainGenerator {
            x,
            y,
            internal_x: 0,
            internal_y: 0,
            atlas_handle: atlas,
        }
    }
}

impl Iterator for TerrainGenerator {
    type Item = TerrainBundle;

    fn next(&mut self) -> Option<Self::Item> {
        let (x, y) = (self.internal_x, self.internal_y);

        if x >= self.x {
            self.internal_x = 0;
            self.internal_y += 1;
        } else {
            self.internal_x += 1;
        }

        if y >= self.y {
            return None;
        }

        let scale_factor = 3;

        let rand = rand::thread_rng().sample(rand::distributions::Uniform::new(0, 16));
        let scale = Vec3::new(scale_factor as f32, scale_factor as f32, 2f32);

        Some((
            SpriteSheetBundle {
                texture_atlas: self.atlas_handle.clone(),
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
        ))
    }
}
