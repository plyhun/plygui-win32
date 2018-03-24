use super::*;
use super::common::WindowsContainer;

use std::{mem, thread};
use std::borrow::Cow;

use plygui_api::traits::{UiApplication, UiWindow, UiMember};
use plygui_api::types::WindowStartSize;
use plygui_api::ids::Id;
use plygui_api::development;

use winapi::shared::windef;
use winapi::um::winuser;
use winapi::um::commctrl;

pub struct WindowsApplication {
    name: String,
    windows: Vec<windef::HWND>,
}

pub type Application = WindowsApplication;

impl development::ApplicationInner for WindowsApplication {
    fn new_window(&mut self, title: &str, size: WindowStartSize, has_menu: bool) -> Box<UiWindow> {
        let w = window::WindowsWindow::new(title, size, has_menu);
        unsafe {
            self.windows.push(w.hwnd());
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
    fn find_member_by_id_mut(&mut self, id: Id) -> Option<&mut UiMember> {
        use plygui_api::traits::UiContainer;

        for window in self.windows.as_mut_slice() {
            let window = unsafe { common::cast_hwnd::<Window>(*window) };
            if window.as_base().id() == id {
                return Some(window);
            } else {
                return window.find_control_by_id_mut(id).map(|control| {
                    control.as_member_mut()
                });
            }
        }
        None
    }
    fn find_member_by_id(&self, id: Id) -> Option<&UiMember> {
        use plygui_api::traits::UiContainer;

        for window in self.windows.as_slice() {
            let window = unsafe { common::cast_hwnd::<Window>(*window) };
            if window.as_base().id() == id {
                return Some(window);
            } else {
                return window.find_control_by_id_mut(id).map(|control| {
                    control.as_member()
                });
            }
        }

        None
    }
}

impl WindowsApplication {
    pub fn with_name(name: &str) -> Box<UiApplication> {
        init_comctl();
        Box::new(development::Application {
	        inner: Application {
	            name: name.into(),
	            windows: Vec::with_capacity(1),
	        }
        })
    }
}

fn start_window(hwnd: windef::HWND) {
    let w: &mut Window = unsafe { mem::transmute(winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA)) };
    w.start();
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
