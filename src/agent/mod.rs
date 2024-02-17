use bevy::ecs::component::Component;

pub mod npc;

#[repr(u8)]
#[derive(Default, Clone, Eq, PartialEq)]
pub enum Direction {
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

#[derive(Clone, Default, Eq, PartialEq)]
pub enum Action {
    #[default]
    Running,
    Attacking,
    Idle,
}

#[derive(Component, PartialEq, Eq)]
pub struct CharacterState {
    pub action: Action,
    pub direction: Direction,
}
