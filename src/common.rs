pub use plygui_api::development::*;
pub use plygui_api::{callbacks, controls, defaults, ids, layout, types, utils};

pub use winapi::ctypes::c_void;
pub use winapi::shared::minwindef;
pub use winapi::shared::ntdef;
pub use winapi::shared::windef;
pub use winapi::shared::winerror;
pub use winapi::um::commctrl;
pub use winapi::um::errhandlingapi;
pub use winapi::um::libloaderapi;
pub use winapi::um::winbase;
pub use winapi::um::wingdi;
pub use winapi::um::winuser;
#[cfg(feature = "prettier")]
pub use winapi::um::{dwmapi, uxtheme};

pub use std::borrow::Cow;
pub use std::ffi::OsStr;
pub use std::marker::PhantomData;
pub use std::os::windows::ffi::OsStrExt;
pub use std::{cmp, mem, ptr, str, ops, sync::mpsc};

pub const DEFAULT_PADDING: i32 = 6;
pub const WM_UPDATE_INNER: u32 = winuser::WM_APP + 1;

pub type WndHandler = unsafe extern "system" fn(windef::HWND, msg: minwindef::UINT, minwindef::WPARAM, minwindef::LPARAM, usize, usize) -> isize;
pub type WndProc = unsafe extern "system" fn (hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT;

#[derive(Debug, Clone)]
pub struct Hfont(windef::HFONT);

impl From<windef::HFONT> for Hfont {
	fn from(a: windef::HFONT) -> Self {
		Hfont(a)
	}
}
impl AsRef<windef::HFONT> for Hfont {
	fn as_ref(&self) -> &windef::HFONT {
		&self.0
	}
}
impl Drop for Hfont {
	fn drop(&mut self) {
		unsafe {
			if wingdi::DeleteObject(self.0 as *mut c_void) == minwindef::FALSE {
				log_error();
				panic!("Could not delete HFONT {:?}", self.0);
			}
		}
	}
}
unsafe impl Sync for Hfont {}

#[inline]
pub fn hfont() -> windef::HFONT {
    *(*HFONT).as_ref()
}
lazy_static! {
    static ref HFONT: Hfont = unsafe {
        let mut ncm: winuser::NONCLIENTMETRICSW = mem::zeroed();
        let size = mem::size_of::<winuser::NONCLIENTMETRICSW>() as u32;
        ncm.cbSize = size;
        if winuser::SystemParametersInfoW(winuser::SPI_GETNONCLIENTMETRICS, size, &mut ncm as *mut _ as *mut ::winapi::ctypes::c_void, size) == 0 {
            panic!("Cannot get NonClientMetrics for Font");
        }
        let hfont = wingdi::CreateFontIndirectW(&mut ncm.lfMessageFont);
        if hfont.is_null() {
            log_error();
        }
        hfont.into()
    };
}

#[inline]
pub fn hinstance() -> minwindef::HINSTANCE {
    *INSTANCE as *mut c_void as minwindef::HINSTANCE
}
lazy_static! {
    static ref INSTANCE: usize = unsafe { libloaderapi::GetModuleHandleW(ptr::null()) as usize };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    _marker: PhantomData<T>,
}

impl<T: controls::Control + Sized> WindowsControlBase<T> {
    pub fn new() -> WindowsControlBase<T> {
        WindowsControlBase {
            hwnd: 0 as windef::HWND,
            subclass_id: 0,
            measured_size: (0, 0),
            coords: None,
            _marker: PhantomData,
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
    pub fn as_outer(&self) -> &T {
        member_from_hwnd::<T>(self.hwnd)
    }
    pub fn as_outer_mut(&self) -> &mut T {
        member_from_hwnd::<T>(self.hwnd)
    }
    pub fn invalidate(&mut self) {
        let parent_hwnd = self.parent_hwnd();
        let this = self.as_outer_mut();
        if this.is_skip_draw() {
           return; 
        }
        if let Some(parent_hwnd) = parent_hwnd {
            let mparent = member_base_from_hwnd(parent_hwnd);
            let (pw, ph) = mparent.as_member().size();
            let (_, _, changed) = this.measure(pw, ph);

            if let Some(cparent) = mparent.as_member_mut().is_control_mut() {
                if changed && !cparent.is_skip_draw() {
                    cparent.invalidate();
                }
            } else {
                this.draw(None);
            }
        }
    }
    pub fn draw(&mut self, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.coords = coords;
        }
        if let Some((x, y)) = self.coords {
            unsafe {
                winuser::SetWindowPos(self.hwnd, ptr::null_mut(), x, y, self.measured_size.0 as i32, self.measured_size.1 as i32, 0);
            }
        }
    }
    pub fn size(&self) -> (u16, u16) {
        if self.hwnd.is_null() {
            self.measured_size
        } else {
            let rect = window_rect(self.hwnd);
            match rect {
                Ok(rect) => ((rect.right - rect.left) as u16, (rect.bottom - rect.top) as u16),
                Err(_) => self.measured_size
            }
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
    handler: Option<WndHandler>,
) -> (windef::HWND, usize) {
    let mut style = style;
    if (style & winuser::WS_TABSTOP) != 0 {
        style |= winuser::WS_GROUP;
    }
    let subclass_id = {
        use std::hash::Hasher;
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        hasher.write_usize(class_name as usize);
        hasher.finish()
    };
    let os_control_name = OsStr::new(control_name).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    let hwnd = winuser::CreateWindowExW(
        ex_style | winuser::WS_EX_NOPARENTNOTIFY,
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

pub fn destroy_hwnd(hwnd: windef::HWND, subclass_id: usize, handler: Option<unsafe extern "system" fn(windef::HWND, msg: minwindef::UINT, minwindef::WPARAM, minwindef::LPARAM, usize, usize) -> isize>) {
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
pub fn window_rect(hwnd: windef::HWND) -> Result<windef::RECT, ()> {
    let mut rect: windef::RECT = unsafe { mem::zeroed() };
    if unsafe { winuser::GetClientRect(hwnd, &mut rect) } >= 0 {
        Ok(rect)
    } else {
         Err(())
    }
}

#[inline]
unsafe fn cast_hwnd<'a, T>(hwnd: windef::HWND) -> &'a mut T
where
    T: Sized,
{
    let hwnd_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    mem::transmute(hwnd_ptr as *mut c_void)
}
#[inline]
pub fn member_from_hwnd<'a, T>(hwnd: windef::HWND) -> &'a mut T
where
    T: Sized + controls::Member,
{
    unsafe { cast_hwnd(hwnd) }
}
#[inline]
pub fn member_base_from_hwnd<'a>(hwnd: windef::HWND) -> &'a mut MemberBase {
    unsafe { cast_hwnd(hwnd) }
}

#[inline]
pub unsafe fn log_error() {
    log_error_value(errhandlingapi::GetLastError())
}

#[cfg(not(debug_assertions))]
pub unsafe fn log_error_value(error: u32) {}

#[cfg(debug_assertions)]
pub unsafe fn log_error_value(error: u32) {
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

    println!("Last error #{}: {}", error, String::from_utf16_lossy(&string));
}

#[cfg(feature = "prettier")]
pub mod aero {
    pub use super::*;
    
    const DEFAULT_GLOW_SIZE: i32 = 12;
    
    lazy_static! {
        static ref WINDOW_CLASS_COMPOSITED: Vec<u16> = OsStr::new("CompositedWindow::Window").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
        static ref WINDOW_CLASS_EDIT: Vec<u16> = OsStr::new("Edit").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    }
    
    pub unsafe fn prettify(hwnd: windef::HWND) -> Result<(),()> {
        let mut ps: winuser::PAINTSTRUCT = mem::zeroed();
        let hdc = winuser::BeginPaint(hwnd, &mut ps);
        let margins = uxtheme::MARGINS { cxLeftWidth: -1, cxRightWidth: -1, cyBottomHeight: -1, cyTopHeight: -1}; 
        let mut enabled = minwindef::FALSE;
        if !hdc.is_null() && winerror::SUCCEEDED(dwmapi::DwmIsCompositionEnabled(&mut enabled)) {
            let error = dwmapi::DwmExtendFrameIntoClientArea(hwnd, &margins);
            if winerror::SUCCEEDED(error) {
                let rclient = window_rect(hwnd)?;
                wingdi::PatBlt(hdc, 0, 0, rclient.right - rclient.left, rclient.bottom - rclient.top, wingdi::BLACKNESS);
            } else {
                log_error_value(error as u32);
            }
        } else {
            log_error();
        }
        winuser::EndPaint(hwnd, &mut ps);
        Ok(())
    }
    pub fn state_from_button_state(style: u32, hot: bool, focus: bool, check_state: usize, part_id: i32, has_mouse_capture: bool) -> i32 {
        let mut state;
        match part_id {
            1 => { // BP_PUSHBUTTON:
                state = 1; //PBS_NORMAL;
                if (style & winuser::WS_DISABLED) != 0 {
                    state = 4; //PBS_DISABLED;
                } else {
                    if (style & winuser::BS_DEFPUSHBUTTON) != 0 {
                        state = 5; //PBS_DEFAULTED;
                    }
                    if has_mouse_capture && hot {
                        state = 3; //PBS_PRESSED;
                    } else if has_mouse_capture || hot {
                        state = 2; //PBS_HOT;
                    }
                }
            }
            4 => { // BP_GROUPBOX:
                state = if (style & winuser::WS_DISABLED) != 0 { 1 /*GBS_DISABLED*/ } else { 2 /*GBS_NORMAL*/ };
            }
            2 => { // BP_RADIOBUTTON:
                match check_state {
                    winuser::BST_CHECKED => {
                        if (style & winuser::WS_DISABLED) != 0 {
                            state = 8; //RBS_CHECKEDDISABLED;
                        } else if focus {
                            state = 7; //RBS_CHECKEDPRESSED;
                        } else if hot {
                            state = 6; //RBS_CHECKEDHOT;
                        } else {
                            state = 5; //RBS_CHECKEDNORMAL;       
                        }
                    }
                    winuser::BST_UNCHECKED => {
                        if (style & winuser::WS_DISABLED) != 0 {
                            state = 4; //RBS_UNCHECKEDDISABLED;
                        } else if focus {
                            state = 3; //RBS_UNCHECKEDPRESSED;
                        } else if hot {
                            state = 2; //RBS_UNCHECKEDHOT;
                        } else { 
                            state = 1; //RBS_UNCHECKEDNORMAL;       
                        }
                    }
                    _ => unreachable!(),
                }
            }
            3 => { // BP_CHECKBOX:
                match check_state {
                    winuser::BST_CHECKED => {
                        if (style & winuser::WS_DISABLED) != 0 {
                            state = 8; //CBS_CHECKEDDISABLED;
                        } else if focus {
                            state = 7; //CBS_CHECKEDPRESSED;
                        } else if hot {
                            state = 6; //CBS_CHECKEDHOT;
                        } else {
                            state = 5; //CBS_CHECKEDNORMAL; 
                        }      
                    }
                    winuser::BST_INDETERMINATE => {
                        if (style & winuser::WS_DISABLED) != 0 {
                            state = 12; //CBS_MIXEDDISABLED;
                        } else if focus {
                            state = 11; //CBS_MIXEDPRESSED;
                        } else if hot {
                            state = 10; //CBS_MIXEDHOT;
                        } else { 
                            state = 9; //CBS_MIXEDNORMAL;
                        }       
                    }
                    winuser::BST_UNCHECKED => {
                        if (style & winuser::WS_DISABLED) != 0 {
                            state = 4; //CBS_UNCHECKEDDISABLED;
                        } else if focus {
                            state = 3; //CBS_UNCHECKEDPRESSED;
                        } else if hot {
                            state = 2; //CBS_UNCHECKEDHOT;
                        } else { 
                            state = 1; //CBS_UNCHECKEDNORMAL;
                        }       
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!()
        }
    
        state
    }
    
    pub fn glow_size(mut class_id: *const u16) -> Result<i32,()> {
        if class_id.is_null() {
            class_id = WINDOW_CLASS_COMPOSITED.as_ptr();
        }
        let theme = unsafe { uxtheme::OpenThemeData(ptr::null_mut(), class_id) };
        if theme.is_null() { return Err(()); }
        
        let mut size = DEFAULT_GLOW_SIZE;
        
        unsafe { 
            winerror::SUCCEEDED(uxtheme::GetThemeInt(theme, 0, 0, 2425 /*TMT_TEXTGLOWSIZE*/, &mut size)); 
            winerror::SUCCEEDED(uxtheme::CloseThemeData(theme));
        }    
        Ok(size)
    }
    
    pub fn edit_border_color(hwnd: windef::HWND) -> Result<windef::COLORREF, ()> {
        let theme = unsafe { uxtheme::OpenThemeData(hwnd, WINDOW_CLASS_EDIT.as_ptr()) };
        if theme.is_null() { return Err(()); }
        
        let mut color: windef::COLORREF = wingdi::RGB(0, 0, 0);
    
        unsafe { 
            winerror::SUCCEEDED(uxtheme::GetThemeColor(theme, 5 /*EP_BACKGROUNDWITHBORDER*/, 1 /*EBWBS_NORMAL*/, 3801 /*TMT_BORDERCOLOR*/, &mut color)); 
            winerror::SUCCEEDED(uxtheme::CloseThemeData(theme));
        }    
        Ok(color)
    }
    
    pub unsafe fn aerize(hwnd: windef::HWND, /*hdc: windef::HDC, rect: &mut windef::RECT,*/ draw_border: bool) -> Result<(),()> {
        let mut ps: winuser::PAINTSTRUCT = mem::zeroed();
        let hdc = winuser::BeginPaint(hwnd, &mut ps);
        if hdc.is_null() { return Err(()); }
        
        let mut hdc_paint = ptr::null_mut();
        if draw_border {
            if winuser::InflateRect(&mut ps.rcPaint, 1, 1) < 0 { return Err(()) }
        }
        let buff_paint = uxtheme::BeginBufferedPaint(hdc, &mut ps.rcPaint, uxtheme::BPBF_TOPDOWNDIB, ptr::null_mut(), &mut hdc_paint);
        if hdc_paint.is_null() {
            log_error();
            return Err(());
        }
    
        let mut wrect = window_rect(hwnd)?;
        
        if wingdi::PatBlt(hdc_paint, 0, 0, wrect.right - wrect.left, wrect.bottom - wrect.top, wingdi::BLACKNESS) < 0 {
            log_error();
            return Err(());
        }
        if uxtheme::BufferedPaintSetAlpha(buff_paint, &mut wrect, 0) != winerror::S_OK {
            log_error();
            return Err(());
        }
        
        if draw_border {
            if winuser::InflateRect(&mut ps.rcPaint, -1, -1) < 0  {
                log_error();
                return Err(());
            }
        }
        // Tell the control to paint itself in our memory buffer
        winuser::SendMessageW(hwnd, winuser::WM_PRINTCLIENT, hdc_paint as usize, (winuser::PRF_CLIENT|winuser::PRF_ERASEBKGND |winuser::PRF_NONCLIENT|winuser::PRF_CHECKVISIBLE) as isize);
        
        if draw_border {
            if winuser::InflateRect(&mut ps.rcPaint, 1, 1) < 0 { return Err(()) }
            if winuser::FrameRect(hdc_paint, &mut ps.rcPaint, wingdi::GetStockObject(wingdi::BLACK_BRUSH as i32) as windef::HBRUSH) < 0  {
                log_error();
                return Err(());
            }
        }

        uxtheme::EndBufferedPaint(buff_paint, minwindef::TRUE);
        
        Ok(())
    }
}
