use super::common::*;
use super::*;

const CLASS_ID: &str = "Button";

lazy_static! {
    pub static ref WINDOW_CLASS_GBOX: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub type Frame = Member<Control<SingleContainer<WindowsFrame>>>;

#[repr(C)]
pub struct WindowsFrame {
    base: common::WindowsControlBase<Frame>,
    hwnd_gbox: windef::HWND,
    label: String,
    label_padding: i32,
    child: Option<Box<controls::Control>>,
}

impl FrameInner for WindowsFrame {
    fn with_label(label: &str) -> Box<Frame> {
        let b = Box::new(Member::with_inner(
            Control::with_inner(
                SingleContainer::with_inner(
                    WindowsFrame {
                        base: common::WindowsControlBase::new(),
                        child: None,
                        hwnd_gbox: 0 as windef::HWND,
                        label: label.to_owned(),
                        label_padding: 0,
                    },
                    (),
                ),
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        b
    }
}

impl HasLayoutInner for WindowsFrame {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            unsafe { winuser::RedrawWindow(self.hwnd_gbox, ptr::null_mut(), ptr::null_mut(), winuser::RDW_INVALIDATE | winuser::RDW_UPDATENOW) };
            self.base.invalidate();
        }
    }
    fn layout_margin(&self, _member: &MemberBase) -> layout::BoundarySize {
        layout::BoundarySize::Distinct(DEFAULT_PADDING, DEFAULT_PADDING + self.label_padding, DEFAULT_PADDING, DEFAULT_PADDING)
    }
}

impl HasLabelInner for WindowsFrame {
    fn label<'a>(&'a self) -> ::std::borrow::Cow<'a, str> {
        Cow::Borrowed(self.label.as_ref())
    }
    fn set_label(&mut self, base: &mut MemberBase, label: &str) {
        self.label = label.into();
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            let control_name = OsStr::new(&self.label).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
            unsafe {
                winuser::SetWindowTextW(self.base.hwnd, control_name.as_ptr());
            }
        }
        self.on_layout_changed(base);
    }
}

impl SingleContainerInner for WindowsFrame {
    fn set_child(&mut self, base: &mut MemberBase, child: Option<Box<controls::Control>>) -> Option<Box<controls::Control>> {
        let mut old = self.child.take();
        if let Some(old) = old.as_mut() {
            if !self.base.hwnd.is_null() {
                old.on_removed_from_container(self.base.as_outer_mut());
            }
        }
        self.child = child;

        if self.child.is_some() {
            if !self.base.hwnd.is_null() {
                let (w, h) = self.size();
                if let Some(new) = self.child.as_mut() {
                    new.as_mut().on_added_to_container(
                        self.base.as_outer_mut(),
                        0,
                        0,
                        cmp::max(0, w as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16,
                        cmp::max(0, h as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16,
                    );
                }
            }
        }
        self.on_layout_changed(base);

        old
    }
    fn child(&self) -> Option<&controls::Control> {
        self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut controls::Control> {
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
}

impl ContainerInner for WindowsFrame {
    fn find_control_by_id_mut(&mut self, id: ids::Id) -> Option<&mut controls::Control> {
        if let Some(child) = self.child.as_mut() {
            if child.as_member().id() == id {
                Some(child.as_mut())
            } else if let Some(c) = child.is_container_mut() {
                c.find_control_by_id_mut(id)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn find_control_by_id(&self, id: ids::Id) -> Option<&controls::Control> {
        if let Some(child) = self.child.as_ref() {
            if child.as_member().id() == id {
                Some(child.as_ref())
            } else if let Some(c) = child.is_container() {
                c.find_control_by_id(id)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl ControlInner for WindowsFrame {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &controls::Container, px: i32, py: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        let (hwnd, hwnd_gbox, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (width, height, _) = self.measure(member, control, pw, ph);
            let hwnd_gbox = winuser::CreateWindowExW(
                0,
                WINDOW_CLASS_GBOX.as_ptr(),
                OsStr::new(self.label.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>().as_ptr(),
                winuser::BS_GROUPBOX | winuser::WS_CHILD | winuser::WS_VISIBLE,
                px,
                py,
                width as i32,
                height as i32,
                self.base.hwnd,
                ptr::null_mut(),
                common::hinstance(),
                ptr::null_mut(),
            );
            common::set_default_font(hwnd_gbox);
            let (hwnd, id) = common::create_control_hwnd(
                px + DEFAULT_PADDING,
                py + self.label_padding + DEFAULT_PADDING,
                width as i32 - DEFAULT_PADDING - DEFAULT_PADDING,
                height as i32 - self.label_padding - DEFAULT_PADDING - DEFAULT_PADDING,
                self.base.hwnd,
                winuser::WS_EX_CONTROLPARENT,
                WINDOW_CLASS.as_ptr(),
                "",
                0,
                selfptr,
                None,
            );
            (hwnd, hwnd_gbox, id)
        };
        self.base.hwnd = hwnd;
        self.hwnd_gbox = hwnd_gbox;
        self.base.subclass_id = id;
        self.base.coords = Some((px, py));
        if let Some(ref mut child) = self.child {
            let self2: &mut Frame = unsafe { utils::base_to_impl_mut(member) };
            child.on_added_to_container(
                self2,
                0,
                0,
                cmp::max(0, self.base.measured_size.0 as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16,
                cmp::max(0, self.base.measured_size.1 as i32 - DEFAULT_PADDING - DEFAULT_PADDING - self.label_padding) as u16,
            );
        }
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &controls::Container) {
        if let Some(ref mut child) = self.child {
            let self2: &mut Frame = unsafe { utils::base_to_impl_mut(member) };
            child.on_removed_from_container(self2);
        }
        common::destroy_hwnd(self.hwnd_gbox, 0, None);
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.hwnd_gbox = 0 as windef::HWND;
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
        use plygui_api::markup::MEMBER_TYPE_FRAME;

        fill_from_markup_base!(self, member, markup, registry, Frame, [MEMBER_TYPE_FRAME]);
        fill_from_markup_label!(self, member, markup);
        fill_from_markup_child!(self, member, markup, registry);
    }
}

impl MemberInner for WindowsFrame {
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
            self.on_layout_changed(base);
        }
    }
    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl Drawable for WindowsFrame {
    fn draw(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.base.coords = coords;
        }
        if let Some((x, y)) = self.base.coords {
            unsafe {
                winuser::SetWindowPos(self.base.hwnd, ptr::null_mut(), x + DEFAULT_PADDING, y + DEFAULT_PADDING + self.label_padding, self.base.measured_size.0 as i32 - DEFAULT_PADDING - DEFAULT_PADDING, self.base.measured_size.1 as i32 - self.label_padding - DEFAULT_PADDING - DEFAULT_PADDING, 0);
                winuser::SetWindowPos(self.hwnd_gbox, ptr::null_mut(), x, y, self.base.measured_size.0 as i32, self.base.measured_size.1 as i32, 0);
            }
            if let Some(ref mut child) = self.child {
                child.draw(Some((0, 0)));
            }
        }
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let old_size = self.base.measured_size;
        let hp = DEFAULT_PADDING + DEFAULT_PADDING;
        let vp = DEFAULT_PADDING + DEFAULT_PADDING;
        self.base.measured_size = match member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut measured = false;
                let w = match control.layout.width {
                    layout::Size::Exact(w) => w,
                    layout::Size::MatchParent => parent_width,
                    layout::Size::WrapContent => {
                        let mut w = 0;
                        if let Some(ref mut child) = self.child {
                            self.label_padding = update_label_size(self.label.as_str(), self.base.hwnd);
                            let (cw, _, _) = child.measure(max(0, parent_width as i32 - hp) as u16, max(0, parent_height as i32 - vp - self.label_padding) as u16);
                            w += cw as i32;
                            measured = true;
                        }
                        max(0, w as i32 + hp) as u16
                    }
                };
                let h = match control.layout.height {
                    layout::Size::Exact(h) => h,
                    layout::Size::MatchParent => parent_height,
                    layout::Size::WrapContent => {
                        let mut h = 0;
                        if let Some(ref mut child) = self.child {
                            let ch = if measured {
                                child.size().1
                            } else {
                                self.label_padding = update_label_size(self.label.as_str(), self.base.hwnd);
                                let (_, ch, _) = child.measure(max(0, parent_width as i32 - hp) as u16, max(0, parent_height as i32 - vp - self.label_padding) as u16);
                                ch
                            };
                            h += ch as i32;
                            h += self.label_padding;
                        }
                        max(0, h as i32 + vp) as u16
                    }
                };
                (w, h)
            }
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        unsafe { winuser::RedrawWindow(self.hwnd_gbox, ptr::null_mut(), ptr::null_mut(), winuser::RDW_INVALIDATE | winuser::RDW_UPDATENOW) };
        self.base.invalidate();
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Frame::with_label("").into_control()
}

fn update_label_size(label: &str, hwnd: windef::HWND) -> i32 {
    let mut label_size: windef::SIZE = unsafe { mem::zeroed() };
    let label = OsStr::new(label).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    unsafe {
        wingdi::GetTextExtentPointW(winuser::GetDC(hwnd), label.as_ptr(), label.len() as i32, &mut label_size);
    }
    label_size.cy as i32 / 2
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32Frame").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    let class = winuser::WNDCLASSEXW {
        cbSize: mem::size_of::<winuser::WNDCLASSEXW>() as minwindef::UINT,
        style: 0,
        lpfnWndProc: Some(whandler),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: common::hinstance(),
        hIcon: winuser::LoadIconW(ptr::null_mut(), winuser::IDI_APPLICATION),
        hCursor: winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_ARROW),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: ptr::null_mut(),
    };
    winuser::RegisterClassExW(&class);
    class_name
}

unsafe extern "system" fn whandler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        if winuser::WM_CREATE == msg {
            let cs: &mut winuser::CREATESTRUCTW = mem::transmute(lparam);
            winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, cs.lpCreateParams as isize);
        }
        return winuser::DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    match msg {
        winuser::WM_SIZE => {
            let mut width = lparam as u16;
            let mut height = (lparam >> 16) as u16;
            let mut frame: &mut Frame = mem::transmute(ww);
            
            if let Some(ref mut child) = frame.as_inner_mut().as_inner_mut().as_inner_mut().child {
                child.measure(cmp::max(0, width as i32) as u16, cmp::max(0, height as i32) as u16);
                child.draw(Some((0, 0)));
            }
            frame.call_on_resize(width, height);
            return 0;
        }
        #[cfg(feature = "prettier")]
        winuser::WM_PAINT => {
            let mut frame: &mut Frame = mem::transmute(ww);
            if aerize(frame.as_inner_mut().as_inner_mut().as_inner_mut().hwnd_gbox).is_ok() && common::aero::aerize(hwnd, true).is_ok() {
                return 1;
            }
        }
        _ => {}
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
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
    let mut exclusion_rect = client_rect.clone();
    
    let theme = uxtheme::OpenThemeData(hwnd, WINDOW_CLASS_GBOX.as_ptr());
    if theme.is_null() { 
        winuser::EndPaint(hwnd, &mut ps);
        return Err(()); 
    }
    
    let mut font_old = winuser::SendMessageW(hwnd, winuser::WM_GETFONT, 0, 0) as windef::HFONT;
    if !font_old.is_null() {
        font_old = wingdi::SelectObject(hdc, font_old as *mut c_void) as windef::HFONT;
    }
    
    let mut draw_rect = client_rect.clone();
    let dw_flags = winuser::DT_SINGLELINE;
    
    // Reusing WINDOW_CLASS_GBOX for getting text height, because it has a capital letter
    winuser::DrawTextW(hdc, WINDOW_CLASS_GBOX.as_ptr(), -1, &mut draw_rect, dw_flags | winuser::DT_CALCRECT);
    
    if !font_old.is_null() {
        wingdi::SelectObject(hdc, font_old as *mut c_void);
        //font_old = ptr::null_mut();
    }
    
    if winuser::InflateRect(&mut exclusion_rect, -1, -1 * (draw_rect.bottom - draw_rect.top)) < 0 { 
        common::log_error();
        uxtheme::CloseThemeData(theme);
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
    
    font_old = winuser::SendMessageW(hwnd, winuser::WM_GETFONT, 0, 0) as windef::HFONT;
    if !font_old.is_null() {
        font_old = wingdi::SelectObject(hdc_paint, font_old as *mut c_void) as windef::HFONT;
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
    
    let part = 4i32; // BP_GROUPBOX
    let state = common::aero::state_from_button_state(style as u32, false, false, 0, part, false);
    
    let mut dtt_opts: uxtheme::DTTOPTS = mem::zeroed();
    dtt_opts.dwSize = mem::size_of::<uxtheme::DTTOPTS>() as u32;
    dtt_opts.dwFlags = uxtheme::DTT_COMPOSITED | uxtheme::DTT_GLOWSIZE;
    dtt_opts.crText = wingdi::RGB(0xFF, 0xFF, 0xFF);
    dtt_opts.iGlowSize = common::aero::glow_size(ptr::null()).map_err(|e| {
        uxtheme::CloseThemeData(theme);
        e
    })?;
    
    {
        let original = wingdi::SelectObject(hdc_paint, wingdi::GetStockObject(wingdi::DC_PEN as i32));
        
        let mut cr = common::aero::edit_border_color(hwnd).map_err(|e| {
            uxtheme::CloseThemeData(theme);
            e
        })?;
        cr |= 0xFF000000;
        
        wingdi::SetDCPenColor(hdc_paint, cr);
        
        let iy = (draw_rect.bottom - draw_rect.top) / 2;
        wingdi::Rectangle(hdc_paint, client_rect.left, client_rect.top + iy, client_rect.right - 1, client_rect.bottom - iy - 1);
        
        wingdi::SelectObject(hdc_paint, original);
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
    
    //let ix = draw_rect.right - draw_rect.left;
    draw_rect = client_rect.clone();
    draw_rect.left += common::DEFAULT_PADDING; //ix; // title left offset
    winuser::DrawTextW(hdc_paint, text.as_mut_slice().as_mut_ptr(), -1,  &mut draw_rect, dw_flags | winuser::DT_CALCRECT);
    wingdi::PatBlt(hdc_paint, draw_rect.left, draw_rect.top, draw_rect.right - draw_rect.left + 3, draw_rect.bottom - draw_rect.top, wingdi::BLACKNESS);
    draw_rect.left += 1;
    draw_rect.right += 1;
    
    if uxtheme::DrawThemeTextEx(theme, hdc_paint, part, state, text.as_mut_slice().as_mut_ptr(), -1, dw_flags, &mut draw_rect, &mut dtt_opts) != winerror::S_OK {
        common::log_error();
        uxtheme::CloseThemeData(theme);
        winuser::EndPaint(hwnd, &mut ps);
        return Err(());
    }
    if !font_old.is_null() {
        wingdi::SelectObject(hdc_paint, font_old as *mut c_void);
    }
    uxtheme::EndBufferedPaint(buff_paint, minwindef::TRUE);
    uxtheme::CloseThemeData(theme);
    winuser::EndPaint(hwnd, &mut ps);
    
    Ok(())
}

impl_all_defaults!(Frame);
