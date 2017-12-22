use std::{ptr, mem, str};
use std::os::raw::c_void;
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

use plygui_api::{layout, development, ids, types};
use plygui_api::traits::{UiMember, UiContainer};

use winapi::shared::windef;
use winapi::shared::minwindef;
use winapi::shared::ntdef;
use winapi::um::winuser;
use winapi::um::winbase;
use winapi::um::commctrl;
use winapi::um::errhandlingapi;

pub static mut INSTANCE: minwindef::HINSTANCE = 0 as minwindef::HINSTANCE;

#[repr(C)]
pub struct WindowsControlBase {
	pub control_base: development::UiControlCommon, 
	
    pub hwnd: windef::HWND,
    pub subclass_id: usize,
    pub coords: Option<(i32, i32)>,
    pub measured_size: (u16, u16),

    pub h_resize: Option<types::ResizeCallback>,
    
    invalidate: unsafe fn(this: &mut WindowsControlBase),
}

impl WindowsControlBase {
	pub fn with_params(invalidate: unsafe fn(this: &mut WindowsControlBase), functions: development::UiMemberFunctions) -> WindowsControlBase {
        WindowsControlBase {
        	control_base: development::UiControlCommon {
	        	member_base: development::UiMemberCommon::with_params(types::Visibility::Visible, functions),
		        layout: development::layout::LayoutBase {
		            width: layout::Size::MatchParent,
					height: layout::Size::WrapContent,
					gravity: layout::gravity::CENTER_HORIZONTAL | layout::gravity::TOP,
					orientation: layout::Orientation::Vertical,
					alignment: layout::Alignment::None,
	            },
        	},
        	hwnd: 0 as windef::HWND,
            h_resize: None,
            subclass_id: 0,
            measured_size: (0, 0),
            coords: None,
            
            invalidate: invalidate,
        }
    }
	
	pub fn invalidate(&mut self) {
		unsafe { (self.invalidate)(self) }
	}
	pub fn id(&self) -> ids::Id {
    	self.control_base.member_base.id
    }
    pub fn parent_hwnd(&self) -> Option<windef::HWND> {
    	unsafe {
    		let parent_hwnd = winuser::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                None
            } else {
            	Some(parent_hwnd)
            }
    	}
    }
    pub fn parent(&self) -> Option<&types::UiMemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn parent_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn root(&self) -> Option<&types::UiMemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetAncestor(self.hwnd, 2); //GA_ROOT
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn root_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetAncestor(self.hwnd, 2); //GA_ROOT
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
}

pub unsafe trait WindowsContainer: UiContainer + UiMember {
    unsafe fn hwnd(&self) -> windef::HWND;
}

pub unsafe fn get_class_name_by_hwnd(hwnd: windef::HWND) -> Vec<u16> {
    let mut max_id = 256;
    let mut name = vec![0u16; max_id];
    max_id = winuser::GetClassNameW(hwnd, name.as_mut_slice().as_ptr(), max_id as i32) as usize;
    name.truncate(max_id);
    name
}

pub unsafe fn create_control_hwnd(x: i32,
                                  y: i32,
                                  w: i32,
                                  h: i32,
                                  parent: windef::HWND,
                                  ex_style: minwindef::DWORD,
                                  class_name: ntdef::LPCWSTR,
                                  control_name: &str,
                                  style: minwindef::DWORD,
                                  param: minwindef::LPVOID,
                                  handler: Option<unsafe extern "system" fn(windef::HWND,
                                                                            msg: minwindef::UINT,
                                                                            minwindef::WPARAM,
                                                                            minwindef::LPARAM,
                                                                            usize,
                                                                            usize)
                                                                            -> isize>)
                                  -> (windef::HWND, usize) {
    let mut style = style;
    if (style & winuser::WS_TABSTOP) != 0 {
        style |= winuser::WS_GROUP;
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
    let hwnd = winuser::CreateWindowExW(ex_style,
                                       class_name,
                                       control_name.as_ptr(),
                                       style | winuser::WS_CHILD | winuser::WS_VISIBLE,
                                       x,
                                       y,
                                       w,
                                       h,
                                       parent,
                                       ptr::null_mut(),
                                       INSTANCE,
                                       param);
    log_error();
    commctrl::SetWindowSubclass(hwnd, handler, subclass_id as usize, param as usize);
    log_error();
    (hwnd, subclass_id as usize)
}

pub fn destroy_hwnd(hwnd: windef::HWND,
                    subclass_id: usize,
                    handler: Option<unsafe extern "system" fn(windef::HWND,
                                                              msg: minwindef::UINT,
                                                              minwindef::WPARAM,
                                                              minwindef::LPARAM,
                                                              usize,
                                                              usize)
                                                              -> isize>) {
    unsafe {
        if subclass_id != 0 {
            commctrl::RemoveWindowSubclass(hwnd, handler, subclass_id);
        }
        if winuser::DestroyWindow(hwnd) == 0 {
            //panic!("Cannot destroy window!");
        }
    }
}

pub unsafe fn window_rect(hwnd: windef::HWND) -> windef::RECT {
    let mut rect: windef::RECT = mem::zeroed();
    winuser::GetClientRect(hwnd, &mut rect);
    rect
}

/*pub unsafe fn cast_uicontrol_to_windows(input: &mut Box<UiControl>) -> &mut WindowsControl {
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

pub unsafe fn cast_hwnd_to_windows<'a>(hwnd: HWND) -> Option<&'a mut WindowsControl> {
	let hwnd_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
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
        _ => None,
    }
}*/    

pub unsafe fn cast_hwnd<'a, T>(hwnd: windef::HWND) -> &'a mut T where T: Sized {// TODO merge with above using T: Sized
	let hwnd_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    mem::transmute(hwnd_ptr as *mut c_void)
}

pub unsafe fn log_error() {
    let error = errhandlingapi::GetLastError();
    if error == 0 {
        return;
    }

    let mut string = vec![0u16; 127];
    winbase::FormatMessageW(winbase::FORMAT_MESSAGE_FROM_SYSTEM | winbase::FORMAT_MESSAGE_IGNORE_INSERTS,
                             ptr::null_mut(),
                             error,
                             ntdef::LANG_SYSTEM_DEFAULT as u32,
                             string.as_mut_ptr(),
                             string.len() as u32,
                             ptr::null_mut());

    println!("Last error #{}: {}",
             error,
             String::from_utf16_lossy(&string));
}

#[macro_export]
macro_rules! impl_invalidate {
	($typ: ty) => {
		unsafe fn invalidate_impl(this: &mut common::WindowsControlBase) {
			use plygui_api::development::UiDrawable;
			
			let parent_hwnd = this.parent_hwnd();	
			if let Some(parent_hwnd) = parent_hwnd {
				let mparent = common::cast_hwnd::<plygui_api::development::UiMemberCommon>(parent_hwnd);
				let (pw, ph) = mparent.size();
				let this: &mut $typ = mem::transmute(this);
				//let (_,_,changed) = 
				this.measure(pw, ph);
				this.draw(None);		
						
				if mparent.is_control().is_some() {
					let wparent = common::cast_hwnd::<common::WindowsControlBase>(parent_hwnd);
					//if changed {
						wparent.invalidate();
					//} 
				}
				if parent_hwnd != 0 as ::winapi::shared::windef::HWND {
		    		::winapi::um::winuser::InvalidateRect(parent_hwnd, ptr::null_mut(), ::winapi::shared::minwindef::TRUE);
		    	}
		    }
		}
	}
}
#[macro_export]
macro_rules! impl_is_control {
	($typ: ty) => {
		unsafe fn is_control(this: &::plygui_api::development::UiMemberCommon) -> Option<&::plygui_api::development::UiControlCommon> {
			Some(&::plygui_api::utils::base_to_impl::<$typ>(this).base.control_base)
		}
		unsafe fn is_control_mut(this: &mut ::plygui_api::development::UiMemberCommon) -> Option<&mut ::plygui_api::development::UiControlCommon> {
			Some(&mut ::plygui_api::utils::base_to_impl_mut::<$typ>(this).base.control_base)
		}
	}
}
#[macro_export]
macro_rules! impl_size {
	($typ: ty) => {
		unsafe fn size(this: &::plygui_api::development::UiMemberCommon) -> (u16, u16) {
			::plygui_api::utils::base_to_impl::<$typ>(this).size()
		}
	}
}
#[macro_export]
macro_rules! impl_member_id {
	($mem: expr) => {
		unsafe fn member_id(_: &::plygui_api::development::UiMemberCommon) -> &'static str {
			$mem
		}
	}
}
#[macro_export]
macro_rules! impl_measure {
	($typ: ty) => {
		unsafe fn measure(&mut UiMemberCommon, w: u16, h: u16) -> (u16, u16, bool) {
			::plygui_api::utils::base_to_impl::<$typ>(this).measure(w, h)
		}
	}
}
#[macro_export]
macro_rules! impl_draw {
	($typ: ty) => {
		unsafe fn draw(&mut UiMemberCommon, coords: Option<(i32, i32)>) {
			::plygui_api::utils::base_to_impl::<$typ>(this).draw(coords)
		}
	}
}