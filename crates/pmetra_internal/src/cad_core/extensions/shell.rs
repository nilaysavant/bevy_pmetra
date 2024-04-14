use anyhow::{anyhow, Result};
use truck_modeling::{Shell, Solid};

/// Extensions to [`Wire`] primitive.
pub trait ShellCadExtension: Sized {
    /// Create new shell from last boundary [`Shell`] in [`Solid`].
    fn try_from_solid(solid: &Solid) -> Result<Self>;
}

impl ShellCadExtension for Shell {
    fn try_from_solid(solid: &Solid) -> Result<Self> {
        let shell = solid
            .boundaries()
            .last()
            .ok_or_else(|| anyhow!("`from_solid` failed! Could not get last shell from solid!"))?
            .clone();

        Ok(shell)
    }
}
