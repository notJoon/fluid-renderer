/// Trait for velocity field abstraction
/// 
/// This trait provides access to velocity components (x and y) for fluid simulation.
/// Velocities can be positive, negative, or special values (NaN, Inf).
pub trait VelocityField {
    /// Returns the grid width
    fn grid_width(&self) -> usize;
    
    /// Returns the grid height
    fn grid_height(&self) -> usize;
    
    /// Returns the x-component of velocity at grid position (i, j)
    fn velocity_x_at(&self, i: usize, j: usize) -> f32;
    
    /// Returns the y-component of velocity at grid position (i, j)
    fn velocity_y_at(&self, i: usize, j: usize) -> f32;
}