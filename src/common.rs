pub use plygui_api::development::*;
pub use plygui_api::{callbacks, controls, defaults, ids, layout, types, utils};

pub use winapi::ctypes::c_void;
pub use winapi::shared::basetsd;
pub use winapi::shared::minwindef;
pub use winapi::shared::ntdef;
pub use winapi::shared::windef;
pub use winapi::shared::winerror;
pub use winapi::um::commctrl;
pub use winapi::um::errhandlingapi;
pub use winapi::um::libloaderapi;
pub use winapi::um::stringapiset;
pub use winapi::um::winbase;
pub use winapi::um::wingdi;
pub use winapi::um::winnls;
pub use winapi::um::winuser;

pub use std::borrow::Cow;
pub use std::ffi::{CString, IntoStringError, OsStr};
pub use std::marker::PhantomData;
pub use std::os::windows::ffi::OsStrExt;
pub use std::{cmp, mem, ops, ptr, str, sync::mpsc};

pub const DEFAULT_PADDING: i32 = 6;
pub const WM_UPDATE_INNER: u32 = winuser::WM_APP + 1;

pub type WndHandler = unsafe extern "system" fn(windef::HWND, msg: minwindef::UINT, minwindef::WPARAM, minwindef::LPARAM, usize, usize) -> isize;
pub type WndProc = unsafe extern "system" fn(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT;

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
fn hfont() -> windef::HFONT {
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
    _marker: PhantomData<T>,
}

impl<T: controls::Control + Sized> WindowsControlBase<T> {
    pub fn new() -> WindowsControlBase<T> {
        WindowsControlBase {
            hwnd: 0 as windef::HWND,
            subclass_id: 0,
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
        member_from_hwnd::<T>(self.hwnd).unwrap()
    }
    pub fn as_outer_mut(&self) -> &mut T {
        member_from_hwnd::<T>(self.hwnd).unwrap()
    }
    pub fn invalidate(&mut self) {
        if self.hwnd.is_null() {
            return;
        }
        let parent_hwnd = self.parent_hwnd();
        let this = self.as_outer_mut();
        if this.is_skip_draw() {
            return;
        }
        if let Some(parent_hwnd) = parent_hwnd {
            if let Some(mparent) = member_base_from_hwnd(parent_hwnd) {
                let (pw, ph) = mparent.as_member().is_has_size().unwrap().size();
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
    }
    pub fn draw(&mut self, coords: Option<(i32, i32)>, (width, height): (u16, u16)) -> bool {
        draw(self.hwnd, coords, (width, height))
    }
    pub fn on_set_visibility(&mut self, visibility: types::Visibility) -> bool {
        if !self.hwnd.is_null() {
            unsafe {
                winuser::ShowWindow(self.hwnd, if visibility == types::Visibility::Visible { winuser::SW_SHOW } else { winuser::SW_HIDE });
            }
            self.invalidate();
            true
        } else {
            false
        }
    }
}

pub fn size_hwnd(hwnd: windef::HWND) -> (u16, u16) {
    let rect = unsafe { window_rect(hwnd) };
    ((rect.right - rect.left) as u16, (rect.bottom - rect.top) as u16)
}
pub fn pos_hwnd(hwnd: windef::HWND) -> (i32, i32) {
    let rect = unsafe { window_rect(hwnd) };
    (rect.left as i32, rect.top as i32)
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
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        hasher.write_usize(class_name as usize);
        hasher.finish()
    };
    let os_control_name = OsStr::new(control_name).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
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

pub fn str_to_wchar<S: AsRef<str>>(a: S) -> Vec<u16> {
    OsStr::new(a.as_ref()).encode_wide().chain(Some(0).into_iter()).collect()
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
pub fn draw(hwnd: windef::HWND, coords: Option<(i32, i32)>, (width, height): (u16, u16)) -> bool {
    if let Some((x, y)) = coords {
        unsafe {
            winuser::SetWindowPos(hwnd, ptr::null_mut(), x, y, width as i32, height as i32, 0);
        }
        true
    } else {
        false
    }
}

#[inline]
pub unsafe fn window_rect(hwnd: windef::HWND) -> windef::RECT {
    let mut rect: windef::RECT = mem::zeroed();
    winuser::GetClientRect(hwnd, &mut rect);
    rect
}

#[inline]
pub(crate) unsafe fn cast_hwnd<'a, T>(hwnd: windef::HWND) -> Option<&'a mut T>
where
    T: Sized,
{
    let hwnd_ptr = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if hwnd_ptr == 0 {
        None
    } else {
        Some(mem::transmute(hwnd_ptr as *mut c_void))
    }
}
#[inline]
pub fn member_from_hwnd<'a, T>(hwnd: windef::HWND) -> Option<&'a mut T>
where
    T: Sized + controls::Member,
{
    unsafe { cast_hwnd(hwnd) }
}
#[inline]
pub fn member_base_from_hwnd<'a>(hwnd: windef::HWND) -> Option<&'a mut MemberBase> {
    unsafe { cast_hwnd(hwnd) }
}

pub unsafe fn make_menu(menu: windef::HMENU, mut items: Vec<types::MenuItem>, storage: &mut Vec<callbacks::Action>) {
    let mut options = Vec::new();
    let mut help = Vec::new();

    let append_item = |menu, label, action, storage: &mut Vec<callbacks::Action>| {
        let wlabel = str_to_wchar(label);
        let id = storage.len();
        storage.push(action);
        winuser::AppendMenuW(menu, winuser::MF_STRING, id, wlabel.as_ptr());
    };
    let append_level = |menu, label, items, storage: &mut Vec<callbacks::Action>| {
        let wlabel = str_to_wchar(label);
        let submenu = winuser::CreateMenu();
        make_menu(submenu, items, storage);
        winuser::AppendMenuW(menu, winuser::MF_POPUP, submenu as usize, wlabel.as_ptr());
    };
    let make_special = |menu, mut special: Vec<types::MenuItem>, storage: &mut Vec<callbacks::Action>| {
        for item in special.drain(..) {
            match item {
                types::MenuItem::Action(label, action, _) => {
                    append_item(menu, label, action, storage);
                }
                types::MenuItem::Sub(label, items, _) => {
                    append_level(menu, label, items, storage);
                }
                types::MenuItem::Delimiter => {
                    winuser::AppendMenuW(menu, winuser::MF_SEPARATOR, 0, ptr::null_mut());
                }
            }
        }
    };

    for item in items.drain(..) {
        match item {
            types::MenuItem::Action(label, action, role) => match role {
                types::MenuItemRole::None => {
                    append_item(menu, label, action, storage);
                }
                types::MenuItemRole::Options => {
                    options.push(types::MenuItem::Action(label, action, role));
                }
                types::MenuItemRole::Help => {
                    help.push(types::MenuItem::Action(label, action, role));
                }
            },
            types::MenuItem::Sub(label, items, role) => match role {
                types::MenuItemRole::None => {
                    append_level(menu, label, items, storage);
                }
                types::MenuItemRole::Options => {
                    options.push(types::MenuItem::Sub(label, items, role));
                }
                types::MenuItemRole::Help => {
                    help.push(types::MenuItem::Sub(label, items, role));
                }
            },
            types::MenuItem::Delimiter => {
                winuser::AppendMenuW(menu, winuser::MF_SEPARATOR, 0, ptr::null_mut());
            }
        }
    }

    make_special(menu, options, storage);
    make_special(menu, help, storage);
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

    println!("Last error #{}: {}", error, String::from_utf16_lossy(&string));
}
