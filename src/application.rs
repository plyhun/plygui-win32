use super::*;
use super::common::*;

use plygui_api::controls;
use plygui_api::ids::Id;
use plygui_api::types;

use winapi::shared::windef;
use winapi::um::commctrl;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub struct WindowsApplication {
    pub(crate) root: windef::HWND,
    name: String,
    windows: Vec<windef::HWND>,
    trays: Vec<ids::Id>,
}

pub type Application = ::plygui_api::development::Application<WindowsApplication>;

impl ApplicationInner for WindowsApplication {
    fn get() -> Box<Application> {
        init_comctl();
        
        let mut a = Box::new(Application::with_inner(
            WindowsApplication {
                name: String::new(), //name.into(), // TODO later
                windows: Vec::with_capacity(1),
                trays: Vec::with_capacity(0),
                root: 0 as windef::HWND,
            },
            (),
        ));
        
        let name = OsStr::new(a.as_inner().name.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
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
        a.as_inner_mut().root = hwnd;
        a
    }
    fn new_window(&mut self, title: &str, size: types::WindowStartSize, menu: types::Menu) -> Box<dyn controls::Window> {
        let w = window::WindowsWindow::with_params(title, size, menu);
        unsafe {
            use plygui_api::controls::HasNativeId;
            
            self.windows.push(w.native_id() as windef::HWND);
        }
        w
    }
    fn new_tray(&mut self, title: &str, menu: types::Menu) -> Box<dyn controls::Tray> {
        use plygui_api::controls::Member;
        
        let tray = tray::WindowsTray::with_params(title, menu);
        self.trays.push(tray.id());
        tray
    }
    fn name<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(self.name.as_str())
    }
    fn start(&mut self) {
        //for i in (0..self.windows.len()).rev() {
            /*let mut hwnd = AtomicPtr::new(self.windows[i].clone());
            thread::spawn(move ||
                { start_window(*hwnd.get_mut()); }
            );*/
            //start_window(self.windows[i]);
        //}
        
        let mut msg: winuser::MSG = unsafe { mem::zeroed() };
        while unsafe { winuser::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } > 0 {
            unsafe {
                winuser::TranslateMessage(&mut msg);
                winuser::DispatchMessageW(&mut msg);
            }
            let mut i = 0;
            while i < self.windows.len() {
                if dispatch_window(self.windows[i]) <= 0 {
                    self.windows.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }
    fn find_member_by_id_mut(&mut self, id: Id) -> Option<&mut dyn controls::Member> {
        use plygui_api::controls::{Container, Member, SingleContainer};

        for window in self.windows.as_mut_slice() {
            let window: &mut window::Window = common::member_from_hwnd::<window::Window>(*window);
            if window.id() == id {
                return Some(window.as_single_container_mut().as_container_mut().as_member_mut());
            } else {
                return window.find_control_by_id_mut(id).map(|control| control.as_member_mut());
            }
        }
        None
    }
    fn find_member_by_id(&self, id: Id) -> Option<&dyn controls::Member> {
        use plygui_api::controls::{Container, Member, SingleContainer};

        for window in self.windows.as_slice() {
            let window: &mut window::Window = common::member_from_hwnd::<window::Window>(*window);
            if window.id() == id {
                return Some(window.as_single_container().as_container().as_member());
            } else {
                return window.find_control_by_id_mut(id).map(|control| control.as_member());
            }
        }

        None
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

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        if winuser::WM_CREATE == msg {
            let cs: &mut winuser::CREATESTRUCTW = mem::transmute(lparam);
            winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, cs.lpCreateParams as isize);
        }
        //return winuser::DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn dispatch_window(hwnd: windef::HWND) -> i32 {
    let w: &mut window::Window = common::member_from_hwnd::<window::Window>(hwnd);
    w.as_inner_mut().as_inner_mut().as_inner_mut().dispatch()
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
