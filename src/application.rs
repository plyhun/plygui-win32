use super::*;
use super::common::WindowsContainer;

use std::{mem, thread};

use plygui_api::traits::{UiApplication, UiWindow};
use plygui_api::types::WindowStartSize;

use winapi::shared::windef;
use winapi::um::winuser;
use winapi::um::commctrl;

pub struct Application {
    name: String,
    windows: Vec<windef::HWND>,
}

impl UiApplication for Application {
    fn new_window(&mut self, title: &str, size: WindowStartSize, has_menu: bool) -> Box<UiWindow> {
        let w = Window::new(title, size, has_menu);
        unsafe {
            self.windows.push(w.hwnd());
        }
        w
    }
    fn name(&self) -> &str {
        self.name.as_str()
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
}

impl Application {
    pub fn with_name(name: &str) -> Box<Application> {
    	init_comctl();
    	//Id::next();
	    Box::new(Application {
                     name: name.into(),
                     windows: Vec::with_capacity(1),
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
		icc.dwICC = commctrl::ICC_STANDARD_CLASSES 
			| commctrl::ICC_LISTVIEW_CLASSES
			| commctrl::ICC_TAB_CLASSES
			| commctrl::ICC_PROGRESS_CLASS
			| commctrl::ICC_UPDOWN_CLASS
			| commctrl::ICC_BAR_CLASSES;
		if commctrl::InitCommonControlsEx(&icc) == 0 {
			common::log_error();
		}
	}        
}