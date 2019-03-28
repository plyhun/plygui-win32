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
mod image;
mod layout_linear;
mod message;
mod splitted;
mod text;
mod tray;
mod window;

default_markup_register_members!();
default_pub_use!();
