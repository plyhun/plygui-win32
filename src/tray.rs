use crate::common::{self, *};
use crate::application::Application;

use winapi::um::shellapi;

pub const MESSAGE: u32 = 0xbaba;

#[repr(C)]
pub struct WindowsTray {
    label: String,
    cfg: shellapi::NOTIFYICONDATAW,
    menu: (windef::HMENU, Vec<callbacks::Action>, isize),
    on_close: Option<callbacks::Action>,
    this: *mut Tray,
}

pub type Tray = Member<WindowsTray>;

impl WindowsTray {
    pub(crate) fn toggle_menu(&mut self) {
        if !self.menu.0.is_null() {
            unsafe {
                let hwnd = Application::get().native_id() as windef::HWND;
                if self.menu.2 > -2 {
                    self.menu.2 = -2;
                    winuser::SendMessageW(hwnd, winuser::WM_CANCELMODE, 0, 0);
                } else {
                    self.menu.2 = -1;
                    let mut click_point = mem::zeroed();
                    winuser::GetCursorPos(&mut click_point);
                    winuser::TrackPopupMenu(self.menu.0, winuser::TPM_LEFTALIGN | winuser::TPM_LEFTBUTTON | winuser::TPM_BOTTOMALIGN, click_point.x, click_point.y, 0, hwnd, ptr::null_mut());
                }
            }
        }
    }
    pub(crate) fn is_menu_shown(&self) -> bool {
        self.menu.2 > -2
    }
    pub(crate) fn select_menu(&mut self, id: usize) {
        self.menu.2 = id as isize;
    }
    pub(crate) fn run_menu(&mut self, this: &mut Tray) {
        if self.menu.2 > -1 {
            if let Some(a) = self.menu.1.get_mut(self.menu.2 as usize) {
                (a.as_mut())(this);
            }
        }
        self.menu.2 = -2;
    }
}

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
    fn close(&mut self, skip_callbacks: bool) -> bool {
        if !skip_callbacks {
            if let Some(ref mut on_close) = self.on_close {
                if !(on_close.as_mut())(unsafe { &mut *self.this }) {
                    return false;
                }
            }
        }

        let mut app = Application::get();
        let app = app.as_any_mut().downcast_mut::<Application>().unwrap();

        unsafe {
            if shellapi::Shell_NotifyIconW(shellapi::NIM_DELETE, &mut self.cfg) == minwindef::FALSE {
                common::log_error();
            }
        }
        app.as_inner_mut().remove_tray((self.this as windef::HWND).into());

        true
    }
    fn on_close(&mut self, callback: Option<callbacks::Action>) {
        self.on_close = callback;
    }
}

impl TrayInner for WindowsTray {
    fn with_params(title: &str, menu: types::Menu) -> Box<Member<Self>> {
        use plygui_api::controls::Member as OuterMember;

        let mut t = Box::new(Member::with_inner(
            WindowsTray {
                label: title.into(),
                cfg: unsafe { mem::zeroed() },
                menu: (ptr::null_mut(), if menu.is_some() { Vec::new() } else { vec![] }, -2),
                on_close: None,
                this: ptr::null_mut(),
            },
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let this = t.as_mut() as *mut Tray;
        t.as_inner_mut().this = this;

        let app = super::application::Application::get();
        let tip_size = t.as_inner_mut().cfg.szTip.len();
        let title = OsStr::new(t.as_inner().label.as_str()).encode_wide().take(tip_size - 1).chain(Some(0).into_iter()).collect::<Vec<_>>();

        t.as_inner_mut().cfg.hWnd = unsafe { app.native_id() as windef::HWND };
        t.as_inner_mut().cfg.cbSize = mem::size_of::<shellapi::NOTIFYICONDATAW>() as u32;
        t.as_inner_mut().cfg.uID = unsafe { t.id().into_raw() as u32 };
        //t.as_inner_mut().cfg.hIcon = unsafe { winuser::GetClassLongW(app.as_inner().root.into(), winuser::GCL_HICON) as windef::HICON };

        unsafe {
            commctrl::LoadIconMetric(ptr::null_mut(), winuser::MAKEINTRESOURCEW(32512), commctrl::LIM_SMALL as i32, &mut t.as_inner_mut().cfg.hIcon);
        }

        t.as_inner_mut().cfg.uFlags = shellapi::NIF_ICON | shellapi::NIF_TIP | shellapi::NIF_MESSAGE | shellapi::NIF_SHOWTIP;
        t.as_inner_mut().cfg.uCallbackMessage = MESSAGE;
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
        if let Some(items) = menu {
            unsafe {
                let menu = winuser::CreatePopupMenu();
                common::make_menu(menu, items, &mut t.as_inner_mut().menu.1);
                t.as_inner_mut().menu.0 = menu;
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
            if !self.menu.0.is_null() {
                winuser::DeleteMenu(self.menu.0, 0, 0);
                if shellapi::Shell_NotifyIconW(shellapi::NIM_DELETE, &mut self.cfg) == minwindef::FALSE {
                    common::log_error();
                }
            }
        }
    }
}

default_impls_as!(Tray);
