use crate::common::{self, *};

const DEFAULT_BOUND: i32 = DEFAULT_PADDING * 2;
const DEFAULT_DIVIDER_PADDING: i32 = DEFAULT_PADDING * 8;
const HALF_DIVIDER_PADDING: i32 = DEFAULT_DIVIDER_PADDING / 2;
const HALF_BOUND: i32 = DEFAULT_BOUND / 2;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub type Splitted = AMember<AControl<AContainer<AMultiContainer<ASplitted<WindowsSplitted>>>>>;

#[repr(C)]
pub struct WindowsSplitted {
    base: common::WindowsControlBase<Splitted>,

    orientation: layout::Orientation,

    splitter: f32,
    moving: bool,
    cursor: windef::HCURSOR,
    default_cursor: windef::HCURSOR,

    first: Box<dyn controls::Control>,
    second: Box<dyn controls::Control>,
}

impl WindowsSplitted {
    fn children_sizes(&self, base: &ControlBase) -> (u16, u16) {
        let (w, h) = base.measured;
        let target = match self.orientation {
            layout::Orientation::Horizontal => w,
            layout::Orientation::Vertical => h,
        };
        (
            utils::coord_to_size((target as f32 * self.splitter) as i32 - DEFAULT_PADDING - HALF_BOUND),
            utils::coord_to_size((target as f32 * (1.0 - self.splitter)) as i32 - DEFAULT_PADDING - HALF_BOUND),
        )
    }
    fn draw_divider(&mut self, base: &ControlBase) {
        let (w, h) = base.measured;
        let (x0, y0, x1, y1) = match self.orientation {
            layout::Orientation::Vertical => {
                let coord = (h as f32 * self.splitter) as i32;
                (HALF_DIVIDER_PADDING, coord, w as i32 - HALF_DIVIDER_PADDING, coord)
            }
            layout::Orientation::Horizontal => {
                let coord = (w as f32 * self.splitter) as i32;
                (coord, HALF_DIVIDER_PADDING, coord, h as i32 - HALF_DIVIDER_PADDING)
            }
        };

        unsafe {
            let color = winuser::GetSysColor(winuser::COLOR_ACTIVEBORDER);

            let mut ps: winuser::PAINTSTRUCT = mem::zeroed();
            let dc = winuser::BeginPaint(self.base.hwnd, &mut ps);
            wingdi::SelectObject(dc, wingdi::GetStockObject(wingdi::DC_PEN as i32));
            wingdi::SetDCPenColor(dc, color);
            wingdi::SelectObject(dc, wingdi::GetStockObject(wingdi::DC_BRUSH as i32));
            wingdi::SetDCBrushColor(dc, color);

            wingdi::MoveToEx(dc, x0, y0, ptr::null_mut());
            wingdi::LineTo(dc, x1, y1);

            winuser::EndPaint(self.base.hwnd, &ps);
        }
    }
    fn draw_children(&mut self) {
        let mut x = DEFAULT_PADDING;
        let mut y = DEFAULT_PADDING;
        for ref mut child in [self.first.as_mut(), self.second.as_mut()].iter_mut() {
            child.draw(Some((x, y)));
            let (xx, yy) = child.size();
            match self.orientation {
                layout::Orientation::Horizontal => {
                    x += xx as i32;
                    x += DEFAULT_BOUND;
                }
                layout::Orientation::Vertical => {
                    y += yy as i32;
                    y += DEFAULT_BOUND;
                }
            }
        }
    }
    fn reload_cursor(&mut self) {
        unsafe {
            if !self.cursor.is_null() && winuser::DestroyCursor(self.cursor) == 0 {
                common::log_error();
            }
        }
        self.cursor = unsafe {
            winuser::LoadCursorW(
                ptr::null_mut(),
                match self.orientation {
                    layout::Orientation::Horizontal => winuser::IDC_SIZEWE,
                    layout::Orientation::Vertical => winuser::IDC_SIZENS,
                },
            )
        };
    }
    fn update_children_layout(&mut self, base: &ControlBase) {
        if self.base.hwnd.is_null() {
            return;
        }

        let orientation = self.orientation;
        let (first_size, second_size) = self.children_sizes(base);
        let (width, height) = base.measured;
        for (size, child) in [(first_size, self.first.as_mut()), (second_size, self.second.as_mut())].iter_mut() {
            match orientation {
                layout::Orientation::Horizontal => {
                    child.measure(cmp::max(0, *size) as u16, cmp::max(0, height as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16);
                }
                layout::Orientation::Vertical => {
                    child.measure(cmp::max(0, width as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16, cmp::max(0, *size) as u16);
                }
            }
        }
    }
}
impl<O: controls::Splitted> NewSplittedInner<O> for WindowsSplitted {
    fn with_uninit_params(_: &mut mem::MaybeUninit<O>, first: Box<dyn controls::Control>, second: Box<dyn controls::Control>, orientation: layout::Orientation) -> Self {
        WindowsSplitted {
            base: common::WindowsControlBase::with_wndproc(Some(handler::<O>)),
            
            splitter: 0.5,
            cursor: ptr::null_mut(),
            default_cursor: unsafe {
	            winuser::LoadCursorW(
	                ptr::null_mut(),
	                winuser::IDC_ARROW,
	            )
	        },
            moving: false,

            first, second, orientation
        }
    }
}
impl SplittedInner for WindowsSplitted {
    fn with_content(first: Box<dyn controls::Control>, second: Box<dyn controls::Control>, orientation: layout::Orientation) -> Box<dyn controls::Splitted> {
        let mut b: Box<mem::MaybeUninit<Splitted>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AMultiContainer::with_inner(
                        ASplitted::with_inner(
                            <Self as NewSplittedInner<Splitted>>::with_uninit_params(b.as_mut(), first, second, orientation)
                        )
                    ),
                )
            ),
        );
        unsafe {
            b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
    fn set_splitter(&mut self, _member: &mut MemberBase, pos: f32) {
        self.splitter = pos;
        self.base.invalidate();
    }
    fn splitter(&self) -> f32 {
        self.splitter
    }
    fn first(&self) -> &dyn controls::Control {
        self.first.as_ref()
    }
    fn second(&self) -> &dyn controls::Control {
        self.second.as_ref()
    }
    fn first_mut(&mut self) -> &mut dyn controls::Control {
        self.first.as_mut()
    }
    fn second_mut(&mut self) -> &mut dyn controls::Control {
        self.second.as_mut()
    }
}

impl HasNativeIdInner for WindowsSplitted {
    type Id = common::Hwnd;

    fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl HasSizeInner for WindowsSplitted {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}
impl HasVisibilityInner for WindowsSplitted {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for WindowsSplitted {}

impl ControlInner for WindowsSplitted {
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
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, px: i32, py: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_container_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            control.measured = (pw, ph);
            let (width, height, _) = self.measure(member, control, pw, ph);
            common::create_control_hwnd(
                px as i32,
                py as i32,
                width as i32,
                height as i32,
                parent.native_id() as windef::HWND,
                winuser::WS_EX_CONTROLPARENT | winuser::WS_CLIPCHILDREN,
                WINDOW_CLASS.as_ptr(),
                "",
                0,
                selfptr,
                None,
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
        control.coords = Some((px as i32, py as i32));
        self.reload_cursor();
        //self.update_children_layout();

        let self2: &mut Splitted = unsafe { mem::transmute(selfptr) };
        let (first_size, second_size) = self.children_sizes(control);

        match self.orientation {
            layout::Orientation::Horizontal => {
                let h = utils::coord_to_size(control.measured.1 as i32 - DEFAULT_PADDING - DEFAULT_PADDING);
                self.first.on_added_to_container(self2, DEFAULT_PADDING, DEFAULT_PADDING, first_size, h);
                self.second.on_added_to_container(self2, DEFAULT_PADDING + DEFAULT_BOUND + first_size as i32, DEFAULT_PADDING, second_size, h);
            }
            layout::Orientation::Vertical => {
                let w = utils::coord_to_size(control.measured.0 as i32 - DEFAULT_PADDING - DEFAULT_PADDING);
                self.first.on_added_to_container(self2, DEFAULT_PADDING, DEFAULT_PADDING, w, first_size);
                self.second.on_added_to_container(self2, DEFAULT_PADDING, DEFAULT_PADDING + DEFAULT_BOUND + first_size as i32, w, second_size);
            }
        }
        //self.draw_divider(control);
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        let self2: &mut Splitted = unsafe { utils::base_to_impl_mut(member) };

        self.first.on_removed_from_container(self2);
        self.second.on_removed_from_container(self2);

        self.base.destroy_control_hwnd();
        self.cursor = ptr::null_mut();
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_SPLITTED;

        fill_from_markup_base!(self, member, markup, registry, Splitted, [MEMBER_TYPE_SPLITTED]);
        fill_from_markup_children!(self, member, markup, registry);
    }
}

impl HasLayoutInner for WindowsSplitted {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        //self.update_children_layout();
        self.base.invalidate();
    }
    fn layout_margin(&self, _member: &MemberBase) -> layout::BoundarySize {
        layout::BoundarySize::AllTheSame(DEFAULT_PADDING)
    }
}

impl ContainerInner for WindowsSplitted {
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        match arg {
            types::FindBy::Id(id) => {
                if self.first().as_member().id() == id {
                    return Some(self.first_mut());
                }
                if self.second().as_member().id() == id {
                    return Some(self.second_mut());
                }
            }
            types::FindBy::Tag(tag) => {
                if let Some(mytag) = self.first.as_member().tag() {
                    if tag == mytag {
                        return Some(self.first_mut());
                    }
                }
                if let Some(mytag) = self.second.as_member().tag() {
                    if tag == mytag {
                        return Some(self.second_mut());
                    }
                }
            }
        }

        let self2: &mut WindowsSplitted = unsafe { mem::transmute(self as *mut WindowsSplitted) }; // bck is stupid
        if let Some(c) = self.first_mut().is_container_mut() {
            let ret = c.find_control_mut(arg.clone());
            if ret.is_some() {
                return ret;
            }
        }
        if let Some(c) = self2.second_mut().is_container_mut() {
            let ret = c.find_control_mut(arg);
            if ret.is_some() {
                return ret;
            }
        }
        None
    }
    fn find_control<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn controls::Control> {
        match arg {
            types::FindBy::Id(id) => {
                if self.first().as_member().id() == id {
                    return Some(self.first());
                }
                if self.second().as_member().id() == id {
                    return Some(self.second());
                }
            }
            types::FindBy::Tag(tag) => {
                if let Some(mytag) = self.first.as_member().tag() {
                    if tag == mytag {
                        return Some(self.first.as_ref());
                    }
                }
                if let Some(mytag) = self.second.as_member().tag() {
                    if tag == mytag {
                        return Some(self.second.as_ref());
                    }
                }
            }
        }
        if let Some(c) = self.first().is_container() {
            let ret = c.find_control(arg.clone());
            if ret.is_some() {
                return ret;
            }
        }
        if let Some(c) = self.second().is_container() {
            let ret = c.find_control(arg);
            if ret.is_some() {
                return ret;
            }
        }
        None
    }
}

impl MultiContainerInner for WindowsSplitted {
    fn len(&self) -> usize {
        2
    }
    fn set_child_to(&mut self, _: &mut MemberBase, index: usize, mut child: Box<dyn controls::Control>) -> Option<Box<dyn controls::Control>> {
        match index {
            0 => {
                if !self.base.hwnd.is_null() {
                    let self2 = self.base.as_outer_mut();
                    let sizes = self.first.size();
                    self.first.on_removed_from_container(self2);
                    child.on_added_to_container(self2, DEFAULT_PADDING, DEFAULT_PADDING, sizes.0, sizes.1);
                }
                mem::swap(&mut self.first, &mut child);
            }
            1 => {
                if !self.base.hwnd.is_null() {
                    let self2 = self.base.as_outer_mut();

                    let mut x = DEFAULT_PADDING;
                    let mut y = DEFAULT_PADDING;

                    let (xx, yy) = self.first.size();
                    match self.orientation {
                        layout::Orientation::Horizontal => {
                            x += xx as i32;
                            x += DEFAULT_BOUND;
                        }
                        layout::Orientation::Vertical => {
                            y += yy as i32;
                            y += DEFAULT_BOUND;
                        }
                    }
                    let sizes = self.second.size();
                    self.second.on_removed_from_container(self2);
                    child.on_added_to_container(self2, x, y, sizes.0, sizes.1);
                }
                mem::swap(&mut self.second, &mut child);
            }
            _ => return None,
        }

        Some(child)
    }
    fn remove_child_from(&mut self, _: &mut MemberBase, _: usize) -> Option<Box<dyn controls::Control>> {
        None
    }
    fn child_at(&self, index: usize) -> Option<&dyn controls::Control> {
        match index {
            0 => Some(self.first()),
            1 => Some(self.second()),
            _ => None,
        }
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut dyn controls::Control> {
        match index {
            0 => Some(self.first_mut()),
            1 => Some(self.second_mut()),
            _ => None,
        }
    }
}

impl HasOrientationInner for WindowsSplitted {
    fn orientation(&self, _base: &MemberBase) -> layout::Orientation {
        self.orientation
    }
    fn set_orientation(&mut self, _base: &mut MemberBase, orientation: layout::Orientation) {
        if orientation != self.orientation {
            self.orientation = orientation;
            self.reload_cursor();
            self.base.invalidate();
        }
    }
}

impl Drawable for WindowsSplitted {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
        //self.draw_children();
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let orientation = self.orientation;
        let old_size = control.measured;
        let hp = DEFAULT_PADDING + DEFAULT_PADDING + if orientation == layout::Orientation::Horizontal { DEFAULT_BOUND } else { 0 };
        let vp = DEFAULT_PADDING + DEFAULT_PADDING + if orientation == layout::Orientation::Vertical { DEFAULT_BOUND } else { 0 };
        let (first_size, second_size) = self.children_sizes(control);
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut measured = false;
                let w = match control.layout.width {
                    layout::Size::Exact(w) => w,
                    layout::Size::MatchParent => parent_width,
                    layout::Size::WrapContent => {
                        let mut w = 0;
                        for (size, child) in [(first_size, self.first.as_mut()), (second_size, self.second.as_mut())].iter_mut() {
                            match orientation {
                                layout::Orientation::Horizontal => {
                                    let (cw, _, _) = child.measure(max(0, *size) as u16, max(0, parent_height as i32 - vp) as u16);
                                    w += cw;
                                }
                                layout::Orientation::Vertical => {
                                    let (cw, _, _) = child.measure(max(0, parent_width as i32 - hp) as u16, max(0, *size) as u16);
                                    w = max(w, cw);
                                }
                            }
                        }
                        measured = true;
                        max(0, w as i32 + hp) as u16
                    }
                };
                let h = match control.layout.height {
                    layout::Size::Exact(h) => h,
                    layout::Size::MatchParent => parent_height,
                    layout::Size::WrapContent => {
                        let mut h = 0;
                        for (size, child) in [(first_size, self.first.as_mut()), (second_size, self.second.as_mut())].iter_mut() {
                            let ch = if measured {
                                child.size().1
                            } else {
                                let (_, ch, _) = match orientation {
                                    layout::Orientation::Horizontal => child.measure(max(0, *size) as u16, max(0, parent_height as i32 - vp) as u16),
                                    layout::Orientation::Vertical => child.measure(max(0, parent_width as i32 - hp) as u16, max(0, *size) as u16),
                                };
                                ch
                            };
                            match orientation {
                                layout::Orientation::Horizontal => {
                                    h = max(h, ch);
                                }
                                layout::Orientation::Vertical => {
                                    h += ch;
                                }
                            }
                        }
                        max(0, h as i32 + vp) as u16
                    }
                };
                (w, h)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate()
    }
}
impl Spawnable for WindowsSplitted {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_content(super::text::Text::spawn(), super::text::Text::spawn(), layout::Orientation::Vertical).into_control()
    }
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32Splitted").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    let class = winuser::WNDCLASSEXW {
        cbSize: mem::size_of::<winuser::WNDCLASSEXW>() as minwindef::UINT,
        style: winuser::CS_DBLCLKS,
        lpfnWndProc: Some(window_handler),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: libloaderapi::GetModuleHandleW(ptr::null()),
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

unsafe extern "system" fn window_handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        if winuser::WM_CREATE == msg {
            let cs: &mut winuser::CREATESTRUCTW = mem::transmute(lparam);
            winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, cs.lpCreateParams as WinPtr);
        }
        return winuser::DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    
    let s: &mut Splitted = mem::transmute(ww);
    let s2: &mut Splitted = mem::transmute(ww);
    if let Some(_proc) = s.inner().inner().inner().inner().inner().base.proc_handler.as_proc() {
        _proc(s2, msg, wparam, lparam)
    } else {
        winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

unsafe extern "system" fn handler<T: controls::Splitted>(this: &mut Splitted, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
    let hwnd = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().native_id().into();
    match msg {
        winuser::WM_SIZE | common::WM_UPDATE_INNER => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;
            {
                this.set_skip_draw(true);
                {
                    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
                    let base = &mem::transmute::<WinPtr, &Splitted>(ww).inner().base;
                    let ll = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
                    ll.update_children_layout(base);
                    ll.draw_children();
                    //ll.draw_divider(base);
                }
                this.set_skip_draw(false);
            }
            if msg != common::WM_UPDATE_INNER {
                this.call_on_size::<T>(width, height);
            } else {
                winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::TRUE);
            }
            return 0;
        }
        winuser::WM_MOUSEMOVE => {
            let x = lparam as u16;
            let y = (lparam >> 16) as u16;
            let mut updated = false;

            let (width, height) = this.inner().base.measured;

            match controls::HasOrientation::orientation(this) {
                layout::Orientation::Horizontal => {
                    if width >= DEFAULT_BOUND as u16 && x > DEFAULT_BOUND as u16 && x < (width - DEFAULT_BOUND as u16) {
                        winuser::SetCursor(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().cursor);

                        if wparam == winuser::MK_LBUTTON && true {
                            this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().splitter = x as f32 / width as f32;
                            updated = true;
                        }
                    } else {
	                    winuser::SetCursor(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().default_cursor);	
                    }
                }
                layout::Orientation::Vertical => {
                    if height >= DEFAULT_BOUND as u16 && y > DEFAULT_BOUND as u16 && y < (height - DEFAULT_BOUND as u16) {
                        winuser::SetCursor(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().cursor);

                        if wparam == winuser::MK_LBUTTON && this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().moving {
                            this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().splitter = y as f32 / height as f32;
                            updated = true;
                        }
                    } else {
                    	winuser::SetCursor(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().default_cursor);
                    }
                }
            }
            if updated {
                let packed = ((height as i32) << 16) + width as i32;
                winuser::SendMessageW(hwnd, common::WM_UPDATE_INNER, 0, packed as isize);
            }
            return 0;
        }
        winuser::WM_LBUTTONDOWN => {
            winuser::SetCursor(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().cursor);
            this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().moving = true;
            winuser::SetCapture(hwnd);
            return 0;
        }
        winuser::WM_LBUTTONUP => {
            winuser::ReleaseCapture();
            this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().moving = false;
            return 0;
        }
        winuser::WM_CTLCOLORLISTBOX | winuser::WM_CTLCOLORSTATIC => {
            let hdc = wparam as windef::HDC;
            //wingdi::SetTextColor(hdc, wingdi::RGB(0, 0, 0));
            wingdi::SetBkMode(hdc, wingdi::TRANSPARENT as i32);

            return wingdi::GetStockObject(wingdi::NULL_BRUSH as i32) as isize;
        }
        winuser::WM_PAINT => {
            let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
            let base = &mem::transmute::<WinPtr, &Splitted>(ww).inner().base;
            let this = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
            this.draw_divider(base);
        }
        _ => {}
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}
