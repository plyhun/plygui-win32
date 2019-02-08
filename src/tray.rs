use super::common::*;
use super::*;

use winapi::um::shellapi;

#[repr(C)]
pub struct WindowsTray {
    hwnd: windef::HWND,
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
        if !self.hwnd.is_null() {
            let control_name = common::str_to_wchar(&self.label);
            unsafe {
                winuser::SetWindowTextW(self.hwnd, control_name.as_ptr());
            }
        }
    }
}

impl CloseableInner for WindowsTray {
    fn close(&mut self, skip_callbacks: bool) {
        self.skip_callbacks = skip_callbacks;
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
                hwnd: 0 as windef::HWND,    
                label: title.into(),    
                cfg: unsafe { mem::zeroed() },
                on_close: None,
                skip_callbacks: false,
            }, 
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)
        ));
        
        let app = super::application::WindowsApplication::get();
        let tip_size = t.as_inner_mut().cfg.szTip.len();
        let title = OsStr::new(t.as_inner().label.as_str()).encode_wide().take(tip_size - 1).chain(Some(0).into_iter()).collect::<Vec<_>>();
        
        t.as_inner_mut().cfg.hWnd = app.as_inner().root.into();
        t.as_inner_mut().cfg.cbSize = mem::size_of::<shellapi::NOTIFYICONDATAW>() as u32;
        t.as_inner_mut().cfg.uID = unsafe { t.id().into_raw() as u32 }; 
        t.as_inner_mut().cfg.hIcon = unsafe { winuser::GetClassLongW(app.as_inner().root.into(), winuser::GCL_HICON) as windef::HICON };        
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

impl MemberInner for WindowsTray {
    type Id = common::Hwnd;

    fn size(&self) -> (u16, u16) {
        common::size_hwnd(self.hwnd)
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        if !self.hwnd.is_null() {
            unsafe {
                winuser::ShowWindow(self.hwnd, if base.visibility == types::Visibility::Visible { winuser::SW_SHOW } else { winuser::SW_HIDE });
            }
        }
    }

    unsafe fn native_id(&self) -> Self::Id {
        self.hwnd.into()
    }
}

impl Drop for WindowsTray {
    fn drop(&mut self) {
        destroy_hwnd(self.hwnd, 0, None);
    }
}

impl_all_defaults!(Tray);
