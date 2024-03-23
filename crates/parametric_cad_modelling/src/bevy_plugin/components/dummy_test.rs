use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct DummyBuilder<F>
where
    F: Fn(DummyInput) -> DummyOutput,
{
    pub build_fn: F,
}

#[derive(Debug, Clone, Deref, DerefMut, Reflect)]
pub struct DummyInput(pub f32);

#[derive(Debug, Clone, Deref, DerefMut, Reflect)]
pub struct DummyOutput(pub f32);

mod test {
    use super::*;

    #[test]
    pub fn test_fn() {
        let builder = DummyBuilder {
            build_fn: |input: DummyInput| DummyOutput(input.0 * 2.),
        };

        let input = DummyInput(2.);
        let output = (builder.build_fn)(input);
        assert_eq!(output.0, 4.);
    }
}