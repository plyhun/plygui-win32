use super::*;

use plygui_api::{development, ids, types, callbacks};
use plygui_api::traits::{UiControl, UiWindow, UiSingleContainer, UiMember, UiContainer, UiHasLabel};
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
	//pub static ref INSTANCE: winuser::HINSTANCE = unsafe { kernel32::GetModuleHandleW(ptr::null()) };
}

#[repr(C)]
pub struct Window {
    //base: development::UiMemberCommon,
    hwnd: windef::HWND,
    child: Option<Box<UiControl>>,

    h_resize: Option<callbacks::Resize>,
}

impl Window {
    pub(crate) fn new(title: &str, window_size: types::WindowStartSize, has_menu: bool) -> Box<Window> {
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
                        common::log_error();
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

            let mut w = Box::new(Window {
                base: Default::default(),

                hwnd: 0 as windef::HWND,
                child: None,
                h_resize: None,
            });

            if common::INSTANCE as usize == 0 {
                common::INSTANCE = libloaderapi::GetModuleHandleW(ptr::null());
            }

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
                common::INSTANCE,
                w.as_mut() as *mut _ as *mut c_void,
            );

            w.hwnd = hwnd;
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

impl development::UiWindowExtension for Window {}

impl development::UiHasLabelExtension for Window {
	fn label<'a>(&'a self) -> ::std::borrow::Cow<'a, str> {
		if self.hwnd != 0 as windef::HWND {
			let mut wbuffer = vec![0u16; 4096];
			let len = unsafe { winuser::GetWindowTextW(self.hwnd, wbuffer.as_mut_slice().as_mut_ptr(), 4096) };
			Cow::Owned(String::from_utf16_lossy(&wbuffer.as_slice()[..len as usize]))
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

impl development::UiContainerExtension for Window {
    fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut UiControl> {
        if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_by_id_mut(id_);
            }
        }
        None
    }
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&UiControl> {
        if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control_by_id(id_);
            }
        }
        None
    }
}

impl development::UiSingleContainerExtension for Window {
    fn set_child(&mut self, mut child: Option<Box<UiControl>>) -> Option<Box<UiControl>> {
        let mut old = self.child.take();
        if let Some(old) = old.as_mut() {
            old.on_removed_from_container(self);
        }
        if let Some(new) = child.as_mut() {
            new.on_added_to_container(self, 0, 0); 

        }
        self.child = child;

        old
    }
    fn child(&self) -> Option<&UiControl> {
        self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut UiControl> {
        //self.child.as_mut().map(|c|c.as_mut()) // WTF ??
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
}

impl development::UiMemberExtension for Window {
    fn size(&self) -> (u16, u16) {
        let rect = unsafe { common::window_rect(self.hwnd) };
        (
            (rect.right - rect.left) as u16,
            (rect.bottom - rect.top) as u16,
        )
    }

    fn on_resize(&mut self, handler: Option<callbacks::Resize>) {
        self.h_resize = handler;
    }

    fn set_visibility(&mut self, visibility: types::Visibility) {
        self.base.visibility = visibility;
        unsafe {
            winuser::ShowWindow(
                self.hwnd,
                if self.base.visibility == types::Visibility::Visible {
                    winuser::SW_SHOW
                } else {
                    winuser::SW_HIDE
                },
            );
        }
    }
    fn visibility(&self) -> types::Visibility {
        self.base.visibility
    }
    unsafe fn native_id(&self) -> usize {
        self.hwnd as usize
    }
}

impl Drop for Window {
    fn drop(&mut self) {
    	use plygui_api::development::{UiMemberExtension, UiSingleContainerExtension};
    	
        self.set_child(None);
        self.set_visibility(types::Visibility::Gone);
        common::destroy_hwnd(self.hwnd, 0, None);
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
            let mut w: &mut window::Window = mem::transmute(ww);

            if let Some(ref mut child) = w.child {
                child.measure(width, height);
                child.draw(Some((0, 0))); //TODO padding
            }

            if let Some(ref mut cb) = w.h_resize {
                let w2: &mut development::UiMemberBase<Window> = mem::transmute(winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA));
                (cb.as_mut())(w2, width, height);
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
