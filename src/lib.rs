#![cfg(target_os="windows")]
#![windows_subsystem = "windows"]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate plygui_api;

extern crate winapi;

#[macro_use]
pub mod common;

mod application;
mod window;
mod button;
//mod layout_linear;

pub use self::application::Application;
pub use self::window::Window;
pub use self::button::Button;
//pub use self::layout_linear::LinearLayout;

#[cfg(feature = "markup")]
pub fn register_members(registry: &mut plygui_api::markup::MarkupRegistry) {
    registry
        .register_member(plygui_api::markup::MEMBER_TYPE_BUTTON.into(), button::spawn)
        .unwrap();
    registry
        .register_member(
            plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT.into(),
            layout_linear::spawn,
        )
        .unwrap();
}
