use crate::common::{self, *};

const CLASS_ID: &str = "static";

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type Text = AMember<AControl<AText<WindowsText>>>;

#[repr(C)]
pub struct WindowsText {
    base: common::WindowsControlBase<Text>,
    text: String,
}

impl HasLabelInner for WindowsText {
    fn label(&self, _base: &MemberBase) -> Cow<str> {
        Cow::Borrowed(self.text.as_ref())
    }
    fn set_label(&mut self, _base: &mut MemberBase, label: Cow<str>) {
        self.text = label.into();
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            let control_name = OsStr::new(&self.text).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
            unsafe {
                winuser::SetWindowTextW(self.base.hwnd, control_name.as_ptr());
            }
            self.base.invalidate();
        }
    }
}
impl<O: controls::Text> NewTextInner<O> for WindowsText {
    fn with_uninit(_: &mut mem::MaybeUninit<O>) -> Self {
        WindowsText {
            base: common::WindowsControlBase::with_handler(Some(handler::<O>)),
            text: String::new(),
        }
    }
}
impl TextInner for WindowsText {
    fn with_text<S: AsRef<str>>(text: S) -> Box<dyn controls::Text> {
        let mut b: Box<mem::MaybeUninit<Text>> = Box::new_uninit();
        let mut ab = AMember::with_inner(
            AControl::with_inner(
                AText::with_inner(
                    <Self as NewTextInner<Text>>::with_uninit(b.as_mut()),
                )
            ),
        );
        controls::HasLabel::set_label(&mut ab, text.as_ref().into());
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
}

impl ControlInner for WindowsText {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        self.base.hwnd = unsafe { parent.native_id() as windef::HWND }; // required for measure, as we don't have own hwnd yet
        let (w, h, _) = self.measure(member, control, pw, ph);
        self.base.create_control_hwnd(x as i32, y as i32, w as i32, h as i32, self.base.hwnd, 0, WINDOW_CLASS.as_ptr(), self.text.as_str(), 
            winuser::WS_TABSTOP | winuser::SS_NOPREFIX, selfptr);
    }
    fn on_removed_from_container(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        self.base.destroy_control_hwnd();
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
        use plygui_api::markup::MEMBER_TYPE_BUTTON;
        fill_from_markup_base!(self, member, markup, registry, Text, [MEMBER_TYPE_BUTTON]);
        fill_from_markup_label!(self, member, markup);
    }
}

impl HasLayoutInner for WindowsText {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl HasSizeInner for WindowsText {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        true
    }
}
impl HasVisibilityInner for WindowsText {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl HasNativeIdInner for WindowsText {
    type Id = common::Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl MemberInner for WindowsText {}

impl Drawable for WindowsText {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;

        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut label_size: windef::SIZE = unsafe { mem::zeroed() };
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        if label_size.cx < 1 {
                            let label = OsStr::new(self.text.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
                            unsafe {
                                wingdi::GetTextExtentPointW(winuser::GetDC(self.base.hwnd), label.as_ptr(), self.text.len() as i32, &mut label_size);
                            }
                        }
                        label_size.cx as i32
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.cy < 1 {
                            let label = OsStr::new(self.text.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
                            unsafe {
                                wingdi::GetTextExtentPointW(winuser::GetDC(self.base.hwnd), label.as_ptr(), self.text.len() as i32, &mut label_size);
                            }
                        }
                        label_size.cy as i32
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
impl Spawnable for WindowsText {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_text("").into_control()
    }
}

unsafe extern "system" fn handler<T: controls::Text>(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let text: &mut Text = mem::transmute(param);
            text.call_on_size::<T>(width, height);
            return 0;
        }
        /*winuser::WM_PAINT => {
            use crate::plygui_api::controls::HasLabel;
            
            let text: &mut Text = mem::transmute(param);
            let mut ps: winuser::PAINTSTRUCT = mem::zeroed();
            let hdc = winuser::BeginPaint(hwnd, &mut ps);
            let mut text = common::str_to_wchar(text.label());
            let mut rc = common::window_rect(hwnd);
            
            wingdi::SetBkMode(hdc, wingdi::TRANSPARENT as i32);
            winuser::DrawTextW(hdc, text.as_mut_ptr(), text.len() as i32, &mut rc, winuser::DT_CENTER | winuser::DT_VCENTER);
            winuser::EndPaint(hwnd, &mut ps);
            return 0;
        }
        winuser::WM_NCCALCSIZE => {
            match wparam as i32 {
                minwindef::TRUE => {
                    let nccalc: &mut winuser::NCCALCSIZE_PARAMS = mem::transmute(lparam);
                    //println!("{:?}", nccalc);
                }
                minwindef::FALSE => {
                    let rect: &mut windef::RECT = mem::transmute(lparam);
                    rect.left += DEFAULT_PADDING;
                    rect.top += DEFAULT_PADDING;
                    rect.right -= DEFAULT_PADDING;
                    rect.bottom -= DEFAULT_PADDING;
                }
                _ => {}
            }
        }*/
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}
