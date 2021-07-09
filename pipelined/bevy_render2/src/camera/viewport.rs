use std::ops::Range;

use bevy_math::Vec2;

#[derive(Clone)]
pub struct Viewport {
    pub origin: Vec2,
    pub size: Vec2,
    pub depth_range: Range<f32>,
}
