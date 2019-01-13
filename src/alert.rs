use super::common::*;
use super::*;

#[repr(C)]
pub struct WindowsAlert {
    hwnd: windef::HWND,
    label: String,
    text: String,
}

pub type Alert = Member<WindowsAlert>;

impl HasLabelInner for WindowsAlert {
    fn label<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(self.label.as_ref())
    }
    fn set_label(&mut self, _base: &mut MemberBase, label: &str) {
        self.label = label.into();
        if !self.hwnd.is_null() {
            let control_name = OsStr::new(&self.label).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
            unsafe {
                winuser::SetWindowTextW(self.hwnd, control_name.as_ptr());
            }
        }
    }
}

impl AlertInner for WindowsAlert {
    fn with_text(label: &str, text: &str, severity: types::AlertSeverity, parent: Option<&controls::Member>) -> Box<Member<Self>> {
        let label_u16 = OsStr::new(label).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
        let text_u16 = OsStr::new(text).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
        let mut cfg: commctrl::TASKDIALOGCONFIG = unsafe { mem::zeroed() };
        cfg.cbSize = mem::size_of::<commctrl::TASKDIALOGCONFIG>() as u32;
        cfg.hwndParent = if let Some(parent) = parent { unsafe { parent.native_id() as windef::HWND } } else { 0 as windef::HWND };
        cfg.hInstance = common::hinstance();
        cfg.pfCallback = Some(dialog_proc);
        cfg.pszWindowTitle = label_u16.as_ptr();
        cfg.pszMainInstruction = text_u16.as_ptr();
        unsafe { 
            *cfg.u1.pszMainIcon_mut() = match severity {
                types::AlertSeverity::Info => commctrl::TD_INFORMATION_ICON,
                types::AlertSeverity::Alert => commctrl::TD_ERROR_ICON,
            };
        }
        
        unsafe {
            if winerror::S_OK != commctrl::TaskDialogIndirect(&cfg, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) {
                common::log_error();
            }
        }
        let a: Box<Alert> = Box::new(Member::with_inner(
            WindowsAlert {
                hwnd: 0 as windef::HWND,
                label: label.into(),
                text: text.into(),
            },
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        a
    }
    fn severity(&self) -> types::AlertSeverity {
        types::AlertSeverity::Alert
    }
}

impl MemberInner for WindowsAlert {
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

impl Drop for WindowsAlert {
    fn drop(&mut self) {
        destroy_hwnd(self.hwnd, 0, None);
    }
}

unsafe extern "system" fn dialog_proc(hwnd: windef::HWND, msg: minwindef::UINT, _wparam: minwindef::WPARAM, _lparam: minwindef::LPARAM, _param: isize) -> i32 {
    let mut lr = 0;
    
    match msg {
        winuser::WM_CLOSE => {
            lr = winuser::EndDialog(hwnd, 0);
        }
        /*winuser::WM_COMMAND => {
            match minwindef::LOWORD(wparam as u32)    {    
                winuser::IDC_WAIT => {
                    winuser::EndDialog(hwnd, 0);
                },
                _ => {}
            }
        }*/
        _ => {}
    }

    lr
}

impl_all_defaults!(Alert);
