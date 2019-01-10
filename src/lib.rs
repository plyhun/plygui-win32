#![cfg(target_os = "windows")]
#![windows_subsystem = "windows"]
#![cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
#![cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
#![cfg_attr(feature = "cargo-clippy", allow(single_match))]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate plygui_api;

#[macro_use]
pub mod common;

mod application;
mod button;
mod frame;
mod layout_linear;
mod splitted;
mod window;
mod text;
mod alert;

#[cfg(feature = "markup")]
pub fn register_members(registry: &mut plygui_api::markup::MarkupRegistry) {
    registry.register_member(plygui_api::markup::MEMBER_TYPE_BUTTON.into(), button::spawn).unwrap();
    registry.register_member(plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT.into(), layout_linear::spawn).unwrap();
    registry.register_member(plygui_api::markup::MEMBER_TYPE_FRAME.into(), frame::spawn).unwrap();
    //registry.register_member(plygui_api::markup::MEMBER_TYPE_SPLITTED.into(), splitted::spawn).unwrap();
}

pub mod prelude {
    pub use plygui_api::callbacks;
    pub use plygui_api::controls::*;
    pub use plygui_api::ids::*;
    pub use plygui_api::layout;
    pub use plygui_api::types::*;
    pub use plygui_api::utils;

    pub mod imp {
        pub use crate::application::Application;
        pub use crate::button::Button;
        pub use crate::frame::Frame;
        pub use crate::layout_linear::LinearLayout;
        pub use crate::splitted::Splitted;
        pub use crate::window::Window;
        pub use crate::text::Text;
    }
}
