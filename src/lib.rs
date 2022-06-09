#![cfg(target_os = "windows")]
#![windows_subsystem = "windows"]

#![feature(new_uninit)]
#![allow(invalid_value)]
#![allow(type_alias_bounds)]

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
mod progress_bar;
mod list;
mod tree;
mod table;

default_markup_register_members!();
default_pub_use!();
