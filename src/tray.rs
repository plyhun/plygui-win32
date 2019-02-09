use super::common::*;
use super::*;

use winapi::um::shellapi;

#[repr(C)]
pub struct WindowsTray {
    label: String,
    cfg: shellapi::NOTIFYICONDATAW,
    on_close: Option<callbacks::Action>,
    skip_callbacks: bool,
}

pub type Tray = Member<WindowsTray>;

impl HasLabelInner for WindowsTray {
    fn label<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(self.label.as_ref())
    }
    fn set_label(&mut self, _base: &mut MemberBase, label: &str) {
        self.label = label.into();
        if !self.cfg.hWnd.is_null() {
            let control_name = common::str_to_wchar(&self.label);
            unsafe {
                winuser::SetWindowTextW(self.cfg.hWnd, control_name.as_ptr());
            }
        }
    }
}

impl CloseableInner for WindowsTray {
    fn close(&mut self, skip_callbacks: bool) {
        self.skip_callbacks = skip_callbacks;
        
        let mut app = application::Application::get();
        let app = app.as_any_mut().downcast_mut::<application::Application>().unwrap();
        app.as_inner_mut().remove_tray(unsafe {ids::Id::from_raw(self.cfg.uID as usize)});
        
        unsafe {
            if shellapi::Shell_NotifyIconW(shellapi::NIM_DELETE, &mut self.cfg) == minwindef::FALSE {
                common::log_error();
            }
        }
    }
    fn on_close(&mut self, callback: Option<callbacks::Action>) {
        self.on_close = callback;
    }
}

impl TrayInner for WindowsTray {
    fn with_params(title: &str, _menu: types::Menu) -> Box<Member<Self>> {
        use plygui_api::controls::Member as OuterMember;
        
        let mut t = Box::new(Member::with_inner(WindowsTray {
                label: title.into(),    
                cfg: unsafe { mem::zeroed() },
                on_close: None,
                skip_callbacks: false,
            }, 
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)
        ));
        
        let app = super::application::Application::get();
        let tip_size = t.as_inner_mut().cfg.szTip.len();
        let title = OsStr::new(t.as_inner().label.as_str()).encode_wide().take(tip_size - 1).chain(Some(0).into_iter()).collect::<Vec<_>>();
        
        t.as_inner_mut().cfg.hWnd = unsafe { app.native_id() as windef::HWND };
        t.as_inner_mut().cfg.cbSize = mem::size_of::<shellapi::NOTIFYICONDATAW>() as u32;
        t.as_inner_mut().cfg.uID = unsafe { t.id().into_raw() as u32 }; 
        //t.as_inner_mut().cfg.hIcon = unsafe { winuser::GetClassLongW(app.as_inner().root.into(), winuser::GCL_HICON) as windef::HICON };
        
        unsafe { commctrl::LoadIconMetric(ptr::null_mut(), winuser::MAKEINTRESOURCEW(32512), commctrl::LIM_SMALL as i32, &mut t.as_inner_mut().cfg.hIcon); }
        
        t.as_inner_mut().cfg.uFlags = shellapi::NIF_ICON | shellapi::NIF_TIP | shellapi::NIF_MESSAGE | shellapi::NIF_SHOWTIP;
        t.as_inner_mut().cfg.uCallbackMessage = 12345678;
        t.as_inner_mut().cfg.szTip[..title.len()].clone_from_slice(title.as_slice());
        unsafe {
            if shellapi::Shell_NotifyIconW(shellapi::NIM_ADD, &mut t.as_inner_mut().cfg) == minwindef::FALSE {
                common::log_error();
            }
            *t.as_inner_mut().cfg.u.uVersion_mut() = shellapi::NOTIFYICON_VERSION_4;
            if shellapi::Shell_NotifyIconW(shellapi::NIM_SETVERSION, &mut t.as_inner_mut().cfg) == minwindef::FALSE {
                common::log_error();
            }
        }
    
        t
    }
}

impl HasNativeIdInner for WindowsTray {
    type Id = common::Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.cfg.hWnd.into()
    }
}

impl MemberInner for WindowsTray {}

impl Drop for WindowsTray {
    fn drop(&mut self) {
        unsafe {
            if shellapi::Shell_NotifyIconW(shellapi::NIM_DELETE, &mut self.cfg) == minwindef::FALSE {
                common::log_error();
            }
        }
    }
}

impl_all_defaults!(Tray);
