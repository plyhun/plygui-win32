use super::*;
use super::common::WindowsContainer;

use std::{mem, thread};

use plygui::{Id, UiApplication, UiWindow};

pub struct Application {
    name: String,
    windows: Vec<winapi::HWND>,
}

impl UiApplication for Application {
    fn new_window(&mut self, title: &str, width: u16, height: u16, has_menu: bool) -> Box<UiWindow> {
        let w = Window::new(title, width, height, has_menu);
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
    	Id::next();
	    Box::new(Application {
                     name: name.into(),
                     windows: Vec::with_capacity(1),
                 })
    }
}

fn start_window(hwnd: winapi::HWND) {
	let w: &mut Window = unsafe { mem::transmute(user32::GetWindowLongPtrW(hwnd, winapi::GWLP_USERDATA)) };
    w.start();
}

fn init_comctl() {
	unsafe {
    	let mut icc: winapi::INITCOMMONCONTROLSEX = mem::zeroed();
    	icc.dwSize = mem::size_of::<winapi::INITCOMMONCONTROLSEX>() as u32;
		icc.dwICC = winapi::ICC_STANDARD_CLASSES 
			| winapi::ICC_LISTVIEW_CLASSES
			| winapi::ICC_TAB_CLASSES
			| winapi::ICC_PROGRESS_CLASS
			| winapi::ICC_UPDOWN_CLASS
			| winapi::ICC_BAR_CLASSES;
		if comctl32::InitCommonControlsEx(&icc) == 0 {
			common::log_error();
		}
	}        
}