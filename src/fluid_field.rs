/// Trait representing a fluid field with density data
pub trait FluidField {
    /// Get the width of the fluid grid
    fn grid_width(&self) -> usize;

    /// Get the height of the fluid grid
    fn grid_height(&self) -> usize;

    /// Get the density at grid position (i, j)
    /// Returns 0.0 if out of bounds
    fn density_at(&self, i: usize, j: usize) -> f32;
}

