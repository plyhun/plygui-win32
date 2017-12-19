#![cfg(target_os="windows")]
#![windows_subsystem = "windows"]

#[macro_use]
extern crate lazy_static;

extern crate plygui_api;

extern crate winapi;
/*extern crate gdi32;
extern crate kernel32;
extern crate user32;
extern crate comctl32;
extern crate comdlg32;*/

#[macro_use]
pub mod common;

mod application;
mod window;
mod button;
mod layout_linear;
//mod layout_relative;

//pub type NativeId = winapi::shared::windef::HWND;

pub use self::application::Application;
pub use self::window::Window;
pub use self::button::Button;
pub use self::layout_linear::LinearLayout;
//pub use self::layout_relative::RelativeLayout;

#[cfg(feature = "markup")]
pub fn register_members(registry: &mut plygui_api::markup::MarkupRegistry) {
	//registry.insert(plygui_api::members::MEMBER_ID_BUTTON.into(), button::spawn);
	//registry.insert(plygui_api::members::MEMBER_ID_LAYOUT_LINEAR.into(), layout_linear::spawn);
	registry.register_member(plygui_api::markup::MEMBER_TYPE_BUTTON.into(), button::spawn).unwrap();
	registry.register_member(plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT.into(), layout_linear::spawn).unwrap();
}