use super::common::*;
use super::*;

use plygui_api::controls;
use plygui_api::types;

use winapi::shared::windef;
use winapi::um::commctrl;

use std::any::TypeId;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

const DEFAULT_FRAME_SLEEP_MS: u32 = 10;

pub struct WindowsApplication {
    pub(crate) root: windef::HWND,
    sleep: u32,
}

pub type Application = AApplication<WindowsApplication>;

impl<O: controls::Application> NewApplicationInner<O> for WindowsApplication {
    fn with_uninit_params(u: &mut mem::MaybeUninit<O>, name: &str) -> Self {
        init_comctl();
        let osname = OsStr::new(name).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
        let hwnd = unsafe {
            winuser::CreateWindowExW(
                0,
                WINDOW_CLASS.as_ptr(),
                osname.as_ptr() as ntdef::LPCWSTR,
                0,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                1,
                1,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance(),
                u as *mut _ as *mut c_void,
            )
        };
        WindowsApplication {
            sleep: DEFAULT_FRAME_SLEEP_MS,
            root: hwnd,
        }
    }
}

impl ApplicationInner for WindowsApplication {
    fn with_name<S: AsRef<str>>(name: S) -> Box<dyn controls::Application> {
        let mut b: Box<mem::MaybeUninit<Application>> = Box::new_uninit();
        let ab = AApplication::with_inner(
            <Self as NewApplicationInner<Application>>::with_uninit_params(b.as_mut(), name.as_ref()),
        );
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
    fn name(&self) -> Cow<str> {
        if self.root != 0 as windef::HWND {
            let mut wbuffer = vec![0u16; 4096];
            let len = unsafe { winuser::GetWindowTextW(self.root, wbuffer.as_mut_slice().as_mut_ptr(), 4096) };
            Cow::Owned(String::from_utf16_lossy(&wbuffer.as_slice()[..len as usize]))
        } else {
            unreachable!();
        }
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
                let w = &mut w.base;
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
                unsafe {
                    synchapi::Sleep(self.sleep);
    
                    if winuser::PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, winuser::PM_REMOVE) > 0 {
                        winuser::TranslateMessage(&mut msg);
                        winuser::DispatchMessageW(&mut msg);
                    }
                }
    
                i = 0;
                while i < w.windows.len() {
                    if dispatch_window(w.windows[i].as_any_mut().downcast_mut::<crate::window::Window>().unwrap().inner_mut().inner_mut().inner_mut().inner_mut().native_id().into()) < 0 {
                        w.windows.remove(i);
                    } else {
                        i += 1;
                    }
                }
                if w.windows.len() < 1 && w.trays.len() < 1 {
                    unsafe {
                        winuser::DestroyWindow(self.root);
                    }
                    break;
                }
            }
        }
    }
    fn find_member_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn Member> {
    	if let Some(w) = unsafe { cast_hwnd::<Application>(self.root) } {
            let w = &mut w.base;
            for window in w.windows.as_mut_slice() {
                match arg {
                    types::FindBy::Id(id) => {
                        if window.id() == id {
                            return Some(window.as_member_mut());
                        }
                    }
                    types::FindBy::Tag(tag) => {
                        if let Some(mytag) = window.tag() {
                            if tag == mytag {
                                return Some(window.as_member_mut());
                            }
                        }
                    }
                }
                let found = controls::Container::find_control_mut(window.as_mut(), arg.clone()).map(|control| control.as_member_mut());
                if found.is_some() {
                    return found;
                }
            }
            for tray in w.trays.as_mut_slice() {
                let tray = &mut **tray;
                match arg {
                    types::FindBy::Id(ref id) => {
                        if tray.id() == *id {
                            return Some(tray.as_member_mut());
                        }
                    }
                    types::FindBy::Tag(tag) => {
                        if let Some(mytag) = tray.tag() {
                            if tag == mytag {
                                return Some(tray.as_member_mut());
                            }
                        }
                    }
                }
            }
        }
        None
    }
    fn find_member<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn Member> {
        if let Some(w) = unsafe { cast_hwnd::<Application>(self.root) } {
            let w = &w.base;
            for window in w.windows.as_slice() {
                match arg {
                    types::FindBy::Id(id) => {
                        if window.id() == id {
                            return Some(window.as_member());
                        }
                    }
                    types::FindBy::Tag(tag) => {
                        if let Some(mytag) = window.tag() {
                            if tag == mytag {
                                return Some(window.as_member());
                            }
                        }
                    }
                }
                let found = controls::Container::find_control(window.as_ref(), arg.clone()).map(|control| control.as_member());
                if found.is_some() {
                    return found;
                }
            }
            for tray in w.trays.as_slice() {
                match arg {
                    types::FindBy::Id(ref id) => {
                        if tray.id() == *id {
                            return Some(tray.as_member());
                        }
                    }
                    types::FindBy::Tag(tag) => {
                        if let Some(mytag) = tray.tag() {
                            if tag == mytag {
                                return Some(tray.as_member());
                            }
                        }
                    }
                }
            }
        }
        None
    }
    fn add_root(&mut self, m: Box<dyn controls::Closeable>) -> &mut dyn controls::Member {
        let base = &mut unsafe { cast_hwnd::<Application>(self.root) }.unwrap().base;
        
        let is_window = m.as_any().type_id() == TypeId::of::<crate::window::Window>();
        let is_tray = m.as_any().type_id() == TypeId::of::<crate::tray::Tray>();
        
        if is_window {
            let i = base.windows.len();
            base.windows.push(m.into_any().downcast::<crate::window::Window>().unwrap());
            return base.windows[i].as_mut().as_member_mut();
        }
        
        if is_tray {
            let i = base.trays.len();
            base.trays.push(m.into_any().downcast::<crate::tray::Tray>().unwrap());
            return base.trays[i].as_mut().as_member_mut();
        }
        
        panic!("Unsupported Closeable: {:?}", m.as_any().type_id());
    }
    fn close_root(&mut self, arg: types::FindBy, skip_callbacks: bool) -> bool {
        let base = &mut unsafe { cast_hwnd::<Application>(self.root) }.unwrap().base;
        match arg {
            types::FindBy::Id(id) => {
                (0..base.windows.len()).into_iter().find(|i| if base.windows[*i].id() == id 
                    && base.windows[*i].as_any_mut().downcast_mut::<crate::window::Window>().unwrap().inner_mut().inner_mut().inner_mut().inner_mut().close(skip_callbacks) {
                        base.windows.remove(*i);
                        true
                    } else {
                        false
                }).is_some()
                || 
                (0..base.trays.len()).into_iter().find(|i| if base.trays[*i].id() == id 
                    && base.trays[*i].as_any_mut().downcast_mut::<crate::tray::Tray>().unwrap().inner_mut().close(skip_callbacks) {
                        base.trays.remove(*i);
                        true
                    } else {
                        false
                }).is_some()
            }
            types::FindBy::Tag(tag) => {
                (0..base.windows.len()).into_iter().find(|i| if base.windows[*i].tag().is_some() && base.windows[*i].tag().unwrap() == Cow::Borrowed(tag.into()) 
                    && base.windows[*i].as_any_mut().downcast_mut::<crate::window::Window>().unwrap().inner_mut().inner_mut().inner_mut().inner_mut().close(skip_callbacks) {
                        base.windows.remove(*i);
                        true
                    } else {
                        false
                }).is_some()
                || 
                (0..base.trays.len()).into_iter().find(|i| if base.trays[*i].tag().is_some() && base.trays[*i].tag().unwrap() == Cow::Borrowed(tag.into()) 
                    && base.trays[*i].as_any_mut().downcast_mut::<crate::tray::Tray>().unwrap().inner_mut().close(skip_callbacks) {
                        base.trays.remove(*i);
                        true
                    } else {
                        false
                }).is_some()
            }
        }
    }
    fn exit(&mut self) {
        let base = &mut unsafe { cast_hwnd::<Application>(self.root) }.unwrap().base; 
        for mut window in base.windows.drain(..) {
            window.as_any_mut().downcast_mut::<crate::window::Window>().unwrap().inner_mut().inner_mut().inner_mut().inner_mut().close(true);
        }
        for mut tray in base.trays.drain(..) {
            tray.as_any_mut().downcast_mut::<crate::tray::Tray>().unwrap().inner_mut().close(true);
        }
    }
    fn roots<'a>(&'a self) -> Box<dyn Iterator<Item = &'a (dyn controls::Member)> + 'a> {
        unsafe { cast_hwnd::<Application>(self.root) }.unwrap().roots()
    }
    fn roots_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut (dyn controls::Member)> + 'a> {
        unsafe { cast_hwnd::<Application>(self.root) }.unwrap().roots_mut()
    }
}

impl HasNativeIdInner for WindowsApplication {
    type Id = common::Hwnd;

    fn native_id(&self) -> Self::Id {
        self.root.into()
    }
}

impl Drop for WindowsApplication {
    fn drop(&mut self) {
        destroy_hwnd(self.root, 0, None);
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

// TODO <O: controls::Application>
unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        if winuser::WM_CREATE == msg {
            let cs: &mut winuser::CREATESTRUCTW = mem::transmute(lparam);
            winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, cs.lpCreateParams as WinPtr);
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
            let tray = if let Some(tray) = w.base.trays.iter_mut().find(|tray| tray.as_any().downcast_ref::<crate::tray::Tray>().unwrap().inner().inner().inner().is_menu_shown()) {
                tray.as_any_mut().downcast_mut::<crate::tray::Tray>().unwrap()
            } else {
                return 0;
            };

            if lparam == 0 {
                let w2: &mut application::Application = mem::transmute(ww);

                let tray2 = if let Some(tray) = w2.base.trays.iter_mut().find(|tray| tray.as_any().downcast_ref::<crate::tray::Tray>().unwrap().inner().inner().inner().is_menu_shown()) {
                    tray.as_any_mut().downcast_mut::<crate::tray::Tray>().unwrap()
                } else {
                    return 0;
                };
                tray.inner_mut().inner_mut().inner_mut().run_menu(tray2);
            } else {
                let item = minwindef::LOWORD(wparam as u32);
                tray.inner_mut().inner_mut().inner_mut().select_menu(item as usize);
            }
        }
        crate::tray::MESSAGE => {
            let evt = minwindef::LOWORD(lparam as u32);
            let id = minwindef::HIWORD(lparam as u32);

            let w: &mut application::Application = mem::transmute(ww);

            let tray = if let Some(tray) = w.base.trays.iter_mut().find(|tray| tray.id() == ids::Id::from_raw(id as usize)) {
                tray.as_any_mut().downcast_mut::<crate::tray::Tray>().unwrap()
            } else {
                return 0;
            };

            match evt as u32 {
                winuser::WM_CONTEXTMENU => {
                    tray.inner_mut().inner_mut().inner_mut().toggle_menu();
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
        w.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().dispatch()
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
