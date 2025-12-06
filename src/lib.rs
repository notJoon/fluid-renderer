pub mod fluid_field;
pub mod frame_buffer;
pub mod renderer;
pub mod utils;

#[cfg(test)]
pub mod test_utils;

pub use fluid_field::FluidField;
pub use frame_buffer::{FrameBuffer, HeadlessBuffer};
pub use renderer::{RendererCore, RendererMinifb, rgb};
pub use utils::{FpsCounter, constants};
