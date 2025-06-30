use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SimulationState {
    #[default]
    MeshGen,
    Tectonics,
    Erosion,
}

impl std::fmt::Display for SimulationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationState::MeshGen => write!(f, "MeshGen"),
            SimulationState::Tectonics => write!(f, "Tectonics"),
            SimulationState::Erosion => write!(f, "Erosion"),
        }
    }
}
