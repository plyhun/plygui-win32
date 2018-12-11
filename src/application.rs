use super::*;

use std::borrow::Cow;
use std::{mem, thread};

use plygui_api::controls;
use plygui_api::development::*;
use plygui_api::ids::Id;
use plygui_api::types;

use winapi::shared::windef;
use winapi::um::commctrl;

pub struct WindowsApplication {
    name: String,
    windows: Vec<windef::HWND>,
}

pub type Application = ::plygui_api::development::Application<WindowsApplication>;

impl ApplicationInner for WindowsApplication {
    fn with_name(name: &str) -> Box<Application> {
        init_comctl();
        let a = Box::new(Application::with_inner(
            WindowsApplication {
                name: name.into(),
                windows: Vec::with_capacity(1),
            },
            (),
        ));
        a
    }
    fn new_window(&mut self, title: &str, size: types::WindowStartSize, menu: types::WindowMenu) -> Box<dyn controls::Window> {
        let mut w = window::WindowsWindow::with_params(title, size, menu);
        unsafe {
            use plygui_api::controls::SingleContainer;

            self.windows.push(
                w.as_single_container_mut()
                    .as_container_mut()
                    .as_member_mut()
                    .as_any_mut()
                    .downcast_mut::<window::Window>()
                    .unwrap()
                    .as_inner_mut()
                    .native_id()
                    .into(),
            );
        }
        w
    }
    fn name<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(self.name.as_str())
    }
    fn start(&mut self) {
        for i in (0..self.windows.len()).rev() {
            if i > 0 {
                thread::spawn(move || {});
            } else {
                start_window(self.windows[i]);
            }
        }
    }
    fn find_member_by_id_mut(&mut self, id: Id) -> Option<&mut dyn controls::Member> {
        use plygui_api::controls::{Container, Member, SingleContainer};

        for window in self.windows.as_mut_slice() {
            let window: &mut window::Window = common::member_from_hwnd::<window::Window>(*window);
            if window.id() == id {
                return Some(window.as_single_container_mut().as_container_mut().as_member_mut());
            } else {
                return window.find_control_by_id_mut(id).map(|control| control.as_member_mut());
            }
        }
        None
    }
    fn find_member_by_id(&self, id: Id) -> Option<&dyn controls::Member> {
        use plygui_api::controls::{Container, Member, SingleContainer};

        for window in self.windows.as_slice() {
            let window: &mut window::Window = common::member_from_hwnd::<window::Window>(*window);
            if window.id() == id {
                return Some(window.as_single_container().as_container().as_member());
            } else {
                return window.find_control_by_id_mut(id).map(|control| control.as_member());
            }
        }

        None
    }
}

fn start_window(hwnd: windef::HWND) {
    let w: &mut window::Window = common::member_from_hwnd::<window::Window>(hwnd);
    w.as_inner_mut().as_inner_mut().as_inner_mut().start();
}

fn init_comctl() {
    unsafe {
        let mut icc: commctrl::INITCOMMONCONTROLSEX = mem::zeroed();
        icc.dwSize = mem::size_of::<commctrl::INITCOMMONCONTROLSEX>() as u32;
        icc.dwICC = commctrl::ICC_STANDARD_CLASSES | commctrl::ICC_LISTVIEW_CLASSES | commctrl::ICC_TAB_CLASSES | commctrl::ICC_PROGRESS_CLASS | commctrl::ICC_UPDOWN_CLASS | commctrl::ICC_BAR_CLASSES;
        if commctrl::InitCommonControlsEx(&icc) == 0 {
            common::log_error();
        }
    }
}
