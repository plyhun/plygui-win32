use super::common::*;
use super::*;

struct WindowsMessageAction {
	title: Vec<u16>,
	id: i32,
	cb: callbacks::Action,
}
impl From<(String, callbacks::Action)> for WindowsMessageAction {
	fn from(a: (String, callbacks::Action)) -> Self {
		WindowsMessageAction {
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
pub struct WindowsMessage {
    hwnd: windef::HWND,
    label: String,
    text: String,
    cfg: commctrl::TASKDIALOGCONFIG,
    actions: Vec<WindowsMessageAction>,
}

pub type Message = Member<WindowsMessage>;

impl HasLabelInner for WindowsMessage {
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

impl MessageInner for WindowsMessage {
    fn with_actions(content: types::TextContent, severity: types::MessageSeverity, actions: Vec<(String, callbacks::Action)>, parent: Option<&controls::Member>) -> Box<Member<Self>> {
    	let (label, text) = match content {
    		types::TextContent::Plain(text) => (String::new(/* TODO app name here? */), text),
    		types::TextContent::LabelDescription(label, description) => (label, description),
    	};
        let mut a: Box<Message> = Box::new(Member::with_inner(
            WindowsMessage {
                hwnd: 0 as windef::HWND,
                cfg: unsafe { mem::zeroed() },
                label: label,
                text: text,
                actions: actions.into_iter().map(|a| a.into()).collect(),
            },
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        
        a.as_inner_mut().cfg.cbSize = mem::size_of::<commctrl::TASKDIALOGCONFIG>() as u32;
        a.as_inner_mut().cfg.hwndParent = if let Some(parent) = parent { unsafe { parent.native_id() as windef::HWND } } else { 0 as windef::HWND };
        a.as_inner_mut().cfg.hInstance = common::hinstance();
        a.as_inner_mut().cfg.pfCallback = Some(dialog_proc);
        a.as_inner_mut().cfg.lpCallbackData = a.as_mut() as *mut Message as isize;
        
        unsafe { 
            *a.as_inner_mut().cfg.u1.pszMainIcon_mut() = match severity {
                types::MessageSeverity::Info => commctrl::TD_INFORMATION_ICON,
                types::MessageSeverity::Warning => commctrl::TD_WARNING_ICON,
                types::MessageSeverity::Alert => commctrl::TD_ERROR_ICON,
            };
        }
        a
    }
    fn start(&mut self) -> Result<String, ()> {
        let label_u16 = common::str_to_wchar(&self.label);
        let text_u16 = common::str_to_wchar(&self.text);
        
        self.cfg.pszWindowTitle = label_u16.as_ptr();
        self.cfg.pszMainInstruction = text_u16.as_ptr();
        
        let actions = self.actions.iter().map(|a| commctrl::TASKDIALOG_BUTTON {
	        nButtonID: a.id,
	        pszButtonText: a.title.as_ptr(),
        }).collect::<Vec<_>>();
        if actions.len() > 0 {
        	self.cfg.pButtons = actions.as_ptr();
        	self.cfg.cButtons = actions.len() as u32;
        }
        
        let mut pressed = -1;
        unsafe {
            if winerror::S_OK != commctrl::TaskDialogIndirect(&self.cfg, &mut pressed, ptr::null_mut(), ptr::null_mut()) || pressed < 0 {
                common::log_error();
                return Err(());
            }
        }
        self.actions.iter().find(|a| a.id == pressed).map(|a| String::from_utf16_lossy(a.title[..a.title.len()-1].as_ref())).ok_or(())
    }
    fn severity(&self) -> types::MessageSeverity {
        match unsafe { *self.cfg.u1.pszMainIcon() as *mut u16 } {
            commctrl::TD_INFORMATION_ICON => types::MessageSeverity::Info,
            commctrl::TD_WARNING_ICON => types::MessageSeverity::Warning,
            commctrl::TD_ERROR_ICON => types::MessageSeverity::Alert,
            _ => unreachable!()
        }
    }
}

impl MemberInner for WindowsMessage {
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

impl Drop for WindowsMessage {
    fn drop(&mut self) {
        destroy_hwnd(self.hwnd, 0, None);
    }
}

unsafe extern "system" fn dialog_proc(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, _lparam: minwindef::LPARAM, param: isize) -> i32 {
    let mut lr = 0;
    
    let alert: &mut Message = mem::transmute(param);
    if alert.as_inner_mut().hwnd.is_null() {
    	alert.as_inner_mut().hwnd = hwnd;
    }
    match msg {
        winuser::WM_CLOSE => {
            lr = winuser::EndDialog(hwnd, 0);
        },
        winuser::WM_DESTROY => {
        	let alert2: &mut Message = mem::transmute(param);
        	match alert.as_inner_mut().actions.iter_mut().find(|a| a.id == wparam as i32) {
        	    Some(a) => {
        	        if !(a.cb.as_mut())(alert2) {
        			    return 0;
        			}
        	    },
        	    None => {}
        	}
        }
        _ => {}
    }
    lr
}

impl_all_defaults!(Message);
