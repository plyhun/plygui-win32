use crate::common::{self, *};

const CLASS_ID: &str = ::winapi::um::commctrl::PROGRESS_CLASS;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type ProgressBar = Member<Control<WindowsProgressBar>>;

#[repr(C)]
pub struct WindowsProgressBar {
    base: common::WindowsControlBase<ProgressBar>,
    progress: types::Progress,
}

impl ProgressBarInner for WindowsProgressBar {
    fn with_progress(progress: types::Progress) -> Box<ProgressBar> {
        let b: Box<ProgressBar> = Box::new(Member::with_inner(
            Control::with_inner(
                WindowsProgressBar {
                    base: common::WindowsControlBase::new(),
                    progress: progress,
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        b
    }
}

impl HasProgressInner for WindowsProgressBar {
    fn progress(&self, _base: &MemberBase) -> types::Progress {
        self.progress.clone()
    }
    fn set_progress(&mut self, _base: &mut MemberBase, arg0: types::Progress) {
        self.progress = arg0;
        if !self.base.hwnd.is_null() {
            let mut style = unsafe { winuser::GetWindowLongPtrW(self.base.hwnd, winuser::GWL_STYLE) };
            match self.progress {
                types::Progress::Undefined => unsafe {
                    style |= commctrl::PBS_MARQUEE as isize;
                    winuser::SetWindowLongPtrW(self.base.hwnd, winuser::GWL_STYLE, style);
                    winuser::SendMessageW(self.base.hwnd, commctrl::PBM_SETMARQUEE, 1, 0); 
                },
                types::Progress::Value(current, total) => unsafe {
                    style &= !commctrl::PBS_MARQUEE as isize;
                    winuser::SetWindowLongPtrW(self.base.hwnd, winuser::GWL_STYLE, style);
                    winuser::SendMessageW(self.base.hwnd, commctrl::PBM_SETSTATE, commctrl::PBST_NORMAL as usize, 0);
                    winuser::SendMessageW(self.base.hwnd, commctrl::PBM_SETRANGE32, 0, total as isize);
                    winuser::SendMessageW(self.base.hwnd, commctrl::PBM_SETPOS, current as usize, 0);
                },
                types::Progress::None => unsafe {
                	style &= !commctrl::PBS_MARQUEE as isize;
                    winuser::SetWindowLongPtrW(self.base.hwnd, winuser::GWL_STYLE, style);
                    winuser::SendMessageW(self.base.hwnd, commctrl::PBM_SETSTATE, commctrl::PBST_PAUSED as usize, 0);
                    winuser::SendMessageW(self.base.hwnd, commctrl::PBM_SETRANGE32, 0, 0);
                    winuser::SendMessageW(self.base.hwnd, commctrl::PBM_SETPOS, 0, 0);
                }
            }
        }
    }
}

impl ControlInner for WindowsProgressBar {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (w, h, _) = self.measure(member, control, pw, ph);
            common::create_control_hwnd(
                x as i32,
                y as i32,
                w as i32,
                h as i32,
                self.base.hwnd,
                0,
                WINDOW_CLASS.as_ptr(),
                "",
                winuser::BS_PUSHBUTTON | winuser::WS_TABSTOP,
                selfptr,
                Some(handler),
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
        self.set_progress(member, self.progress.clone());
    }
    fn on_removed_from_container(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, Some(handler));
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }
    fn parent(&self) -> Option<&dyn controls::Member> {
        self.base.parent().map(|p| p.as_member())
    }
    fn parent_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.parent_mut().map(|p| p.as_member_mut())
    }
    fn root(&self) -> Option<&dyn controls::Member> {
        self.base.root().map(|p| p.as_member())
    }
    fn root_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.root_mut().map(|p| p.as_member_mut())
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_PROGRESS_BAR;
        fill_from_markup_base!(self, member, markup, registry, ProgressBar, [MEMBER_TYPE_PROGRESS_BAR]);
    }
}

impl HasLayoutInner for WindowsProgressBar {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl HasNativeIdInner for WindowsProgressBar {
    type Id = common::Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl HasSizeInner for WindowsProgressBar {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for WindowsProgressBar {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for WindowsProgressBar {}

impl Drawable for WindowsProgressBar {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;

        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        common::DEFAULT_HEIGHT / 2 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        common::DEFAULT_HEIGHT / 2 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                (cmp::max(0, w) as u16, cmp::max(0, h) as u16)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate()
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<dyn controls::Control> {
    ProgressBar::with_progress(types::Progress::Undefined).into_control()
}

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let progress_bar: &mut ProgressBar = mem::transmute(param);
            progress_bar.call_on_size(width, height);
            return 0;
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}

default_impls_as!(ProgressBar);
