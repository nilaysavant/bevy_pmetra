use bevy::utils::HashMap;
use truck_meshalgo::{
    prelude::{nonpositive_tolerance, TOLERANCE},
    rexport_polymesh::PolygonMesh,
    tessellation::{
        triangulation::by_search_parameter, MeshableSurface, MeshedShape, PolylineableCurve,
        PreMeshableSurface,
    },
};
use truck_modeling::{Point3, Surface};
use truck_topology::{Face, FaceID, Shell};

pub mod triangulation;

/// Custom Trait adapted from [`MeshableShape`].
pub trait CustomMeshableShape<S: MeshableSurface> {
    /// Shape whose edges are made polylines and faces polygon surface.
    type MeshedShape: MeshedShape;
    /// Tessellates shapes. The division of curves and surfaces are by `ParameterDivision1D` and `ParameterDivision2D`,
    /// and the constrained Delauney triangulation is based on the crate [`spade`](https://crates.io/crates/spade).
    ///
    /// # Panics
    ///
    /// `tol` must be more than `TOLERANCE`.
    ///
    /// # Remarks
    ///
    /// - The tessellated mesh is not necessarily closed even if `self` is `Solid`.
    /// If you want to get closed mesh, use [`OptimizingFilter::put_together_same_attrs`].
    /// - This method requires that the curve ride strictly on a surface. If not, try [`RobustMeshableShape`].
    ///
    /// [`OptimizingFilter::put_together_same_attrs`]: crate::filters::OptimizingFilter::put_together_same_attrs
    ///
    /// # Examples
    /// ```
    /// use truck_meshalgo::prelude::*;
    /// use truck_modeling::builder;
    /// use truck_topology::shell::ShellCondition;
    ///
    /// // modeling a unit cube
    /// let v = builder::vertex(Point3::origin());
    /// let e = builder::tsweep(&v, Vector3::unit_x());
    /// let f = builder::tsweep(&e, Vector3::unit_y());
    /// let cube = builder::tsweep(&f, Vector3::unit_z());
    ///
    /// // cube is Solid, however, the tessellated mesh is not closed.
    /// let mut mesh = cube.triangulation(0.01).to_polygon();
    /// assert_ne!(mesh.shell_condition(), ShellCondition::Closed);
    ///
    /// // use optimization filters!
    /// mesh.put_together_same_attrs(TOLERANCE);
    /// assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
    /// ```
    fn triangulation(&self, tol: f64) -> CadMeshedShell<S>;
}

impl<C: PolylineableCurve, S: MeshableSurface> CustomMeshableShape<S> for Shell<Point3, C, S> {
    type MeshedShape = Shell<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn triangulation(&self, tol: f64) -> CadMeshedShell<S> {
        nonpositive_tolerance!(tol);
        #[cfg(not(target_arch = "wasm32"))]
        let res = triangulation::shell_tessellation(self, tol, by_search_parameter);
        #[cfg(target_arch = "wasm32")]
        let res = triangulation::shell_tessellation_single_thread(self, tol, by_search_parameter);
        res
    }
}

/// [`MeshedShell`] wrapped for CAD application.
#[derive(Debug, Clone)]
pub struct CadMeshedShell<S> {
    pub meshed_shell: MeshedShell,
    pub meshed_faces_by_brep_face:
        HashMap<FaceID<S>, Face<Point3, PolylineCurve, Option<PolygonMesh>>>,
}

type PolylineCurve = truck_modeling::PolylineCurve<Point3>;
type MeshedShell = Shell<Point3, PolylineCurve, Option<PolygonMesh>>;
