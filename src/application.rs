use super::common::*;
use super::*;

use plygui_api::controls;
use plygui_api::types;

use winapi::shared::windef;
use winapi::um::commctrl;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

const DEFAULT_FRAME_SLEEP_MS: u32 = 10;

pub struct WindowsApplication {
    pub(crate) root: windef::HWND,
    name: String,
    sleep: u32,
    windows: Vec<windef::HWND>,
    trays: Vec<*mut crate::tray::Tray>,
}

pub type Application = ::plygui_api::development::AApplication<WindowsApplication>;

impl ApplicationInner for WindowsApplication {
    fn get() -> Box<Application> {
        init_comctl();

        let mut a = Box::new(AApplication::with_inner(
            WindowsApplication {
                name: String::new(), //name.into(), // TODO later
                sleep: DEFAULT_FRAME_SLEEP_MS,
                windows: Vec::with_capacity(1),
                trays: Vec::with_capacity(0),
                root: 0 as windef::HWND,
            },
        ));

        let name = OsStr::new(a.inner().name.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
        let hwnd = unsafe {
            winuser::CreateWindowExW(
                0,
                WINDOW_CLASS.as_ptr(),
                name.as_ptr() as ntdef::LPCWSTR,
                0,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                1,
                1,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance(),
                a.as_mut() as *mut _ as *mut c_void,
            )
        };
        a.inner_mut().root = hwnd;
        a
    }
    fn new_window(&mut self, title: &str, size: types::WindowStartSize, menu: types::Menu) -> Box<dyn controls::Window> {
        let w = window::WindowsWindow::with_params(title, size, menu);
        unsafe {
            self.windows.push(w.native_id() as windef::HWND);
        }
        w
    }
    fn new_tray(&mut self, title: &str, menu: types::Menu) -> Box<dyn controls::Tray> {
        let mut tray = tray::WindowsTray::with_params(title, menu);
        self.trays.push(tray.as_any_mut().downcast_mut::<tray::Tray>().unwrap() as *mut crate::tray::Tray);
        tray
    }
    fn remove_window(&mut self, _: Self::Id) {
        // Better not to remove directly, as is breaks the wndproc loop.
    }
    fn remove_tray(&mut self, id: Self::Id) {
        let id = windef::HWND::from(id) as *mut tray::Tray;
        self.trays.retain(|t| *t != id);
    }
    fn name<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(self.name.as_str())
    }
    fn frame_sleep(&self) -> u32 {
        self.sleep
    }
    fn set_frame_sleep(&mut self, value: u32) {
        self.sleep = value;
    }    
    fn start(&mut self) {
        let mut msg: winuser::MSG = unsafe { mem::zeroed() };
        let mut i;
        loop {
            let mut frame_callbacks = 0;
            if let Some(w) = unsafe { cast_hwnd::<Application>(self.root) } {
                let w = w.base_mut();
                while !self.root.is_null() && frame_callbacks < defaults::MAX_FRAME_CALLBACKS {
                    match w.queue().try_recv() {
                        Ok(mut cmd) => {
                            if (cmd.as_mut())(unsafe { cast_hwnd::<Application>(self.root) }.unwrap()) {
                                let _ = w.sender().send(cmd);
                            }
                            frame_callbacks += 1;
                        }
                        Err(e) => match e {
                            mpsc::TryRecvError::Empty => break,
                            mpsc::TryRecvError::Disconnected => unreachable!(),
                        },
                    }
                }
            }
            unsafe {
                synchapi::Sleep(self.sleep);

                if winuser::PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, winuser::PM_REMOVE) > 0 {
                    winuser::TranslateMessage(&mut msg);
                    winuser::DispatchMessageW(&mut msg);
                }
            }

            i = 0;
            while i < self.windows.len() {
                if dispatch_window(self.windows[i]) < 0 {
                    self.windows.remove(i);
                } else {
                    i += 1;
                }
            }
            if self.windows.len() < 1 && self.trays.len() < 1 {
                unsafe {
                    winuser::DestroyWindow(self.root);
                }
                break;
            }
        }
    }
    fn find_member_mut(&mut self, arg: types::FindBy) -> Option<&mut dyn controls::Member> {
        for window in self.windows.as_mut_slice() {
            if let Some(window) = common::member_from_hwnd::<window::Window>(*window) {
                match arg {
                    types::FindBy::Id(id) => {
                        if window.id() == id {
                            return Some(window.as_member_mut());
                        }
                    }
                    types::FindBy::Tag(ref tag) => {
                        if let Some(mytag) = window.tag() {
                            if tag.as_str() == mytag {
                                return Some(window.as_member_mut());
                            }
                        }
                    }
                }
                let found = controls::Container::find_control_mut(window, arg.clone()).map(|control| control.as_member_mut());
                if found.is_some() {
                    return found;
                }
            }
        }
        for tray in self.trays.as_mut_slice() {
            let tray = unsafe { &mut **tray };
            match arg {
                types::FindBy::Id(ref id) => {
                    if tray.id() == *id {
                        return Some(tray.as_member_mut());
                    }
                }
                types::FindBy::Tag(ref tag) => {
                    if let Some(mytag) = tray.tag() {
                        if tag.as_str() == mytag {
                            return Some(tray.as_member_mut());
                        }
                    }
                }
            }
        }
        None
    }
    fn find_member(&self, arg: types::FindBy) -> Option<&dyn controls::Member> {
        for window in self.windows.as_slice() {
            if let Some(window) = common::member_from_hwnd::<window::Window>(*window) {
                match arg {
                    types::FindBy::Id(id) => {
                        if window.id() == id {
                            return Some(window.as_member());
                        }
                    }
                    types::FindBy::Tag(ref tag) => {
                        if let Some(mytag) = window.tag() {
                            if tag.as_str() == mytag {
                                return Some(window.as_member());
                            }
                        }
                    }
                }
                let found = controls::Container::find_control(window, arg.clone()).map(|control| control.as_member());
                if found.is_some() {
                    return found;
                }
            }
        }
        for tray in self.trays.as_slice() {
            let tray = unsafe { &mut **tray };
            match arg {
                types::FindBy::Id(ref id) => {
                    if tray.id() == *id {
                        return Some(tray.as_member());
                    }
                }
                types::FindBy::Tag(ref tag) => {
                    if let Some(mytag) = tray.tag() {
                        if tag.as_str() == mytag {
                            return Some(tray.as_member());
                        }
                    }
                }
            }
        }
        None
    }
    fn exit(&mut self, skip_on_close: bool) -> bool {
        for window in self.windows.as_mut_slice() {
            if !controls::Closeable::close(common::member_from_hwnd::<window::Window>(*window).unwrap(), skip_on_close) {
                return false;
            }
        }
        for tray in self.trays.as_mut_slice() {
            if !(controls::Closeable::close(unsafe { &mut **tray }, skip_on_close)) {
                return false;
            }
        }
        true
    }
    fn members<'a>(&'a self) -> Box<dyn Iterator<Item = &'a (dyn controls::Member)> + 'a> {
        Box::new(MemberIterator {
            inner: self,
            is_tray: false,
            index: 0,
            needs_window: true,
            needs_tray: true,
        })
    }
    fn members_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut (dyn controls::Member)> + 'a> {
        Box::new(MemberIteratorMut {
            inner: self,
            is_tray: false,
            index: 0,
            needs_window: true,
            needs_tray: true,
        })
    }
}

impl HasNativeIdInner for WindowsApplication {
    type Id = common::Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.root.into()
    }
}

impl Drop for WindowsApplication {
    fn drop(&mut self) {
        for w in self.windows.drain(..) {
            destroy_hwnd(w, 0, None);
        }
        for _ in self.trays.drain(..) {}
        destroy_hwnd(self.root, 0, None);
    }
}

struct MemberIterator<'a> {
    inner: &'a WindowsApplication,
    needs_window: bool,
    needs_tray: bool,
    is_tray: bool,
    index: usize,
}
impl<'a> Iterator for MemberIterator<'a> {
    type Item = &'a (dyn controls::Member + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.inner.windows.len() {
            self.is_tray = true;
            self.index = 0;
        }
        let ret = if self.needs_tray && self.is_tray {
            self.inner.trays.get(self.index).map(|tray| unsafe { &**tray } as &dyn controls::Member)
        } else if self.needs_window {
            self.inner.windows.get(self.index).map(|window| common::member_from_hwnd::<window::Window>(*window).unwrap() as &dyn controls::Member)
        } else {
            return None;
        };
        self.index += 1;
        ret
    }
}

struct MemberIteratorMut<'a> {
    inner: &'a mut WindowsApplication,
    needs_window: bool,
    needs_tray: bool,
    is_tray: bool,
    index: usize,
}
impl<'a> Iterator for MemberIteratorMut<'a> {
    type Item = &'a mut dyn (controls::Member);

    fn next(&mut self) -> Option<Self::Item> {
        if self.needs_tray && self.index >= self.inner.windows.len() {
            self.is_tray = true;
            self.index = 0;
        }
        let ret = if self.needs_tray && self.is_tray {
            self.inner.trays.get_mut(self.index).map(|tray| unsafe { &mut **tray } as &mut dyn controls::Member)
        } else if self.needs_window {
            self.inner.windows.get_mut(self.index).map(|window| common::member_from_hwnd::<window::Window>(*window).unwrap() as &mut dyn controls::Member)
        } else {
            return None;
        };
        self.index += 1;
        ret
    }
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32Application").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();

    let class = winuser::WNDCLASSEXW {
        cbSize: mem::size_of::<winuser::WNDCLASSEXW>() as minwindef::UINT,
        style: winuser::CS_DBLCLKS,
        lpfnWndProc: Some(handler),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: libloaderapi::GetModuleHandleW(ptr::null()),
        hIcon: ptr::null_mut(),
        hCursor: ptr::null_mut(),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: ptr::null_mut(),
    };
    winuser::RegisterClassExW(&class);
    class_name
}

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        if winuser::WM_CREATE == msg {
            let cs: &mut winuser::CREATESTRUCTW = mem::transmute(lparam);
            winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, cs.lpCreateParams as isize);
        }
        //return winuser::DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    match msg {
        winuser::WM_DESTROY => {
            winuser::PostQuitMessage(0);
        }
        winuser::WM_MENUSELECT => {
            //let flags = minwindef::HIWORD(wparam as u32);

            let w: &mut application::Application = mem::transmute(ww);
            let tray = if let Some(tray) = w.inner_mut().trays.iter().find(|tray| (&mut ***tray).inner_mut().inner_mut().is_menu_shown()) {
                &mut **tray
            } else {
                return 0;
            };

            if lparam == 0 {
                let w2: &mut application::Application = mem::transmute(ww);

                let tray2 = if let Some(tray) = w2.inner_mut().trays.iter().find(|tray| (&mut ***tray).inner_mut().inner_mut().is_menu_shown()) {
                    &mut **tray
                } else {
                    return 0;
                };
                tray.inner_mut().inner_mut().run_menu(tray2);
            } else {
                let item = minwindef::LOWORD(wparam as u32);
                tray.inner_mut().inner_mut().select_menu(item as usize);
            }
        }
        crate::tray::MESSAGE => {
            let evt = minwindef::LOWORD(lparam as u32);
            let id = minwindef::HIWORD(lparam as u32);

            let w: &mut application::Application = mem::transmute(ww);

            let tray = if let Some(tray) = w.inner_mut().trays.iter().find(|tray| (&***tray).id() == ids::Id::from_raw(id as usize)) {
                &mut **tray
            } else {
                return 0;
            };

            match evt as u32 {
                winuser::WM_CONTEXTMENU => {
                    tray.inner_mut().inner_mut().toggle_menu();
                }
                _ => {}
            }
        }
        _ => {}
    }
    return winuser::DefWindowProcW(hwnd, msg, wparam, lparam);
}

fn dispatch_window(hwnd: windef::HWND) -> i32 {
    if let Some(w) = common::member_from_hwnd::<window::Window>(hwnd) {
        w.inner_mut().inner_mut().inner_mut().inner_mut().dispatch()
    } else {
        -1
    }
}

fn init_comctl() {
    unsafe {
        let mut icc: commctrl::INITCOMMONCONTROLSEX = mem::zeroed();
        icc.dwSize = mem::size_of::<commctrl::INITCOMMONCONTROLSEX>() as u32;
        icc.dwICC = commctrl::ICC_STANDARD_CLASSES | commctrl::ICC_LISTVIEW_CLASSES | commctrl::ICC_TAB_CLASSES | commctrl::ICC_PROGRESS_CLASS | commctrl::ICC_UPDOWN_CLASS | commctrl::ICC_BAR_CLASSES;
        if commctrl::InitCommonControlsEx(&icc) == 0 {
            common::log_error();
        }
    }
}
