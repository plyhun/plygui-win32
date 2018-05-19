use std::{ptr, mem, str};
use std::os::raw::c_void;
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

use plygui_api::development;
use plygui_api::traits::{UiMember, UiContainer};

use winapi::shared::windef;
use winapi::shared::minwindef;
use winapi::shared::ntdef;
use winapi::um::winuser;
use winapi::um::winbase;
use winapi::um::commctrl;
use winapi::um::errhandlingapi;
use winapi::um::libloaderapi;

pub fn hinstance() -> minwindef::HINSTANCE {
    *INSTANCE as *mut c_void as minwindef::HINSTANCE
}
lazy_static! {
	static ref INSTANCE: usize = unsafe { libloaderapi::GetModuleHandleW(ptr::null()) as usize };
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct Hwnd(windef::HWND);

impl From<windef::HWND> for Hwnd {
	fn from(a: windef::HWND) -> Hwnd {
		Hwnd(a)
	}
}
impl From<Hwnd> for windef::HWND {
	fn from(a: Hwnd) -> windef::HWND {
		a.0
	}
}
impl From<Hwnd> for usize {
	fn from(a: Hwnd) -> usize {
		a.0 as usize
	}
}
impl development::NativeId for Hwnd {}

#[repr(C)]
pub struct WindowsControlBase {
	pub hwnd: windef::HWND,
    pub subclass_id: usize,
    pub coords: Option<(i32, i32)>,
    pub measured_size: (u16, u16),
}

impl WindowsControlBase {
    pub fn new() -> WindowsControlBase {
        WindowsControlBase {
            hwnd: 0 as windef::HWND,
            subclass_id: 0,
            measured_size: (0, 0),
            coords: None,
        }
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
    pub fn parent(&self) -> Option<&development::MemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn parent_mut(&mut self) -> Option<&mut development::MemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn root(&self) -> Option<&development::MemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetAncestor(self.hwnd, 2); //GA_ROOT
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn root_mut(&mut self) -> Option<&mut development::MemberBase> {
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

pub unsafe fn create_control_hwnd(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    parent: windef::HWND,
    ex_style: minwindef::DWORD,
    class_name: ntdef::LPCWSTR,
    control_name: &str,
    style: minwindef::DWORD,
    param: minwindef::LPVOID,
    handler: Option<
        unsafe extern "system" fn(windef::HWND,
                                  msg: minwindef::UINT,
                                  minwindef::WPARAM,
                                  minwindef::LPARAM,
                                  usize,
                                  usize)
                                  -> isize,
    >,
) -> (windef::HWND, usize) {
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
    let os_control_name = OsStr::new(control_name)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
    let hwnd = winuser::CreateWindowExW(
        ex_style,
        class_name,
        os_control_name.as_ptr(),
        style | winuser::WS_CHILD | winuser::WS_VISIBLE,
        x,
        y,
        w,
        h,
        parent,
        ptr::null_mut(),
        hinstance(),
        param,
    );
    if hwnd.is_null() {
    	log_error();
    	panic!("Cannot create window {}", control_name);
    }
    commctrl::SetWindowSubclass(hwnd, handler, subclass_id as usize, param as usize);
    (hwnd, subclass_id as usize)
}

pub fn destroy_hwnd(
    hwnd: windef::HWND,
    subclass_id: usize,
    handler: Option<
        unsafe extern "system" fn(windef::HWND,
                                  msg: minwindef::UINT,
                                  minwindef::WPARAM,
                                  minwindef::LPARAM,
                                  usize,
                                  usize)
                                  -> isize,
    >,
) {
    unsafe {
        if subclass_id != 0 {
            commctrl::RemoveWindowSubclass(hwnd, handler, subclass_id);
        }
        if winuser::DestroyWindow(hwnd) == 0 && winuser::IsWindow(hwnd) > 0 {
	        log_error();
        }
    }
}

pub unsafe fn window_rect(hwnd: windef::HWND) -> windef::RECT {
    let mut rect: windef::RECT = mem::zeroed();
    winuser::GetClientRect(hwnd, &mut rect);
    rect
}

unsafe fn cast_hwnd<'a, T>(hwnd: windef::HWND) -> &'a mut T
where T: Sized// + UiMember,
{
    let hwnd_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    mem::transmute(hwnd_ptr as *mut c_void)
}
pub fn member_from_hwnd<'a, T>(hwnd: windef::HWND) -> &'a mut T where T: Sized + UiMember {
    unsafe { cast_hwnd(hwnd) }
}
pub fn member_base_from_hwnd<'a>(hwnd: windef::HWND) -> &'a mut development::MemberBase {
    unsafe { cast_hwnd(hwnd) }
}

#[cfg(not(debug_assertions))]
pub unsafe fn log_error() {}

#[cfg(debug_assertions)]
pub unsafe fn log_error() {
    let error = errhandlingapi::GetLastError();
    if error == 0 {
        return;
    }

    let mut string = vec![0u16; 127];
    winbase::FormatMessageW(
        winbase::FORMAT_MESSAGE_FROM_SYSTEM | winbase::FORMAT_MESSAGE_IGNORE_INSERTS,
        ptr::null_mut(),
        error,
        ntdef::LANG_SYSTEM_DEFAULT as u32,
        string.as_mut_ptr(),
        string.len() as u32,
        ptr::null_mut(),
    );

    println!(
        "Last error #{}: {}",
        error,
        String::from_utf16_lossy(&string)
    );
}

/*#[macro_export]
macro_rules! impl_invalidate {
	($typ: ty) => {
		unsafe fn invalidate_impl(this: &mut common::WindowsControlBase) {
			use plygui_api::development::UiDrawable;
			
			
		}
	}
}*/
