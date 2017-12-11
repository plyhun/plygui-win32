use super::*;

use plygui_api::{layout, ids, types, development};
use plygui_api::traits::{UiControl, UiLayedOut, UiButton, UiMember, UiContainer};
use plygui_api::members::MEMBER_ID_BUTTON;

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
                             base: common::WindowsControlBase::with_params(
                             	invalidate_impl,
	                             development::UiMemberFunctions {
		                             fn_member_id: member_id,
								     fn_is_control: is_control,
								     fn_is_control_mut: is_control_mut,
								     fn_size: size,
	                             },
                             ),
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
	fn on_added_to_container(&mut self, parent: &UiContainer, x: u16, y: u16) {
		use plygui_api::development::UiDrawable;
		
		let selfptr = self as *mut _ as *mut c_void;
        let (pw, ph) = parent.size();
        let (hwnd, id) = unsafe {
        	self.base.hwnd = parent.native_id() as winapi::HWND; // required for measure, as we don't have own hwnd yet
	        let (w, h, _) = self.measure(pw, ph);
	        common::create_control_hwnd(x as i32,
	                                                     y as i32,
	                                                     w as i32,
	                                                     h as i32,
	                                                     parent.native_id() as winapi::HWND,
	                                                     0,
	                                                     WINDOW_CLASS.as_ptr(),
	                                                     self.label.as_str(),
	                                                     winapi::BS_PUSHBUTTON | winapi::WS_TABSTOP,
	                                                     selfptr,
	                                                     Some(handler))
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
	}
    fn on_removed_from_container(&mut self, _: &UiContainer) {
    	common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, Some(handler));
        self.base.hwnd = 0 as winapi::HWND;
        self.base.subclass_id = 0;
    }
    
    fn is_container_mut(&mut self) -> Option<&mut UiContainer> {
        None
    }
    fn is_container(&self) -> Option<&UiContainer> {
        None
    }

    fn parent(&self) -> Option<&types::UiMemberCommon> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut types::UiMemberCommon> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&types::UiMemberCommon> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut types::UiMemberCommon> {
        self.base.root_mut()
    }
}

impl UiLayedOut for Button {
	fn layout_width(&self) -> layout::Size {
    	self.base.control_base.layout.width
    }
	fn layout_height(&self) -> layout::Size {
		self.base.control_base.layout.height
	}
	fn layout_gravity(&self) -> layout::Gravity {
		self.base.control_base.layout.gravity
	}
	fn layout_orientation(&self) -> layout::Orientation {
		self.base.control_base.layout.orientation
	}
	fn layout_alignment(&self) -> layout::Alignment {
		self.base.control_base.layout.alignment
	}
	
	fn set_layout_width(&mut self, width: layout::Size) {
		self.base.control_base.layout.width = width;
		self.base.invalidate();
	}
	fn set_layout_height(&mut self, height: layout::Size) {
		self.base.control_base.layout.height = height;
		self.base.invalidate();
	}
	fn set_layout_gravity(&mut self, gravity: layout::Gravity) {
		self.base.control_base.layout.gravity = gravity;
		self.base.invalidate();
	}
	fn set_layout_orientation(&mut self, orientation: layout::Orientation) {
		self.base.control_base.layout.orientation = orientation;
		self.base.invalidate();
	}
	fn set_layout_alignment(&mut self, alignment: layout::Alignment) {
		self.base.control_base.layout.alignment = alignment;
		self.base.invalidate();
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
    
    fn set_visibility(&mut self, visibility: types::Visibility) {
        self.base.control_base.member_base.visibility = visibility;
        unsafe {
            user32::ShowWindow(self.base.hwnd,
                               if self.base.control_base.member_base.visibility == types::Visibility::Invisible {
                                   winapi::SW_HIDE
                               } else {
                                   winapi::SW_SHOW
                               });
            self.base.invalidate();
        }
    }
    fn visibility(&self) -> types::Visibility {
        self.base.control_base.member_base.visibility
    } 

    fn member_id(&self) -> &'static str {
	    self.base.control_base.member_base.member_id()
    }
    fn id(&self) -> ids::Id {
    	self.base.id()
    }
    unsafe fn native_id(&self) -> usize {
	    self.base.hwnd as usize
    }
    fn is_control(&self) -> Option<&UiControl> {
    	Some(self)
    }
    fn is_control_mut(&mut self) -> Option<&mut UiControl> {
    	Some(self)
    }     
}

impl development::UiDrawable for Button {
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
    	
    	self.base.measured_size = match self.visibility() {
    		types::Visibility::Gone => (0, 0),
    		_ => {
    			unsafe {
		            let mut label_size: winapi::SIZE = mem::zeroed();
		            let w = match self.layout_width() {
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
		            let h = match self.layout_height() {
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
}

impl Drop for Button {
    fn drop(&mut self) {
        self.set_visibility(types::Visibility::Gone);
        common::destroy_hwnd(self.base.hwnd, 0, None);
    }
}

unsafe extern "system" fn handler(hwnd: winapi::HWND, msg: winapi::UINT, wparam: winapi::WPARAM, lparam: winapi::LPARAM, _: u64, param: u64) -> i64 {
    let button: &mut Button = mem::transmute(param);
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

impl_invalidate!(Button);
impl_is_control!(Button);
impl_size!(Button);
impl_member_id!(MEMBER_ID_BUTTON);