use bevy::utils::HashMap;
use rustc_hash::FxHashMap;
#[cfg(not(target_arch = "wasm32"))]
use truck_meshalgo::tessellation::triangulation::rayon::iter::{
    IntoParallelIterator, ParallelIterator,
};
use truck_meshalgo::tessellation::{
    triangulation::shell_create_polygon, Parallelizable, PolylineableCurve, PreMeshableSurface,
};

use truck_modeling::Point3;
use truck_topology::{Edge, Face, FaceID, Shell, Wire};

use crate::pmetra_core::tessellation::{CadMeshedShell, PolylineCurve};

/// Tessellates faces
#[cfg(not(target_arch = "wasm32"))]
pub fn shell_tessellation<'a, C, S, F>(
    shell: &Shell<Point3, C, S>,
    tol: f64,
    sp: F,
) -> CadMeshedShell<S>
where
    C: PolylineableCurve + 'a,
    S: PreMeshableSurface + 'a,
    F: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> + Parallelizable,
{
    let vmap: FxHashMap<_, _> = shell
        .vertex_par_iter()
        .map(|v| (v.id(), v.mapped(Point3::clone)))
        .collect();
    let eset: FxHashMap<_, _> = shell.edge_par_iter().map(move |e| (e.id(), e)).collect();
    let edge_map: FxHashMap<_, _> = eset
        .into_par_iter()
        .map(move |(id, edge)| {
            let v0 = vmap.get(&edge.absolute_front().id()).unwrap();
            let v1 = vmap.get(&edge.absolute_back().id()).unwrap();
            let curve = edge.curve();
            let poly = PolylineCurve::from_curve(&curve, curve.range_tuple(), tol);
            (id, Edge::debug_new(v0, v1, poly))
        })
        .collect();
    let create_edge = |edge: &Edge<Point3, C>| -> Edge<_, _> {
        let new_edge = edge_map.get(&edge.id()).unwrap();
        match edge.orientation() {
            true => new_edge.clone(),
            false => new_edge.inverse(),
        }
    };
    let create_boundary =
        |wire: &Wire<Point3, C>| -> Wire<_, _> { wire.edge_iter().map(create_edge).collect() };
    let create_face = move |face: &Face<Point3, C, S>| -> (FaceID<S>, Face<_, _, _>) {
        let wires: Vec<_> = face
            .absolute_boundaries()
            .iter()
            .map(create_boundary)
            .collect();
        let meshed_face =
            shell_create_polygon(&face.surface(), wires, face.orientation(), tol, &sp);
        (face.id(), meshed_face)
    };
    let meshed_faces_by_brep_face = shell
        .face_iter()
        .map(create_face)
        .collect::<HashMap<_, _>>();

    let meshed_shell = meshed_faces_by_brep_face
        .iter()
        .map(|(_, face)| face.clone())
        .collect::<Shell<_, _, _>>();

    CadMeshedShell {
        meshed_shell,
        meshed_faces_by_brep_face,
    }
}

/// Tessellates faces
#[cfg(any(target_arch = "wasm32", test))]
pub fn shell_tessellation_single_thread<'a, C, S, F>(
    shell: &'a Shell<Point3, C, S>,
    tol: f64,
    sp: F,
) -> CadMeshedShell<S>
where
    C: PolylineableCurve + 'a,
    S: PreMeshableSurface + 'a,
    F: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)>,
{
    use truck_base::entry_map::FxEntryMap as EntryMap;
    use truck_topology::Vertex as TVertex;

    let mut vmap = EntryMap::new(
        move |v: &TVertex<Point3>| v.id(),
        move |v| v.mapped(Point3::clone),
    );
    let mut edge_map = EntryMap::new(
        move |edge: &'a Edge<Point3, C>| edge.id(),
        move |edge| {
            let vf = edge.absolute_front();
            let v0 = vmap.entry_or_insert(vf).clone();
            let vb = edge.absolute_back();
            let v1 = vmap.entry_or_insert(vb).clone();
            let curve = edge.curve();
            let poly = PolylineCurve::from_curve(&curve, curve.range_tuple(), tol);
            Edge::debug_new(&v0, &v1, poly)
        },
    );
    let mut create_edge = move |edge: &'a Edge<Point3, C>| -> Edge<_, _> {
        let new_edge = edge_map.entry_or_insert(edge);
        match edge.orientation() {
            true => new_edge.clone(),
            false => new_edge.inverse(),
        }
    };
    let mut create_boundary = move |wire: &'a Wire<Point3, C>| -> Wire<_, _> {
        wire.edge_iter().map(&mut create_edge).collect()
    };
    let create_face = move |face: &'a Face<Point3, C, S>| -> (FaceID<S>, Face<_, _, _>) {
        let wires: Vec<_> = face
            .absolute_boundaries()
            .iter()
            .map(&mut create_boundary)
            .collect();
        let meshed_face =
            shell_create_polygon(&face.surface(), wires, face.orientation(), tol, &sp);
        (face.id(), meshed_face)
    };
    let meshed_faces_by_brep_face = shell
        .face_iter()
        .map(create_face)
        .collect::<HashMap<_, _>>();

    let meshed_shell = meshed_faces_by_brep_face
        .iter()
        .map(|(_, face)| face.clone())
        .collect::<Shell<_, _, _>>();

    CadMeshedShell {
        meshed_shell,
        meshed_faces_by_brep_face,
    }
}
