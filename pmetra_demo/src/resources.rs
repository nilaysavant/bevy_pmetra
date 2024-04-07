use bevy::prelude::*;

#[derive(Debug, Resource, Reflect, Default)]
pub struct CadGeneratedModelSpawner {
    pub selected_params: CadGeneratedModelParamsId,
}

#[derive(Debug, Reflect, Default)]
pub enum CadGeneratedModelParamsId {
    SimpleCubeAtCylinder,
    SimplLazyCubeAtCylinder,
    LazyTowerExtension,
    LazyRoundCabinSegment,
    #[default]
    LazySimpleGear,
    SimpleGear,
    RoundCabinSegment,
}
