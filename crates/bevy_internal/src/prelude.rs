#[doc(hidden)]
pub use crate::{
    app::prelude::*, core::prelude::*, ecs::prelude::*, input::prelude::*, log::prelude::*,
    math::prelude::*, reflect::prelude::*, transform::prelude::*, window::prelude::*,
    DefaultPlugins, MinimalPlugins,
};

pub use bevy_derive::bevy_main;

#[doc(hidden)]
#[cfg(feature = "bevy_asset")]
pub use crate::asset::prelude::*;

#[doc(hidden)]
#[cfg(feature = "bevy_audio")]
pub use crate::audio::prelude::*;

#[doc(hidden)]
#[cfg(feature = "bevy_pbr")]
pub use crate::pbr::prelude::*;

#[doc(hidden)]
#[cfg(feature = "bevy_render")]
pub use crate::render::prelude::*;

#[doc(hidden)]
#[cfg(feature = "bevy_sprite")]
pub use crate::sprite::prelude::*;

#[doc(hidden)]
#[cfg(feature = "bevy_scene")]
pub use crate::scene::prelude::*;

#[doc(hidden)]
#[cfg(feature = "bevy_text")]
pub use crate::text::prelude::*;

#[doc(hidden)]
#[cfg(feature = "bevy_ui")]
pub use crate::ui::prelude::*;

#[doc(hidden)]
#[cfg(feature = "bevy_dynamic_plugin")]
pub use crate::dynamic_plugin::*;

#[doc(hidden)]
#[cfg(feature = "bevy_gilrs")]
pub use crate::gilrs::*;
