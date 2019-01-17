use super::common::*;
use super::*;

struct WindowsAlertAction {
	title: Vec<u16>,
	id: i32,
	cb: callbacks::Action,
}
impl From<(String, callbacks::Action)> for WindowsAlertAction {
	fn from(a: (String, callbacks::Action)) -> Self {
		WindowsAlertAction {
			id: {
		        use std::hash::{Hash, Hasher};
		        use std::collections::hash_map::DefaultHasher;
		
		        let mut hasher = DefaultHasher::new();
		        a.0.hash(&mut hasher);
		        hasher.finish() as i32
		    },
			title: common::str_to_wchar(&a.0),
			cb: a.1
		}
	}
}

#[repr(C)]
pub struct WindowsAlert {
    hwnd: windef::HWND,
    label: String,
    text: String,
    actions: Vec<WindowsAlertAction>,
}

pub type Alert = Member<WindowsAlert>;

impl HasLabelInner for WindowsAlert {
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

impl AlertInner for WindowsAlert {
    fn with_actions(content: types::TextContent, severity: types::AlertSeverity, mut actions: Vec<(String, callbacks::Action)>, parent: Option<&controls::Member>) -> Box<Member<Self>> {
    	let (label, text) = match content {
    		types::TextContent::Plain(text) => (String::new(/* TODO app name here? */), text),
    		types::TextContent::LabelDescription(label, description) => (label, description),
    	};
        let label_u16 = common::str_to_wchar(&label);
        let text_u16 = common::str_to_wchar(&text);
        
        let mut a: Box<Alert> = Box::new(Member::with_inner(
            WindowsAlert {
                hwnd: 0 as windef::HWND,
                label: label,
                text: text,
                actions: actions.drain(..).map(|a| a.into()).collect()
            },
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        
        let mut cfg: commctrl::TASKDIALOGCONFIG = unsafe { mem::zeroed() };
        cfg.cbSize = mem::size_of::<commctrl::TASKDIALOGCONFIG>() as u32;
        cfg.hwndParent = if let Some(parent) = parent { unsafe { parent.native_id() as windef::HWND } } else { 0 as windef::HWND };
        cfg.hInstance = common::hinstance();
        cfg.pszWindowTitle = label_u16.as_ptr();
        cfg.pszMainInstruction = text_u16.as_ptr();
        cfg.pfCallback = Some(dialog_proc);
        cfg.lpCallbackData = a.as_mut() as *mut Alert as isize;
        
        let actions = a.as_inner().actions.iter().map(|a| commctrl::TASKDIALOG_BUTTON {
	        nButtonID: a.id,
	        pszButtonText: a.title.as_ptr(),
        }).collect::<Vec<_>>();
        if actions.len() > 0 {
        	cfg.pButtons = actions.as_ptr();
        	cfg.cButtons = actions.len() as u32;
        }
        
        unsafe { 
            *cfg.u1.pszMainIcon_mut() = match severity {
                types::AlertSeverity::Info => commctrl::TD_INFORMATION_ICON,
                types::AlertSeverity::Alert => commctrl::TD_ERROR_ICON,
            };
            if winerror::S_OK != commctrl::TaskDialogIndirect(&cfg, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) {
                common::log_error();
            }
        }
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

unsafe extern "system" fn dialog_proc(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, _lparam: minwindef::LPARAM, param: isize) -> i32 {
    let mut lr = 0;
    
    let alert: &mut Alert = mem::transmute(param);
    if alert.as_inner_mut().hwnd.is_null() {
    	alert.as_inner_mut().hwnd = hwnd;
    }
    match msg {
        winuser::WM_CLOSE => {
            lr = winuser::EndDialog(hwnd, 0);
        },
        winuser::WM_DESTROY => {
        	let alert2: &mut Alert = mem::transmute(param);
        	let _ = alert.as_inner_mut().actions.iter_mut().filter(|a| a.id == wparam as i32).for_each(|a| {
        			(a.cb.as_mut())(alert2);
        	});
        }
        _ => {}
    }
    lr
}

impl_all_defaults!(Alert);
