#![cfg(target_os="windows")]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derive_builder;

extern crate plygui;

extern crate winapi;
extern crate gdi32;
extern crate kernel32;
extern crate user32;
extern crate comctl32;
extern crate comdlg32;

pub mod common;

mod application;
mod window;
mod button;
mod layout_linear;
mod layout_relative;

pub type NativeId = winapi::HWND;

pub use self::application::Application;
pub use self::window::Window;
pub use self::button::Button;
pub use self::layout_linear::LinearLayout;
pub use self::layout_relative::RelativeLayout;