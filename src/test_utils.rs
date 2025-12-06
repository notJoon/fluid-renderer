/// Testing utilities for fluid renderer
#[cfg(test)]
pub mod mock {
    use crate::fluid_field::FluidField;

    /// Mock implementation of FluidField for testing
    pub struct MockFluidSim {
        width: usize,
        height: usize,
        density: Vec<f32>,
    }

    impl MockFluidSim {
        pub fn new(width: usize, height: usize) -> Self {
            Self {
                width,
                height,
                density: vec![0.0; width * height],
            }
        }

        pub fn set_density(&mut self, i: usize, j: usize, value: f32) {
            if i < self.width && j < self.height {
                let idx = j * self.width + i;
                self.density[idx] = value;
            }
        }

        pub fn fill_density(&mut self, value: f32) {
            self.density.fill(value);
        }
    }

    impl FluidField for MockFluidSim {
        fn grid_width(&self) -> usize {
            self.width
        }

        fn grid_height(&self) -> usize {
            self.height
        }

        fn density_at(&self, i: usize, j: usize) -> f32 {
            if i < self.width && j < self.height {
                let idx = j * self.width + i;
                self.density[idx]
            } else {
                0.0
            }
        }
    }
}