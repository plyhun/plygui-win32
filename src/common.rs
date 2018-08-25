pub use plygui_api::{controls, layout, types, callbacks, utils, ids};
pub use plygui_api::development::*;

pub use winapi::shared::windef;
pub use winapi::shared::minwindef;
pub use winapi::shared::ntdef;
pub use winapi::um::winuser;
pub use winapi::um::wingdi;
pub use winapi::um::winbase;
pub use winapi::um::commctrl;
pub use winapi::um::errhandlingapi;
pub use winapi::um::libloaderapi;
pub use winapi::ctypes::c_void;

pub use std::{ptr, mem, str, cmp};
pub use std::os::windows::ffi::OsStrExt;
pub use std::ffi::OsStr;
pub use std::borrow::Cow;
pub use std::marker::PhantomData;

pub const DEFAULT_PADDING: i32 = 6;
pub const WM_UPDATE_INNER: u32 = winuser::WM_APP + 1;

#[inline]
fn hfont() -> windef::HFONT { 
	*HFONT as *mut c_void as windef::HFONT 
}
lazy_static! {
	static ref HFONT: usize = unsafe { 
		let mut ncm: winuser::NONCLIENTMETRICSW = mem::zeroed();
		let size = mem::size_of::<winuser::NONCLIENTMETRICSW>() as u32;
		ncm.cbSize = size;
		if winuser::SystemParametersInfoW(winuser::SPI_GETNONCLIENTMETRICS, size, &mut ncm as *mut _ as *mut ::winapi::ctypes::c_void, size) == 0 {
			return 0;
		}
		let hfont = wingdi::CreateFontIndirectW(&mut ncm.lfMessageFont);
		if hfont.is_null() {
			log_error();
		}
		hfont as usize 
		// TODO cleanup!
	};
}

#[inline]
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
impl NativeId for Hwnd {}

#[repr(C)]
pub struct WindowsControlBase<T: controls::Control + Sized> {
	pub hwnd: windef::HWND,
    pub subclass_id: usize,
    pub coords: Option<(i32, i32)>,
    pub measured_size: (u16, u16),
    _marker: PhantomData<T>
}

impl <T: controls::Control + Sized> WindowsControlBase<T> {
    pub fn new() -> WindowsControlBase<T> {
        WindowsControlBase {
            hwnd: 0 as windef::HWND,
            subclass_id: 0,
            measured_size: (0, 0),
            coords: None,
            _marker: PhantomData
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
    pub fn parent(&self) -> Option<&MemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn parent_mut(&mut self) -> Option<&mut MemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetParent(self.hwnd);
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn root(&self) -> Option<&MemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetAncestor(self.hwnd, 2); //GA_ROOT
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn root_mut(&mut self) -> Option<&mut MemberBase> {
        unsafe {
            let parent_hwnd = winuser::GetAncestor(self.hwnd, 2); //GA_ROOT
            if parent_hwnd == self.hwnd {
                return None;
            }

            let parent_ptr = winuser::GetWindowLongPtrW(parent_hwnd, winuser::GWLP_USERDATA);
            mem::transmute(parent_ptr as *mut c_void)
        }
    }
    pub fn invalidate(&mut self) {
    	let parent_hwnd = self.parent_hwnd();	
		if let Some(parent_hwnd) = parent_hwnd {
			let mparent = member_base_from_hwnd(parent_hwnd);
			let (pw, ph) = mparent.as_member().size();
			let this = member_from_hwnd::<T>(self.hwnd);
			let (_,_,changed) = this.measure(pw, ph);
			
			if let Some(cparent) = mparent.as_member_mut().is_control_mut() {
				if changed && !cparent.is_skip_draw()  {
					cparent.invalidate();
				} else {
				    this.draw(None);
				    unsafe { winuser::RedrawWindow(self.hwnd, ptr::null_mut(), ptr::null_mut(), winuser::RDW_INVALIDATE | winuser::RDW_UPDATENOW); }
				}
			} else {
			    this.draw(None);
			    unsafe { winuser::RedrawWindow(self.hwnd, ptr::null_mut(), ptr::null_mut(), winuser::RDW_INVALIDATE | winuser::RDW_UPDATENOW); }
			}
	    }
    }
    pub fn draw(&mut self, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.coords = coords;
        }
        if let Some((x, y)) = self.coords {
            unsafe {
                winuser::SetWindowPos(
                    self.hwnd,
                    ptr::null_mut(),
                    x,
                    y,
                    self.measured_size.0 as i32,
                    self.measured_size.1 as i32,
                    0,
                );
            }
        }
    }
    pub fn size(&self) -> (u16, u16) {
        if self.hwnd.is_null() {
        	self.measured_size
        } else {
        	let rect = unsafe { window_rect(self.hwnd) };
	        ((rect.right - rect.left) as u16, (rect.bottom - rect.top) as u16)
        }
    }
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
    set_default_font(hwnd);
    (hwnd, subclass_id as usize)
}

#[inline]
pub unsafe fn set_default_font(hwnd: windef::HWND) {
	winuser::SendMessageW(hwnd, winuser::WM_SETFONT, hfont() as usize, minwindef::TRUE as isize);
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

#[inline]
pub unsafe fn window_rect(hwnd: windef::HWND) -> windef::RECT {
    let mut rect: windef::RECT = mem::zeroed();
    winuser::GetClientRect(hwnd, &mut rect);
    rect
}

#[inline]
unsafe fn cast_hwnd<'a, T>(hwnd: windef::HWND) -> &'a mut T
where T: Sized
{
    let hwnd_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    mem::transmute(hwnd_ptr as *mut c_void)
}
#[inline]
pub fn member_from_hwnd<'a, T>(hwnd: windef::HWND) -> &'a mut T where T: Sized + controls::Member {
    unsafe { cast_hwnd(hwnd) }
}
#[inline]
pub fn member_base_from_hwnd<'a>(hwnd: windef::HWND) -> &'a mut MemberBase {
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
