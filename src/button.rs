use super::common::*;
use super::*;

const CLASS_ID: &str = "Button";

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type Button = Member<Control<WindowsButton>>;

#[repr(C)]
pub struct WindowsButton {
    base: common::WindowsControlBase<Button>,
    label: String,
    h_left_clicked: Option<callbacks::Click>,
}

impl HasLabelInner for WindowsButton {
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

impl ClickableInner for WindowsButton {
    fn on_click(&mut self, handle: Option<callbacks::Click>) {
        self.h_left_clicked = handle;
    }
}

impl ButtonInner for WindowsButton {
    fn with_label(label: &str) -> Box<Button> {
        let b: Box<Button> = Box::new(Member::with_inner(
            Control::with_inner(
                WindowsButton {
                    base: common::WindowsControlBase::new(),
                    h_left_clicked: None,
                    label: label.to_owned(),
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        b
    }
}

impl ControlInner for WindowsButton {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
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
                self.label.as_str(),
                winuser::BS_PUSHBUTTON | winuser::WS_TABSTOP,
                selfptr,
                Some(handler),
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
    }
    fn on_removed_from_container(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, _: &controls::Container) {
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, Some(handler));
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }
    fn parent(&self) -> Option<&controls::Member> {
        self.base.parent().map(|p| p.as_member())
    }
    fn parent_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.parent_mut().map(|p| p.as_member_mut())
    }
    fn root(&self) -> Option<&controls::Member> {
        self.base.root().map(|p| p.as_member())
    }
    fn root_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.root_mut().map(|p| p.as_member_mut())
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_BUTTON;
        fill_from_markup_base!(self, member, markup, registry, Button, [MEMBER_TYPE_BUTTON]);
        fill_from_markup_label!(self, member, markup);
        fill_from_markup_callbacks!(self, markup, registry, [on_click => plygui_api::callbacks::Click]);
    }
}

impl HasLayoutInner for WindowsButton {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            self.base.invalidate();
        }
    }
}

impl MemberInner for WindowsButton {
    type Id = common::Hwnd;

    fn size(&self) -> (u16, u16) {
        self.base.size()
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            unsafe {
                winuser::ShowWindow(self.base.hwnd, if base.visibility == types::Visibility::Visible { winuser::SW_SHOW } else { winuser::SW_HIDE });
            }
            self.base.invalidate();
        }
    }
    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl Drawable for WindowsButton {
    fn draw(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
        self.base.draw(coords);
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = self.base.measured_size;

        self.base.measured_size = match member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut label_size: windef::SIZE = unsafe { mem::zeroed() };
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        if label_size.cx < 1 {
                            let label = OsStr::new(self.label.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
                            unsafe {
                                wingdi::GetTextExtentPointW(winuser::GetDC(self.base.hwnd), label.as_ptr(), self.label.len() as i32, &mut label_size);
                            }
                        }
                        label_size.cx as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.cy < 1 {
                            let label = OsStr::new(self.label.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
                            unsafe {
                                wingdi::GetTextExtentPointW(winuser::GetDC(self.base.hwnd), label.as_ptr(), self.label.len() as i32, &mut label_size);
                            }
                        }
                        label_size.cy as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                (cmp::max(0, w) as u16, cmp::max(0, h) as u16)
            }
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate()
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Button::with_label("").into_control()
}

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_LBUTTONDOWN => {
            let button: &mut Button = mem::transmute(param);
            if let Some(ref mut cb) = button.as_inner_mut().as_inner_mut().h_left_clicked {
                let mut button2: &mut Button = mem::transmute(param);
                (cb.as_mut())(button2);
            }
            return 0;
        }
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let button: &mut Button = mem::transmute(param);
            button.call_on_resize(width, height);
            return 0;
        }
        #[cfg(feature = "prettier")]
        winuser::WM_PAINT => {
            if aerize(hwnd).is_ok() {
                return 1;
            }
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}

#[cfg(feature = "prettier")]
unsafe fn aerize(hwnd: windef::HWND) -> Result<(),()> {
    let style = winuser::GetWindowLongPtrW(hwnd, winuser::GWL_STYLE);
    
    let mut ps: winuser::PAINTSTRUCT = mem::zeroed();
    let hdc = winuser::BeginPaint(hwnd, &mut ps);
    
    if hdc.is_null() { return Err(()); }
        
    let mut client_rect = common::window_rect(hwnd)?;
    let mut hdc_paint: windef::HDC = ptr::null_mut();
    let mut paint_params: uxtheme::BP_PAINTPARAMS = mem::zeroed();
    paint_params.cbSize = mem::size_of::<uxtheme::BP_PAINTPARAMS>() as u32;
    paint_params.dwFlags = uxtheme::BPPF_ERASE;
    
    let theme = uxtheme::OpenThemeData(hwnd, WINDOW_CLASS.as_ptr());
    if theme.is_null() { 
        winuser::EndPaint(hwnd, &mut ps);
        return Err(()); 
    }
    
    let buff_paint = uxtheme::BeginBufferedPaint(hdc, &mut client_rect, uxtheme::BPBF_TOPDOWNDIB, &mut paint_params, &mut hdc_paint);
    if hdc_paint.is_null() {
        common::log_error();
        uxtheme::CloseThemeData(theme);
        winuser::EndPaint(hwnd, &mut ps);
        return Err(());
    }
    
     if wingdi::PatBlt(hdc_paint, 0, 0, client_rect.right - client_rect.left, client_rect.bottom - client_rect.top, wingdi::BLACKNESS) < 0 {
        uxtheme::CloseThemeData(theme);
        winuser::EndPaint(hwnd, &mut ps);
        return Err(());
    }
    if uxtheme::BufferedPaintSetAlpha(buff_paint, &mut ps.rcPaint, 0) != winerror::S_OK {
        uxtheme::CloseThemeData(theme);
        winuser::EndPaint(hwnd, &mut ps);
        return Err(());
    }
    
    let check_state = winuser::SendMessageW(hwnd, winuser::BM_GETCHECK, 0, 0);
    let mut rect = common::window_rect(hwnd)?;
    let mut pt: windef::POINT = mem::zeroed();
    winuser::GetCursorPos(&mut pt);
    let hot = winuser::PtInRect(&rect, pt) > 0;
    let focus = winuser::GetFocus() == hwnd;
    let part_id = 1; // BP_PUSHBUTTON
    
    let state = common::aero::state_from_button_state(style as u32, hot, focus, check_state as usize, part_id, winuser::GetCapture() == hwnd);
    let paint_rect = client_rect.clone();
    
    if uxtheme::DrawThemeBackground(theme, hdc_paint, part_id, state, &paint_rect, ptr::null_mut())!= winerror::S_OK {
        uxtheme::CloseThemeData(theme);
        winuser::EndPaint(hwnd, &mut ps);
        return Err(());
    }                        
    if uxtheme::GetThemeBackgroundContentRect(theme, hdc_paint, part_id, state, &paint_rect, &mut rect) != winerror::S_OK {
        uxtheme::CloseThemeData(theme);
        winuser::EndPaint(hwnd, &mut ps);
        return Err(());
    }
                            
    let mut dtt_opts: uxtheme::DTTOPTS = mem::zeroed();
    dtt_opts.dwSize = mem::size_of::<uxtheme::DTTOPTS>() as u32;
    dtt_opts.dwFlags = uxtheme::DTT_COMPOSITED;
    dtt_opts.crText = wingdi::RGB(0xFF, 0xFF, 0xFF);
    dtt_opts.iGlowSize = common::aero::glow_size(ptr::null()).map_err(|e| {
        uxtheme::CloseThemeData(theme);
        e
    })?;
    
    let mut font_old = winuser::SendMessageW(hwnd, winuser::WM_GETFONT, 0, 0) as windef::HFONT;
    if !font_old.is_null() {
        font_old = wingdi::SelectObject(hdc_paint, font_old as *mut c_void) as windef::HFONT;
    }          

    let mut len = winuser::GetWindowTextLengthW(hwnd);
    if len < 0 {
        uxtheme::CloseThemeData(theme);
        winuser::EndPaint(hwnd, &mut ps);
        return Err(());
    }
    
    len += 5;
    let mut text: Vec<u16> = Vec::with_capacity(len as usize);
    
    len = winuser::GetWindowTextW(hwnd, text.as_mut_slice().as_mut_ptr(), len);
    if len < 0 {
        uxtheme::CloseThemeData(theme);
        winuser::EndPaint(hwnd, &mut ps);
        return Err(());
    }

    let flags = winuser::DT_SINGLELINE | winuser::DT_CENTER | winuser::DT_VCENTER;
    
    if uxtheme::DrawThemeTextEx(theme, hdc_paint, part_id, state, text.as_mut_slice().as_mut_ptr(), -1, flags, &mut rect, &dtt_opts) != winerror::S_OK {
        uxtheme::CloseThemeData(theme);
        winuser::EndPaint(hwnd, &mut ps);
        return Err(());
    }

    if focus {
        let mut draw_rect = client_rect.clone();
        if winuser::InflateRect(&mut draw_rect, -3, -3) < 0 {
            uxtheme::CloseThemeData(theme);
            winuser::EndPaint(hwnd, &mut ps);
            return Err(());
        }
        winuser::DrawFocusRect(hdc_paint, &mut draw_rect);
    }
                                            
    if !font_old.is_null() {
        wingdi::SelectObject(hdc_paint, font_old as *mut c_void);
    }
    uxtheme::EndBufferedPaint(buff_paint, minwindef::TRUE);
    uxtheme::CloseThemeData(theme);
    winuser::EndPaint(hwnd, &mut ps);
    
    Ok(())
}

impl_all_defaults!(Button);
