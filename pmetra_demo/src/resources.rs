use bevy::prelude::*;

#[derive(Debug, Resource, Reflect, Default)]
pub struct CadGeneratedModelSpawner {
    pub selected_params: CadGeneratedModelParamsId,
}

#[derive(Debug, Reflect, Default)]
pub enum CadGeneratedModelParamsId {
    #[default]
    SimplCubeAtCylinder,
    TowerExtension,
    RoundCabinSegment,
    MultiSimplCubeAtCylinderAndTowerExtension,
}
