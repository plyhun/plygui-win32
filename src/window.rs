use super::*;
use super::common::*;

use plygui_api::development::{SingleContainerInner, MemberInner, Member, SingleContainer, HasLabelInner, WindowInner, ContainerInner, HasInner};
use plygui_api::{ids, types, callbacks, traits};
use plygui_api::members::MEMBER_ID_WINDOW;

use winapi::shared::windef;
use winapi::shared::minwindef;
use winapi::shared::ntdef;
use winapi::um::winuser;
use winapi::um::libloaderapi;
use winapi::ctypes::c_void;

use std::{ptr, mem, str};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::borrow::Cow;

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

#[repr(C)]
pub struct WindowsWindow {
    id: ids::Id,
	visibility: types::Visibility,
	
    hwnd: windef::HWND,
    child: Option<types::Dbox<traits::UiControl>>,

    h_resize: Option<callbacks::Resize>,
}

pub type Window = Member<SingleContainer<WindowsWindow>>;

impl WindowsWindow {
    pub(crate) fn new(title: &str, window_size: types::WindowStartSize, has_menu: bool) -> types::Dbox<traits::UiWindow> {
    	use plygui_api::development::HasInner;
    	
        unsafe {
            let mut rect = match window_size {
                types::WindowStartSize::Exact(width, height) => windef::RECT {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                },
                types::WindowStartSize::Fullscreen => {
                    let mut rect = windef::RECT {
                        left: 0,
                        right: 0,
                        top: 0,
                        bottom: 0,
                    };
                    if winuser::SystemParametersInfoW(
                        winuser::SPI_GETWORKAREA,
                        0,
                        &mut rect as *mut _ as *mut c_void,
                        0,
                    ) == 0
                    {
                        log_error();
                        windef::RECT {
                            left: 0,
                            top: 0,
                            right: 640,
                            bottom: 480,
                        }
                    } else {
                        windef::RECT {
                            left: 0,
                            top: 0,
                            right: rect.right,
                            bottom: rect.bottom,
                        }
                    }
                }
            };
            let style = winuser::WS_OVERLAPPEDWINDOW;
            let exstyle = winuser::WS_EX_APPWINDOW;

            winuser::AdjustWindowRectEx(&mut rect, style, minwindef::FALSE, exstyle);
            let window_name = OsStr::new(title)
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect::<Vec<_>>();
                
            let w: Box<traits::UiWindow> = Box::new(Window::with_inner(WindowsWindow {
		                id: ids::Id::next(),
		                visibility: types::Visibility::Visible,
		
		                hwnd: 0 as windef::HWND,
		                child: None,
		                h_resize: None,
		            }));    
 
            let mut w: Box<Box<traits::UiWindow>> = Box::new(w);

            let hwnd = winuser::CreateWindowExW(
                exstyle,
                WINDOW_CLASS.as_ptr(),
                window_name.as_ptr() as ntdef::LPCWSTR,
                style | winuser::WS_VISIBLE,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance(),
                w.as_mut() as *mut _ as *mut c_void,
            );

            w.as_single_container_mut().as_container_mut().as_member_mut().as_any_mut().downcast_mut::<Window>().unwrap().as_inner_mut().hwnd = hwnd;
            w
        }
    }
    pub(crate) fn start(&mut self) {
        loop {
            unsafe {
                let mut msg: winuser::MSG = mem::zeroed();
                if winuser::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) <= 0 {
                    break;
                } else {
                    winuser::TranslateMessage(&mut msg);
                    winuser::DispatchMessageW(&mut msg);
                }
            }
        }
    }
}

impl HasLabelInner for WindowsWindow {
    fn label<'a>(&'a self) -> ::std::borrow::Cow<'a, str> {
        if self.hwnd != 0 as windef::HWND {
            let mut wbuffer = vec![0u16; 4096];
            let len = unsafe { winuser::GetWindowTextW(self.hwnd, wbuffer.as_mut_slice().as_mut_ptr(), 4096) };
            Cow::Owned(String::from_utf16_lossy(
                &wbuffer.as_slice()[..len as usize],
            ))
        } else {
            panic!("Unattached window!");
        }
    }
    fn set_label(&mut self, label: &str) {
        if self.hwnd != 0 as windef::HWND {
            let control_name = OsStr::new(label)
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect::<Vec<_>>();
            unsafe {
                winuser::SetWindowTextW(self.hwnd, control_name.as_ptr());
            }
        }
    }
}

impl WindowInner for WindowsWindow {
}

impl ContainerInner for WindowsWindow {
    fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut traits::UiControl> {
        if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_by_id_mut(id_);
            }
        }
        None
    }
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&traits::UiControl> {
        if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control_by_id(id_);
            }
        }
        None
    }
}

impl SingleContainerInner for WindowsWindow {
    fn set_child(&mut self, mut child: Option<types::Dbox<traits::UiControl>>) -> Option<types::Dbox<traits::UiControl>> {
        let mut old = self.child.take();
        if let Some(old) = old.as_mut() {
        	let outer_self: &mut Box<traits::UiWindow> = unsafe { common::cast_hwnd(self.hwnd) };
        	let outer_self = outer_self.as_single_container_mut().as_container_mut();
            old.on_removed_from_container(outer_self);
        }
        if let Some(new) = child.as_mut() {
            let outer_self: &mut Box<traits::UiWindow> = unsafe { common::cast_hwnd(self.hwnd) }; 
        	let outer_self = outer_self.as_single_container_mut().as_container_mut();
            new.on_added_to_container(outer_self, 0, 0); 
        }
        self.child = child;

        old
    }
    fn child(&self) -> Option<&traits::UiControl> {
        self.child.as_ref().map(|c| c.as_ref().as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut traits::UiControl> {
        //self.child.as_mut().map(|c|c.as_mut()) // WTF ??
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut().as_mut())
        } else {
            None
        }
    }
}

impl MemberInner for WindowsWindow {
	type Id = common::Hwnd;
	
    fn id(&self) -> ids::Id { self.id }
    
    fn size(&self) -> (u16, u16) {
        let rect = unsafe { window_rect(self.hwnd) };
        (
            (rect.right - rect.left) as u16,
            (rect.bottom - rect.top) as u16,
        )
    }

    fn on_resize(&mut self, handler: Option<callbacks::Resize>) {
        self.h_resize = handler;
    }

    fn set_visibility(&mut self, visibility: types::Visibility) {
        self.visibility = visibility;
        unsafe {
            winuser::ShowWindow(
                self.hwnd,
                if self.visibility == types::Visibility::Visible {
                    winuser::SW_SHOW
                } else {
                    winuser::SW_HIDE
                },
            );
        }
    }
    fn visibility(&self) -> types::Visibility {
        self.visibility
    }
    unsafe fn native_id(&self) -> Self::Id {
        self.hwnd.into()
    }
}

impl Drop for WindowsWindow {
    fn drop(&mut self) {
    	self.set_child(None);
        self.set_visibility(types::Visibility::Gone);
        destroy_hwnd(self.hwnd, 0, None);
    }
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new(MEMBER_ID_WINDOW)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();

    let class = winuser::WNDCLASSEXW {
        cbSize: mem::size_of::<winuser::WNDCLASSEXW>() as minwindef::UINT,
        style: winuser::CS_DBLCLKS,
        lpfnWndProc: Some(handler),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: libloaderapi::GetModuleHandleW(ptr::null()),
        hIcon: winuser::LoadIconW(ptr::null_mut(), winuser::IDI_APPLICATION),
        hCursor: winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_ARROW),
        hbrBackground: (winuser::COLOR_BTNFACE + 1) as windef::HBRUSH,
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
        return winuser::DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    match msg {
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;
            let mut w: &mut Box<traits::UiWindow> = mem::transmute(ww);

            if let Some(ref mut child) = w.as_single_container_mut().as_container_mut().as_member_mut().as_any_mut().downcast_mut::<window::Window>().unwrap().as_inner_mut().child {
                child.measure(width, height);
                child.draw(Some((0, 0))); //TODO padding
            }

            if let Some(ref mut cb) = w.as_single_container_mut().as_container_mut().as_member_mut().as_any_mut().downcast_mut::<window::Window>().unwrap().as_inner_mut().h_resize {
                let mut w2: &mut Box<traits::UiWindow> = mem::transmute(ww);
                (cb.as_mut())(w2.as_single_container_mut().as_container_mut().as_member_mut(), width, height);
            }
        }
        winuser::WM_DESTROY => {
            winuser::PostQuitMessage(0);
            return 0;
        }
        /*winuser::WM_PRINTCLIENT => {
        	winuser::SendMessageW(hwnd, winuser::WM_ERASEBKGND, wparam, lparam);
	        return 0;
        },*/
        /*winuser::WM_NOTIFY => {
        	let hdr: winuser::LPNMHDR = mem::transmute(lparam);
        	println!("notify for {:?}", hdr);
        },
        winuser::WM_COMMAND => {
        	let hdr: winuser::LPNMHDR = mem::transmute(lparam);
        	
        	println!("command for {:?}", hdr);
        }*/
        _ => {}
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}
