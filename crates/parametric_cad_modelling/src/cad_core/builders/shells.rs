use anyhow::{Context, Ok, Result};
use bevy::prelude::*;
use truck_meshalgo::{
    filters::OptimizingFilter, rexport_polymesh::PolygonMesh, tessellation::MeshedShape,
};
use truck_modeling::{Shell, Surface};

use crate::{
    cad_core::{
        meshing::{BuildCadMeshedShell, BuildPolygon},
        tessellation::{CadMeshedShell, CustomMeshableShape},
    },
    constants::CUSTOM_TRUCK_TOLERANCE_1,
};

use super::{CadElement, CadElementTag, CadTaggedElements};
