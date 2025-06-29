use bevy::ecs::{component::Component, resource::Resource};

#[derive(Resource)]
pub struct GlobalRng(pub rand::rngs::StdRng);

#[derive(Component)]
pub struct MainCamera;
