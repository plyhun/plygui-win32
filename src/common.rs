use super::*;

use std::{ptr, mem, str};
use std::os::raw::c_void;
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

use plygui::{development, layout, Id, UiContainer, UiMember, UiControl, UiRoleMut, Visibility};

pub static mut INSTANCE: winapi::HINSTANCE = 0 as winapi::HINSTANCE;

#[repr(C)]
pub struct WindowsControlBase {
	id: Id,
    layout: development::layout::LayoutBase,
    visibility: Visibility,
    
    pub hwnd: winapi::HWND,
    pub subclass_id: u64,
    pub coords: Option<(i32, i32)>,
    pub measured_size: (u16, u16),

    pub h_resize: Option<Box<FnMut(&mut UiMember, u16, u16)>>,
}

impl Default for WindowsControlBase {
    fn default() -> WindowsControlBase {
        WindowsControlBase {
        	id: Id::next(),
            hwnd: 0 as winapi::HWND,
            h_resize: None,
            subclass_id: 0,
            layout: development::layout::LayoutBase {
	            width: layout::Size::MatchParent,
				height: layout::Size::WrapContent,
				gravity: layout::gravity::CENTER_HORIZONTAL | layout::gravity::TOP,
				orientation: layout::Orientation::Vertical,
				alignment: layout::Alignment::None,
            },
            measured_size: (0, 0),
            coords: None,
            visibility: Visibility::Visible,
        }
    }
}
impl WindowsControlBase {
	fn invalidate(&mut self) {
		unsafe {
			let self_hwnd = self.hwnd;
			let parent_hwnd = self.parent_hwnd();	
			if let Some(parent) = self.parent_mut() {
				let parent_hwnd = parent_hwnd.unwrap();
				if let Some(real_self) = cast_hwnd_to_uimember(self_hwnd) {
					if let Some(control_self) = real_self.is_control_mut() {
						let (pw, ph) = parent.size();
						let wparent = common::cast_hwnd_to_windows(parent_hwnd);
						/*let (_,_,changed) = */control_self.measure(pw, ph);
						control_self.draw(None);
							
						//if changed {
							if let Some(wparent) = wparent {
								wparent.as_base_mut().invalidate();
							} 
						//} 
					}
				}
				if parent_hwnd != 0 as winapi::HWND {
	        		user32::InvalidateRect(parent_hwnd, ptr::null_mut(), winapi::TRUE);
	        	}
	        }
		}
	}
	pub fn layout_width(&self) -> layout::Size {
    	self.layout.width
    }
	pub fn layout_height(&self) -> layout::Size {
		self.layout.height
	}
	pub fn layout_gravity(&self) -> layout::Gravity {
		self.layout.gravity
	}
	pub fn layout_orientation(&self) -> layout::Orientation {
		self.layout.orientation
	}
	pub fn layout_alignment(&self) -> layout::Alignment {
		self.layout.alignment
	}
	
	pub fn set_layout_width(&mut self, width: layout::Size) {
		self.layout.width = width;
		self.invalidate();
	}
	pub fn set_layout_height(&mut self, height: layout::Size) {
		self.layout.height = height;
		self.invalidate();
	}
	pub fn set_layout_gravity(&mut self, gravity: layout::Gravity) {
		self.layout.gravity = gravity;
		self.invalidate();
	}
	pub fn set_layout_orientation(&mut self, orientation: layout::Orientation) {
		self.layout.orientation = orientation;
		self.invalidate();
	}
	pub fn set_layout_alignment(&mut self, alignment: layout::Alignment) {
		self.layout.alignment = alignment;
		self.invalidate();
	}
    
    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
        unsafe {
            user32::ShowWindow(self.hwnd,
                               if self.visibility == Visibility::Invisible {
                                   winapi::SW_HIDE
                               } else {
                                   winapi::SW_SHOW
                               });
            self.invalidate();
        }
    }
    pub fn id(&self) -> Id {
    	self.id
    }
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }
    pub fn parent_hwnd(&self) -> Option<winapi::HWND> {
    	unsafe {
    		let parent_hwnd = user32::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                None
            } else {
            	Some(parent_hwnd)
            }
    	}
    }
    pub fn parent(&self) -> Option<&UiContainer> {
        unsafe {
            let parent_hwnd = user32::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = user32::GetWindowLongPtrW(parent_hwnd, winapi::GWLP_USERDATA);
            let parent_class = String::from_utf16_lossy(get_class_name_by_hwnd(parent_hwnd).as_ref());
            match parent_class.as_str() {
                development::CLASS_ID_LAYOUT_LINEAR => {
                    let ll: &layout_linear::LinearLayout = mem::transmute(parent_ptr as *mut c_void);
                    return Some(ll);
                }
                development::CLASS_ID_WINDOW => {
                    let w: &window::Window = mem::transmute(parent_ptr as *mut c_void);
                    return Some(w);
                }
                _ => None,
            }
        }
    }
    pub fn parent_mut(&mut self) -> Option<&mut UiContainer> {
        unsafe {
            let parent_hwnd = user32::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = user32::GetWindowLongPtrW(parent_hwnd, winapi::GWLP_USERDATA);
            let parent_class = String::from_utf16_lossy(get_class_name_by_hwnd(parent_hwnd).as_ref());
            match parent_class.as_str() {
                development::CLASS_ID_LAYOUT_LINEAR => {
                    let ll: &mut layout_linear::LinearLayout = mem::transmute(parent_ptr as *mut c_void);
                    return Some(ll);
                }
                development::CLASS_ID_WINDOW => {
                    let w: &mut window::Window = mem::transmute(parent_ptr as *mut c_void);
                    return Some(w);
                }
                _ => None,
            }
        }
    }
    pub fn root(&self) -> Option<&UiContainer> {
        unsafe {
            let parent_hwnd = user32::GetAncestor(self.hwnd, 2); //GA_ROOT
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = user32::GetWindowLongPtrW(parent_hwnd, winapi::GWLP_USERDATA);
            let parent_class = String::from_utf16_lossy(get_class_name_by_hwnd(parent_hwnd).as_ref());
            match parent_class.as_str() {
                development::CLASS_ID_LAYOUT_LINEAR => {
                    let ll: &layout_linear::LinearLayout = mem::transmute(parent_ptr as *mut c_void);
                    return Some(ll);
                }
                development::CLASS_ID_WINDOW => {
                    let w: &window::Window = mem::transmute(parent_ptr as *mut c_void);
                    return Some(w);
                }
                _ => {
                    println!("unknown {}", parent_class);
                    None
                }
            }
        }
    }
    pub fn root_mut(&mut self) -> Option<&mut UiContainer> {
        unsafe {
            let parent_hwnd = user32::GetAncestor(self.hwnd, 2); //GA_ROOT
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = user32::GetWindowLongPtrW(parent_hwnd, winapi::GWLP_USERDATA);
            let parent_class = String::from_utf16_lossy(get_class_name_by_hwnd(parent_hwnd).as_ref());
            match parent_class.as_str() {
                development::CLASS_ID_LAYOUT_LINEAR => {
                    let ll: &mut layout_linear::LinearLayout = mem::transmute(parent_ptr as *mut c_void);
                    return Some(ll);
                }
                development::CLASS_ID_WINDOW => {
                    let w: &mut window::Window = mem::transmute(parent_ptr as *mut c_void);
                    return Some(w);
                }
                _ => {
                    println!("unknown {}", parent_class);
                    None
                }
            }
        }
    }
}

pub unsafe trait WindowsControl: UiMember {
    unsafe fn on_added_to_container(&mut self, &WindowsContainer, x: u16, y: u16);
    unsafe fn on_removed_from_container(&mut self, &WindowsContainer);
    fn as_base(&self) -> &WindowsControlBase;
    fn as_base_mut(&mut self) -> &mut WindowsControlBase;
}

pub unsafe trait WindowsContainer: UiContainer + UiMember {
    unsafe fn hwnd(&self) -> winapi::HWND;
}

pub unsafe fn get_class_name_by_hwnd(hwnd: winapi::HWND) -> Vec<u16> {
    let mut max_id = 256;
    let mut name = vec![0u16; max_id];
    max_id = user32::GetClassNameW(hwnd, name.as_mut_slice().as_ptr(), max_id as i32) as usize;
    name.truncate(max_id);
    name
}

pub unsafe fn create_control_hwnd(x: i32,
                                  y: i32,
                                  w: i32,
                                  h: i32,
                                  parent: winapi::HWND,
                                  ex_style: winapi::DWORD,
                                  class_name: winapi::LPCWSTR,
                                  control_name: &str,
                                  style: winapi::DWORD,
                                  param: winapi::LPVOID,
                                  handler: Option<unsafe extern "system" fn(winapi::HWND,
                                                                            msg: winapi::UINT,
                                                                            winapi::WPARAM,
                                                                            winapi::LPARAM,
                                                                            u64,
                                                                            u64)
                                                                            -> i64>)
                                  -> (winapi::HWND, u64) {
    let mut style = style;
    if (style & winapi::WS_TABSTOP) != 0 {
        style |= winapi::WS_GROUP;
    }
	#[allow(deprecated)]
    let subclass_id = {
        use std::hash::{Hasher, SipHasher};

        let mut hasher = SipHasher::new();
        hasher.write_usize(class_name as usize);
        hasher.finish()
    };
    let control_name = OsStr::new(control_name)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
    let hwnd = user32::CreateWindowExW(ex_style,
                                       class_name,
                                       control_name.as_ptr(),
                                       style | winapi::WS_CHILD | winapi::WS_VISIBLE,
                                       x,
                                       y,
                                       w,
                                       h,
                                       parent,
                                       ptr::null_mut(),
                                       INSTANCE,
                                       param);
    log_error();
    comctl32::SetWindowSubclass(hwnd, handler, subclass_id, param as u64);
    log_error();
    (hwnd, subclass_id)
}

pub fn destroy_hwnd(hwnd: winapi::HWND,
                    subclass_id: u64,
                    handler: Option<unsafe extern "system" fn(winapi::HWND,
                                                              msg: winapi::UINT,
                                                              winapi::WPARAM,
                                                              winapi::LPARAM,
                                                              u64,
                                                              u64)
                                                              -> i64>) {
    unsafe {
        if subclass_id != 0 {
            comctl32::RemoveWindowSubclass(hwnd, handler, subclass_id);
        }
        if user32::DestroyWindow(hwnd) == 0 {
            //panic!("Cannot destroy window!");
        }
    }
}

pub unsafe fn window_rect(hwnd: winapi::HWND) -> winapi::RECT {
    let mut rect: winapi::RECT = mem::zeroed();
    user32::GetWindowRect(hwnd, &mut rect);
    rect
}

pub unsafe fn cast_uicontrol_to_windows(input: &mut Box<UiControl>) -> &mut WindowsControl {
    use std::ops::DerefMut;
    match input.role_mut() {
        UiRoleMut::Button(_) => {
            let a: &mut Box<button::Button> = mem::transmute(input);
            a.deref_mut()
        }
        UiRoleMut::LinearLayout(_) => {
            let a: &mut Box<layout_linear::LinearLayout> = mem::transmute(input);
            a.deref_mut()
        }
        UiRoleMut::Window(_) => {
            panic!("Window as a container child is impossible!");
        }
    }
}

pub unsafe fn cast_hwnd_to_windows<'a>(hwnd: winapi::HWND) -> Option<&'a mut WindowsControl> {
	let hwnd_ptr = user32::GetWindowLongPtrW(hwnd, winapi::GWLP_USERDATA);
    let parent_class = String::from_utf16_lossy(get_class_name_by_hwnd(hwnd).as_ref());
    match parent_class.as_str() {
        development::CLASS_ID_LAYOUT_LINEAR => {
            let ll: &mut layout_linear::LinearLayout = mem::transmute(hwnd_ptr as *mut c_void);
            return Some(ll);
        },
        /*development::CLASS_ID_WINDOW => {
            let w: &mut window::Window = mem::transmute(hwnd_ptr as *mut c_void);
            return Some(w);
        },*/
        button::CLASS_ID => {
            let b: &mut button::Button = mem::transmute(hwnd_ptr as *mut c_void);
            return Some(b);
        },
        _ => None,
    }
}

pub unsafe fn cast_hwnd_to_uimember<'a>(hwnd: winapi::HWND) -> Option<&'a mut UiMember> {// TODO merge with above using T: Sized
	let hwnd_ptr = user32::GetWindowLongPtrW(hwnd, winapi::GWLP_USERDATA);
    let parent_class = String::from_utf16_lossy(get_class_name_by_hwnd(hwnd).as_ref());
    match parent_class.as_str() {
        development::CLASS_ID_LAYOUT_LINEAR => {
            let ll: &mut layout_linear::LinearLayout = mem::transmute(hwnd_ptr as *mut c_void);
            return Some(ll);
        },
        development::CLASS_ID_WINDOW => {
            let w: &mut window::Window = mem::transmute(hwnd_ptr as *mut c_void);
            return Some(w);
        },
        button::CLASS_ID => {
            let b: &mut button::Button = mem::transmute(hwnd_ptr as *mut c_void);
            return Some(b);
        },
        _ => {
        	println!("Unsupported {}", parent_class);
	        None
        },
    }
}

pub unsafe fn log_error() {
    let error = kernel32::GetLastError();
    if error == 0 {
        return;
    }

    let mut string = vec![0u16; 127];
    kernel32::FormatMessageW(winapi::FORMAT_MESSAGE_FROM_SYSTEM | winapi::FORMAT_MESSAGE_IGNORE_INSERTS,
                             ptr::null_mut(),
                             error,
                             winapi::LANG_SYSTEM_DEFAULT as u32,
                             string.as_mut_ptr(),
                             string.len() as u32,
                             ptr::null_mut());

    println!("Last error #{}: {}",
             error,
             String::from_utf16_lossy(&string));
}
