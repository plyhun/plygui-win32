use super::*;

use plygui::{layout, Id, UiRole, UiRoleMut, Visibility, UiControl, UiButton, UiMember, UiContainer};

use std::{ptr, mem, str};
use std::os::raw::c_void;
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

pub const CLASS_ID: &str = "Button";

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
}

#[repr(C)]
pub struct Button {
    base: common::WindowsControlBase,
    label: String,
    h_left_clicked: Option<Box<FnMut(&mut UiButton)>>,
}

impl Button {
    pub fn new(label: &str) -> Box<Button> {
        let b = Box::new(Button {
                             base: Default::default(),
                             h_left_clicked: None,
                             label: label.to_owned(),
                         });

        b
    }
}

impl UiButton for Button {
    fn on_left_click(&mut self, handle: Option<Box<FnMut(&mut UiButton)>>) {
        self.h_left_clicked = handle;
    }
    fn label(&self) -> &str {
        self.label.as_ref()
    }
}

impl UiControl for Button {
    fn layout_width(&self) -> layout::Size {
    	self.base.layout_width()
    }
	fn layout_height(&self) -> layout::Size {
		self.base.layout_height()
	}
	fn layout_gravity(&self) -> layout::Gravity {
		self.base.layout_gravity()
	}
	fn layout_orientation(&self) -> layout::Orientation {
		self.base.layout_orientation()
	}
	fn layout_alignment(&self) -> layout::Alignment {
		self.base.layout_alignment()
	}
	
	fn set_layout_width(&mut self, width: layout::Size) {
		self.base.set_layout_width(width);
	}
	fn set_layout_height(&mut self, height: layout::Size) {
		self.base.set_layout_height(height);
	}
	fn set_layout_gravity(&mut self, gravity: layout::Gravity) {
		self.base.set_layout_gravity(gravity);
	}
	fn set_layout_orientation(&mut self, orientation: layout::Orientation) {
		self.base.set_layout_orientation(orientation);
	}
	fn set_layout_alignment(&mut self, alignment: layout::Alignment) {
		self.base.set_layout_alignment(alignment);
	}
    fn draw(&mut self, coords: Option<(i32, i32)>) {
    	if coords.is_some() {
    		self.base.coords = coords;
    	}
        if let Some((x, y)) = self.base.coords {
        	unsafe {
	            user32::SetWindowPos(self.base.hwnd,
	                                 ptr::null_mut(),
	                                 x as i32,
	                                 y as i32,
	                                 self.base.measured_size.0 as i32,
	                                 self.base.measured_size.1 as i32,
	                                 0);
	        }
        }
    }
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	let old_size = self.base.measured_size;
    	
    	self.base.measured_size = match self.base.visibility() {
    		Visibility::Gone => (0, 0),
    		_ => {
    			unsafe {
		            let mut label_size: winapi::SIZE = mem::zeroed();
		            let w = match self.base.layout_width() {
		                layout::Size::MatchParent => parent_width,
		                layout::Size::Exact(w) => w,
		                layout::Size::WrapContent => {
		                    if label_size.cx < 1 {
		                        let label = OsStr::new(self.label.as_str())
		                            .encode_wide()
		                            .chain(Some(0).into_iter())
		                            .collect::<Vec<_>>();
		                        gdi32::GetTextExtentPointW(user32::GetDC(self.base.hwnd),
		                                                   label.as_ptr(),
		                                                   self.label.len() as i32,
		                                                   &mut label_size);
		                    }
		                    label_size.cx as u16
		                } 
		            };
		            let h = match self.base.layout_height() {
		                layout::Size::MatchParent => parent_height,
		                layout::Size::Exact(h) => h,
		                layout::Size::WrapContent => {
		                    if label_size.cy < 1 {
		                        let label = OsStr::new(self.label.as_str())
		                            .encode_wide()
		                            .chain(Some(0).into_iter())
		                            .collect::<Vec<_>>();
		                        gdi32::GetTextExtentPointW(user32::GetDC(self.base.hwnd),
		                                                   label.as_ptr(),
		                                                   self.label.len() as i32,
		                                                   &mut label_size);
		                    }
		                    label_size.cy as u16
		                } 
		            };
		            (w, h)
		        }
    		}
    	};
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn is_container_mut(&mut self) -> Option<&mut UiContainer> {
        None
    }
    fn is_container(&self) -> Option<&UiContainer> {
        None
    }

    fn parent(&self) -> Option<&UiContainer> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut UiContainer> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&UiContainer> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut UiContainer> {
        self.base.root_mut()
    }
}

impl UiMember for Button {
    fn size(&self) -> (u16, u16) {
        let rect = unsafe { common::window_rect(self.base.hwnd) };
        ((rect.right - rect.left) as u16, (rect.bottom - rect.top) as u16)
    }

    fn on_resize(&mut self, handler: Option<Box<FnMut(&mut UiMember, u16, u16)>>) {
        self.base.h_resize = handler;
    }

    fn set_visibility(&mut self, visibility: Visibility) {
        self.base.set_visibility(visibility);
    }
    fn visibility(&self) -> Visibility {
        self.base.visibility()
    }

    fn role<'a>(&'a self) -> UiRole<'a> {
        UiRole::Button(self)
    }
    fn role_mut<'a>(&'a mut self) -> UiRoleMut<'a> {
        UiRoleMut::Button(self)
    }
    /*fn native_id(&self) -> NativeId {
        self.base.hwnd
    }*/
    fn id(&self) -> Id {
    	self.base.id()
    }
    fn is_control(&self) -> Option<&UiControl> {
    	Some(self)
    }
    fn is_control_mut(&mut self) -> Option<&mut UiControl> {
    	Some(self)
    }     
}

impl Drop for Button {
    fn drop(&mut self) {
        self.set_visibility(Visibility::Gone);
        common::destroy_hwnd(self.base.hwnd, 0, None);
    }
}

unsafe impl common::WindowsControl for Button {
    unsafe fn on_added_to_container(&mut self, parent: &common::WindowsContainer, px: u16, py: u16) {
        let selfptr = self as *mut _ as *mut c_void;
        let (pw, ph) = parent.size();
        self.base.hwnd = parent.hwnd(); // required for measure, as we don't have own hwnd yet
        let (w, h, _) = self.measure(pw, ph);
        let (hwnd, id) = common::create_control_hwnd(px as i32,
                                                     py as i32,
                                                     w as i32,
                                                     h as i32,
                                                     parent.hwnd(),
                                                     0,
                                                     WINDOW_CLASS.as_ptr(),
                                                     self.label.as_str(),
                                                     winapi::BS_PUSHBUTTON | winapi::WS_TABSTOP,
                                                     selfptr,
                                                     Some(handler));
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
    }
    unsafe fn on_removed_from_container(&mut self, _: &common::WindowsContainer) {
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, Some(handler));
        self.base.hwnd = 0 as winapi::HWND;
        self.base.subclass_id = 0;
    }
    fn as_base(&self) -> &common::WindowsControlBase {
    	&self.base
    }
    fn as_base_mut(&mut self) -> &mut common::WindowsControlBase {
    	&mut self.base
    }
}

unsafe extern "system" fn handler(hwnd: winapi::HWND, msg: winapi::UINT, wparam: winapi::WPARAM, lparam: winapi::LPARAM, _: u64, param: u64) -> i64 {
    let mut button: &mut Button = mem::transmute(param);
    let ww = user32::GetWindowLongPtrW(hwnd, winapi::GWLP_USERDATA);
    if ww == 0 {
        user32::SetWindowLongPtrW(hwnd, winapi::GWLP_USERDATA, param as i64);
    }
    match msg {
        winapi::WM_LBUTTONDOWN => {
            if let Some(ref mut cb) = button.h_left_clicked {
                let mut button2: &mut Button = mem::transmute(param);
                (cb)(button2);
            }
        }
        winapi::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            if let Some(ref mut cb) = button.base.h_resize {
                let mut button2: &mut Button = mem::transmute(param);
                (cb)(button2, width, height);
            }
        }
        _ => {}
    }

    comctl32::DefSubclassProc(hwnd, msg, wparam, lparam)
}
