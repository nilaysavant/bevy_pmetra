use bevy::{prelude::*, utils::HashMap};

use anyhow::{anyhow, Result};

use crate::cad_core::builders::CadShell;

/// Holds multiple [`CadShellLazyBuilder`]s.
#[derive(Clone, Default)]
pub struct CadShellsLazyBuilders<P: Default + Clone> {
    pub params: P,
    pub builders: HashMap<CadShellName, CadShellLazyBuilder<P>>,
}

impl<P: Default + Clone> CadShellsLazyBuilders<P> {
    pub fn new(params: P) -> Result<Self> {
        let builder = Self {
            params,
            ..default()
        };
        Ok(builder)
    }

    /// Add new [`CadShellLazyBuilder`] to builders.
    pub fn add_shell_builder(
        &mut self,
        shell_name: CadShellName,
        build_fn: fn(&P) -> Result<CadShell>,
    ) -> Result<Self> {
        let shell_builder = CadShellLazyBuilder {
            params: self.params.clone(),
            build_cad_shell: build_fn,
        };
        self.builders.insert(shell_name, shell_builder);
        Ok(self.clone())
    }

    /// Build [`CadShell`] using the stored [`CadShellLazyBuilder`] with `shell_name`.
    pub fn build_shell(&self, shell_name: CadShellName) -> Result<CadShell> {
        (self
            .builders
            .get(&shell_name)
            .ok_or_else(|| anyhow!("Could not find shell with name: {:?}", shell_name))?
            .build_cad_shell)(&self.params)
    }
}

/// Name of the [`CadShell`].
///
/// Used to identify the shell.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deref, DerefMut, Reflect, Component)]
pub struct CadShellName(pub String);

/// Builder for building [`CadShell`]s.
#[derive(Clone, Component)]
pub struct CadShellLazyBuilder<P: Default + Clone> {
    pub params: P,
    pub build_cad_shell: fn(&P) -> Result<CadShell>,
}

impl<P: Default + Clone> CadShellLazyBuilder<P> {
    pub fn new(params: P, build_fn: fn(&P) -> Result<CadShell>) -> Self {
        Self {
            params,
            build_cad_shell: build_fn,
        }
    }

    pub fn build_cad_shell(&self) -> Result<CadShell> {
        (self.build_cad_shell)(&self.params)
    }
}

/// Component to store all generated [`CadShell`]s by [`CadShellName`].
#[derive(Debug, Clone, Component, Deref, DerefMut, Default)]
pub struct CadShellsByName(pub HashMap<CadShellName, CadShell>);
