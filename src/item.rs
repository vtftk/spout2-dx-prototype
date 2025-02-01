use std::path::PathBuf;

use nalgebra::Vector2;

/// Definition of an item to be thrown
pub struct ThrowableItemDefinition {
    // Path to the throwable
    texture_path: PathBuf,
    // Whether to pixelate the texture when scaling during render
    pixelate: bool,

    /// Starting position for the thrown item
    initial_position: Vector2<f32>,
    /// Destination for the thrown item
    dest_position: Vector2<f32>,

    /// Angle the item is thrown at (deg)
    angle: f32,

    /// Initial rotation for the item (deg)
    initial_rotation: f32,
    /// Initial item scale (Without render scaling applied)
    initial_scale: f32,
}
