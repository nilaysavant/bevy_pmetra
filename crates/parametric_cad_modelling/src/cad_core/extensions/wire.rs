use anyhow::{anyhow, Result};
use truck_modeling::{builder, Edge, Wire};

/// Extensions to [`Wire`] primitive.
pub trait WireCadExtension: Sized {
    /// Gets the closed [`Wire`] using a line [`Edge`] connecting back vertex to front vertex.
    fn get_closed_wire_with_line(&self) -> Result<Self>;
}

impl WireCadExtension for Wire {
    fn get_closed_wire_with_line(&self) -> Result<Self> {
        // create new wire to return...
        let mut wire = self.clone();

        let back_vert = wire
            .back_vertex()
            .ok_or_else(|| anyhow!("Could not get back vertex!"))?;
        let front_vert = wire
            .front_vertex()
            .ok_or_else(|| anyhow!("Could not get front vertex!"))?;
        let connecting_line = builder::line(back_vert, front_vert);
        // Add connecting edge...
        wire.push_back(connecting_line);

        Ok(wire)
    }
}
