use super::*;

use std::{mem, thread};
use std::borrow::Cow;

use plygui_api::traits::{UiApplication, UiWindow, UiMember};
use plygui_api::types::WindowStartSize;
use plygui_api::ids::Id;

use winapi::shared::windef;
use winapi::um::winuser;
use winapi::um::commctrl;

pub struct Application {
    name: String,
    window: windef::HWND,
}

impl UiApplication for Application {
    fn new_window(&mut self, title: &str, size: WindowStartSize, has_menu: bool) -> Box<UiWindow> {
        let w = Window::new(title, size, has_menu);
        unsafe {
            self.window = w.hwnd();
        }
        w
    }
    fn name<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(self.name.as_str())
    }
    fn start(&mut self) {
        if self.window > 0 {
            thread::spawn(move || {});
        } else {
            start_window(self.window);
        }
    }
    fn find_member_by_id_mut(&mut self, id: Id) -> Option<&mut UiMember> {
    	use plygui_api::traits::UiContainer;
    	
    	let window = unsafe { common::cast_hwnd::<Window>(self.window) };
		if window.as_base().id() == id {
			return Some(window);
		} else {
			return window.find_control_by_id_mut(id).map(|control| control.as_member_mut());
		}
	}
    fn find_member_by_id(&self, id: Id) -> Option<&UiMember> {
    	use plygui_api::traits::UiContainer;
    	
    	let window = unsafe { common::cast_hwnd::<Window>(self.window) };
		if window.as_base().id() == id {
			return Some(window);
		} else {
			return window.find_control_by_id_mut(id).map(|control| control.as_member());
		}
    }   
}

impl Application {
    pub fn with_name(name: &str) -> Box<Application> {
    	init_comctl();
    	//Id::next();
	    Box::new(Application {
                     name: name.into(),
                     window: 0,
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