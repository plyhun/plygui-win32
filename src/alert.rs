use super::common::*;
use super::*;

#[repr(C)]
pub struct WindowsAlert {
    hwnd: windef::HWND,
    label: String,
}

pub type Alert = Member<WindowsAlert>;

impl WindowsAlert {
   }

impl HasLabelInner for WindowsAlert {
    fn label<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(self.label.as_ref())
    }
    fn set_label(&mut self, _base: &mut MemberBase, label: &str) {
        self.label = label.into();
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            let control_name = OsStr::new(&self.label).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
            unsafe {
                winuser::SetWindowTextW(self.base.hwnd, control_name.as_ptr());
            }
            self.base.invalidate();
        }
    }
}

impl AlertInner for WindowsAlert {
	fn with_text(text: &str, severity: types::AlertSeverity) -> Box<Member<Self>> {
		let a: Box<Alert> = Box::new(Member::with_inner(
            WindowsAlert {
	            hwnd: unsafe {  },
	            text: text,
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
        self.size_inner()
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        unsafe {
            winuser::ShowAlert(self.hwnd, if base.visibility == types::Visibility::Visible { winuser::SW_SHOW } else { winuser::SW_HIDE });
        }
    }

    unsafe fn native_id(&self) -> Self::Id {
        self.hwnd.into()
    }
}

impl Drop for WindowsAlert {
    fn drop(&mut self) {
        let self2 = common::member_from_hwnd::<Alert>(self.hwnd);
        destroy_hwnd(self.hwnd, 0, None);
    }
}

impl_all_defaults!(Alert);
