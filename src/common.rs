pub use plygui_api::sdk::*;
pub use plygui_api::{callbacks, controls, defaults, ids, layout, types::{self, adapter}, utils};
pub use plygui_api::external::*;

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
pub use winapi::um::synchapi;
pub use winapi::um::winbase;
pub use winapi::um::wingdi;
pub use winapi::um::winnls;
pub use winapi::um::winuser;

pub use std::borrow::Cow;
pub use std::ffi::{CStr, CString, IntoStringError, OsStr};
pub use std::marker::PhantomData;
pub use std::os::windows::ffi::OsStrExt;
pub use std::{cmp, mem, ops, ptr, str, sync::mpsc};

lazy_static! {
	pub static ref THEME_EXPLORER: Vec<u16> = OsStr::new("EXPLORER").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub const DEFAULT_PADDING: i32 = 6;
pub const DEFAULT_HEIGHT: i32 = 24;
pub const WM_UPDATE_INNER: u32 = winuser::WM_APP + 1;

#[cfg(not(target_pointer_width = "32"))]
pub type WinPtr = isize;
#[cfg(target_pointer_width = "32")]
pub type WinPtr = i32;

pub type WndHandler = unsafe extern "system" fn(hwnd: windef::HWND, msg: minwindef::UINT, minwindef::WPARAM, minwindef::LPARAM, usize, usize) -> isize;
pub type WndProc<O: controls::Control> = unsafe extern "system" fn(&mut O, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT;

pub enum WndProcHandler<O: controls::Control> {
    Proc(Option<WndProc<O>>),
    Handler(Option<WndHandler>)
}

impl<O: controls::Control> WndProcHandler<O> {
    pub fn as_proc(&self) -> Option<WndProc<O>> {
        match self {
            WndProcHandler::Proc(_proc) => *_proc,
            _ => None
        }
    }
    pub fn as_handler(&self) -> Option<WndHandler> {
        match self {
            WndProcHandler::Handler(handler) => *handler,
            _ => None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hfont(windef::HFONT);

impl From<windef::HFONT> for Hfont {
    #[inline]
    fn from(a: windef::HFONT) -> Self {
        Hfont(a)
    }
}
impl AsRef<windef::HFONT> for Hfont {
    #[inline]
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

#[inline]
pub fn hinstance() -> minwindef::HINSTANCE {
    *INSTANCE as *mut c_void as minwindef::HINSTANCE
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
    static ref INSTANCE: usize = unsafe { libloaderapi::GetModuleHandleW(ptr::null()) as usize };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hwnd(windef::HWND);

impl From<windef::HWND> for Hwnd {
    #[inline]
    fn from(a: windef::HWND) -> Hwnd {
        Hwnd(a)
    }
}
impl From<Hwnd> for windef::HWND {
    #[inline]
    fn from(a: Hwnd) -> windef::HWND {
        a.0
    }
}
impl From<Hwnd> for usize {
    #[inline]
    fn from(a: Hwnd) -> usize {
        a.0 as usize
    }
}
impl NativeId for Hwnd {
    unsafe fn from_outer(arg: usize) -> Self {
        Hwnd(arg as windef::HWND)
    }
}

#[repr(C)]
pub struct WindowsControlBase<T: controls::Control + Sized> {
    pub hwnd: windef::HWND,
    pub subclass_id: usize,
    pub proc_handler: WndProcHandler<T>,
}
/* hello 0119
pub trait HasWindowsControlBase {
    type T: controls::Control + Sized;
    
    fn base(&self) -> &WindowsControlBase<Self::T>;
    fn base_mut(&mut self) -> &mut WindowsControlBase<Self::T>;
}

impl <T: controls::Control + Sized, I: HasWindowsControlBase<T=T>> HasNativeIdInner for I {
    type Id = Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.base_mut().hwnd.into()
    }
}
*/
impl<T: controls::Control + Sized> WindowsControlBase<T> {
    fn with_wnd_handler(h: WndProcHandler<T>) -> Self {
        Self {
            hwnd: 0 as windef::HWND,
            subclass_id: 0,
            proc_handler: h,
        }
    }
    pub fn with_handler(handler: Option<WndHandler>) -> WindowsControlBase<T> {
        Self::with_wnd_handler(WndProcHandler::Handler(handler))
    }
    pub fn with_wndproc(wndproc: Option<WndProc<T>>) -> WindowsControlBase<T> {
        Self::with_wnd_handler(WndProcHandler::Proc(wndproc))
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
        let this = self.as_outer_mut();
        if this.is_skip_draw() {
            return;
        }
        /*let parent_hwnd = self.parent_hwnd();
        if let Some(parent_hwnd) = parent_hwnd {
            if let Some(mparent) = member_base_from_hwnd(parent_hwnd) {
                if let Some(cparent) = mparent.as_member_mut().is_control_mut() {
                    cparent.invalidate();
                } else {
                    this.draw(None);
                }
            }
        }*/
        //this.draw(None);
        unsafe {
           winuser::InvalidateRect(self.hwnd, ptr::null_mut(), minwindef::FALSE);
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
    pub fn create_control_hwnd(
        &mut self,
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
    ) {
        let (hwnd, subclass_id) = unsafe { create_control_hwnd(x, y, w, h, parent, ex_style, class_name, control_name, style, param, self.proc_handler.as_handler()) };
        self.hwnd = hwnd; 
        self.subclass_id = subclass_id;
    }
    pub fn destroy_control_hwnd(&mut self) {
        match self.proc_handler {
            WndProcHandler::Handler(h) => {
                destroy_hwnd(self.hwnd, self.subclass_id, h)
            }
            _ => {}
        }
        self.hwnd = 0 as windef::HWND;
        self.subclass_id = 0;
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
    commctrl::SetWindowSubclass(hwnd, handler, subclass_id(class_name) as usize, param as usize);
    set_default_font(hwnd);
    (hwnd, subclass_id as usize)
}

pub fn subclass_id(class_name: ntdef::LPCWSTR) -> u64 {
	use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    let mut hasher = DefaultHasher::new();
    hasher.write_usize(class_name as usize);
    hasher.finish()
}    

pub fn str_to_wchar<S: AsRef<str>>(a: S) -> Vec<u16> {
    OsStr::new(a.as_ref()).encode_wide().chain(Some(0).into_iter()).collect()
}
pub unsafe fn wchar_to_str(p: *const u16) -> String {
    let len = (0..).take_while(|&i| *p.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(p, len);
    String::from_utf16_lossy(slice)
}

#[inline]
pub unsafe fn set_default_font(hwnd: windef::HWND) {
    winuser::SendMessageW(hwnd, winuser::WM_SETFONT, hfont() as usize, minwindef::TRUE as isize);
}

pub fn destroy_hwnd(hwnd: windef::HWND, subclass_id: usize, handler: Option<unsafe extern "system" fn(windef::HWND, msg: minwindef::UINT, minwindef::WPARAM, minwindef::LPARAM, usize, usize) -> isize>) {
    unsafe {
        if subclass_id != 0 {
            if minwindef::FALSE == commctrl::RemoveWindowSubclass(hwnd, handler, subclass_id) {
                log_error();
            }
        }
        if winuser::DestroyWindow(hwnd) == 0 && winuser::IsWindow(hwnd) != 0 {
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

pub fn make_menu(menu: windef::HMENU, mut items: Vec<types::MenuItem>, storage: &mut Vec<callbacks::Action>) {
    let mut options = Vec::new();
    let mut help = Vec::new();

    let append_item = |menu, label, action, storage: &mut Vec<callbacks::Action>| {
        let wlabel = str_to_wchar(label);
        let id = storage.len();
        storage.push(action);
        unsafe { winuser::AppendMenuW(menu, winuser::MF_STRING, id, wlabel.as_ptr()); }
    };
    let append_level = |menu, label, items, storage: &mut Vec<callbacks::Action>| {
        let wlabel = str_to_wchar(label);
        let submenu = unsafe { winuser::CreateMenu() };
        make_menu(submenu, items, storage);
        unsafe { winuser::AppendMenuW(menu, winuser::MF_POPUP, submenu as usize, wlabel.as_ptr()); }
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
                    unsafe { winuser::AppendMenuW(menu, winuser::MF_SEPARATOR, 0, ptr::null_mut()); }
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
                unsafe { winuser::AppendMenuW(menu, winuser::MF_SEPARATOR, 0, ptr::null_mut()); }
            }
        }
    }

    make_special(menu, options, storage);
    make_special(menu, help, storage);
}

pub unsafe fn native_to_image(src: windef::HBITMAP) -> image::DynamicImage {
    let (bm, raw) = {
        let mut bm =  mem::MaybeUninit::<wingdi::BITMAP>::uninit();
        wingdi::GetObjectW(src as *mut c_void, mem::size_of::<wingdi::BITMAP>() as i32, bm.as_mut_ptr() as *mut c_void);
        let bm = bm.assume_init();
        
        let mut bminfo = wingdi::BITMAPINFO {
            bmiHeader: wingdi::BITMAPINFOHEADER {
                biSize: mem::size_of::<wingdi::BITMAPINFOHEADER>() as u32,
                biWidth: bm.bmWidth,
                biHeight: bm.bmHeight,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: wingdi::BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: mem::zeroed(),
        };  
    
        let mut raw = vec![0u8; bm.bmWidth as usize * bm.bmHeight as usize * 4];
        let hdc_screen = winuser::GetDC(ptr::null_mut());
        let ret = wingdi::GetDIBits(hdc_screen, src, 0, bm.bmHeight as u32, raw.as_mut_ptr() as *mut c_void, &mut bminfo, wingdi::DIB_RGB_COLORS);
        if ret == 0 {
            log_error();
        }
        
        winuser::ReleaseDC(ptr::null_mut(), hdc_screen);
        
        (bm, raw)
    };
    let img = image::RgbImage::from_raw(bm.bmWidth as u32, bm.bmHeight as u32, raw).unwrap();
    image::DynamicImage::ImageRgb8(img).flipv()
}

pub unsafe fn image_to_native(src: &image::DynamicImage, dst: *mut windef::HBITMAP) {
    use image::GenericImageView;

    let (w, h) = src.dimensions();

    let bminfo = wingdi::BITMAPINFO {
        bmiHeader: wingdi::BITMAPINFOHEADER {
            biSize: mem::size_of::<wingdi::BITMAPINFOHEADER>() as u32,
            biWidth: w as i32,
            biHeight: h as i32,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: wingdi::BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: mem::zeroed(),
    };

    let mut pv_image_bits = ptr::null_mut();
    let hdc_screen = winuser::GetDC(ptr::null_mut());
    *dst = wingdi::CreateDIBSection(hdc_screen, &bminfo, wingdi::DIB_RGB_COLORS, &mut pv_image_bits, ptr::null_mut(), 0);
    winuser::ReleaseDC(ptr::null_mut(), hdc_screen);
    if (*dst).is_null() {
        panic!("Could not load image.")
    }

    ptr::copy(src.flipv().to_rgba().into_raw().as_ptr(), pv_image_bits as *mut u8, (w * h * 4) as usize);
}

pub unsafe fn str_from_wide<'a>(wstring: *mut u16) -> Cow<'a, str> {
	use std::slice;
	if wstring.is_null() {
		Cow::Owned(String::new())
	} else {
		let mut curr = wstring as usize;
		let mut len = 0;
		while *(curr as *const u16) != 0 {
			len += 1;
			curr += 2;
		}
		Cow::Owned(String::from_utf16_lossy(slice::from_raw_parts(wstring as *const u16, len)))
		//Cow::Borrowed(CStr::from_ptr(wstring).to_str().expect("Cannot parse CStr"))
	}
}

pub fn string_of_pixel_len(len: usize) -> String {
    vec!['_'; len / 5].iter().collect::<String>()
}

//TODO density!
pub fn wsz_of_pixel_len(len: usize) -> Vec<u16> {
    OsStr::new(string_of_pixel_len(len).as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
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
