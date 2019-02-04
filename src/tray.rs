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
            if shellapi::Shell_NotifyIconW(shellapi::NIM_DELETE, &mut self.cfg) = minwindef::FALSE {
                common::log_error();
            }
        }
    }
    fn on_close(&mut self, callback: Option<callbacks::Action>) {
        self.on_close = callback;
    }
}

impl TrayInner for WindowsTray {
    fn with_params(title: &str, menu: types::Menu) -> Box<Member<Self>> {
        let mut t = Box::new(Member::with_inner(WindowsTray {
                hwnd: 0 as windef::HWND,    
                label: title.into(),    
                cfg: unsafe { mem::zeroed() },
                on_close: None,
                skip_callbacks: false,
            }, 
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)
        ));
        
        
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

unsafe extern "system" fn dialog_proc(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, _lparam: minwindef::LPARAM, param: isize) -> i32 {
    let mut lr = 0;
    
    let alert: &mut Tray = mem::transmute(param);
    if alert.as_inner_mut().hwnd.is_null() {
    	alert.as_inner_mut().hwnd = hwnd;
    }
    match msg {
        winuser::WM_CLOSE => {
            lr = winuser::EndDialog(hwnd, 0);
        },
        _ => {}
    }
    lr
}

impl_all_defaults!(Tray);
