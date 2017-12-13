#![cfg(target_os="windows")]
#![windows_subsystem = "windows"]

#[macro_use]
extern crate lazy_static;

extern crate plygui_api;

extern crate winapi;
extern crate gdi32;
extern crate kernel32;
extern crate user32;
extern crate comctl32;
extern crate comdlg32;

#[macro_use]
pub mod common;

mod application;
mod window;
mod button;
mod layout_linear;
//mod layout_relative;

pub type NativeId = winapi::HWND;

pub use self::application::Application;
pub use self::window::Window;
pub use self::button::Button;
pub use self::layout_linear::LinearLayout;
//pub use self::layout_relative::RelativeLayout;