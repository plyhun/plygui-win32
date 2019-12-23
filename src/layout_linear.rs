use crate::common::{self, *};

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
    //pub static ref INSTANCE: winuser::HINSTANCE = unsafe { kernel32::GetModuleHandleW(ptr::null()) };
}

pub type LinearLayout = AMember<AControl<AContainer<AMultiContainer<ALinearLayout<WindowsLinearLayout>>>>>;

#[repr(C)]
pub struct WindowsLinearLayout {
    base: WindowsControlBase<LinearLayout>,
    orientation: layout::Orientation,
    children: Vec<Box<dyn controls::Control>>,
}

impl LinearLayoutInner for WindowsLinearLayout {
    fn with_orientation(orientation: layout::Orientation) -> Box<dyn controls::LinearLayout> {
        let b = Box::new(AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AMultiContainer::with_inner(
                        ALinearLayout::with_inner(
                            WindowsLinearLayout {
                                base: WindowsControlBase::new(),
                                orientation: orientation,
                                children: Vec::new(),
                            },
                        )
                    ),
                )
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        b
    }
}

impl HasOrientationInner for WindowsLinearLayout {
    fn orientation(&self, _base: &MemberBase) -> layout::Orientation {
        self.orientation
    }
    fn set_orientation(&mut self, _base: &mut MemberBase, orientation: layout::Orientation) {
        if orientation != self.orientation {
            self.orientation = orientation;
            self.base.invalidate();
        }
    }
}
impl MultiContainerInner for WindowsLinearLayout {
    fn len(&self) -> usize {
        self.children.len()
    }
    fn set_child_to(&mut self, base: &mut MemberBase, index: usize, child: Box<dyn controls::Control>) -> Option<Box<dyn controls::Control>> {
        let old = self.remove_child_from(base, index);

        self.children.insert(index, child);
        if !self.base.hwnd.is_null() {
            let (w, h) = base.as_any().downcast_ref::<LinearLayout>().unwrap().inner().base.measured;
            let (cw, ch) = {
                let mut w = 0;
                let mut h = 0;
                for i in 0..index {
                    let (cw, ch) = self.children[i].size();
                    w += cw;
                    h += ch;
                }
                (w as i32, h as i32)
            };
            match self.orientation {
                layout::Orientation::Vertical => {
                    self.children.get_mut(index).unwrap().on_added_to_container(
                        self.base.as_outer_mut(),
                        DEFAULT_PADDING,
                        ch + DEFAULT_PADDING,
                        utils::coord_to_size(w as i32 - DEFAULT_PADDING - DEFAULT_PADDING),
                        utils::coord_to_size(h as i32 - ch - DEFAULT_PADDING - DEFAULT_PADDING),
                    );
                },
                layout::Orientation::Horizontal => {
                    self.children.get_mut(index).unwrap().on_added_to_container(
                        self.base.as_outer_mut(),
                        cw + DEFAULT_PADDING,
                        DEFAULT_PADDING,
                        utils::coord_to_size(w as i32 - cw - DEFAULT_PADDING - DEFAULT_PADDING),
                        utils::coord_to_size(h as i32 - DEFAULT_PADDING - DEFAULT_PADDING),
                    );                
                }
            }
            
        }
        old
    }
    fn remove_child_from(&mut self, _base: &mut MemberBase, index: usize) -> Option<Box<dyn controls::Control>> {
        if index < self.children.len() {
            let mut old = self.children.remove(index);
            if !self.base.hwnd.is_null() {
                old.on_removed_from_container(self.base.as_outer_mut());
                self.base.invalidate();
            }
            Some(old)
        } else {
            None
        }
    }
    fn child_at(&self, index: usize) -> Option<&dyn controls::Control> {
        self.children.get(index).map(|c| c.as_ref())
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut dyn controls::Control> {
        //self.children.get_mut(index).map(|c| c.as_mut()) //the anonymous lifetime #1 does not necessarily outlive the static lifetime
        if let Some(c) = self.children.get_mut(index) {
            Some(c.as_mut())
        } else {
            None
        }
    }
}
impl ControlInner for WindowsLinearLayout {
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
        let (width, height, _) = self.measure(member, control, pw, ph);
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
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
        let mut x = DEFAULT_PADDING;
        let mut y = DEFAULT_PADDING;
        let mut pw = pw as i32 - DEFAULT_PADDING - DEFAULT_PADDING;
        let mut ph = ph as i32 - DEFAULT_PADDING - DEFAULT_PADDING;
        for ref mut child in self.children.as_mut_slice() {
            let self2: &mut LinearLayout = unsafe { utils::base_to_impl_mut(member) };
            child.on_added_to_container(
                self2,
                x,
                y,
                utils::coord_to_size(pw) as u16,
                utils::coord_to_size(ph) as u16,
            );
            let (xx, yy) = child.size();
            match self.orientation {
                layout::Orientation::Horizontal => {
                    x += xx as i32;
                    pw -= xx as i32;
                },
                layout::Orientation::Vertical => {
                    y += yy as i32;
                    ph -= yy as i32;
                },
            }
        }
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        for ref mut child in self.children.as_mut_slice() {
            let self2: &mut LinearLayout = unsafe { utils::base_to_impl_mut(member) };
            child.on_removed_from_container(self2);
        }
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT;

        fill_from_markup_base!(self, member, markup, registry, LinearLayout, [MEMBER_TYPE_LINEAR_LAYOUT]);
        fill_from_markup_children!(self, member, markup, registry);
    }
}
impl HasLayoutInner for WindowsLinearLayout {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            self.base.invalidate();
        }
    }
    fn layout_margin(&self, _member: &MemberBase) -> layout::BoundarySize {
        layout::BoundarySize::AllTheSame(DEFAULT_PADDING)
    }
}
impl HasNativeIdInner for WindowsLinearLayout {
    type Id = common::Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}
impl MemberInner for WindowsLinearLayout {}

impl HasSizeInner for WindowsLinearLayout {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for WindowsLinearLayout {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl ContainerInner for WindowsLinearLayout {
    fn find_control_mut(&mut self, arg: types::FindBy) -> Option<&mut dyn controls::Control> {
        for child in self.children.as_mut_slice() {
            match arg {
                types::FindBy::Id(ref id) => {
                    if child.as_member_mut().id() == *id {
                        return Some(child.as_mut());
                    }
                }
                types::FindBy::Tag(ref tag) => {
                    if let Some(mytag) = child.as_member_mut().tag() {
                        if tag.as_str() == mytag {
                            return Some(child.as_mut());
                        }
                    }
                }
            }
            if let Some(c) = child.is_container_mut() {
                let ret = c.find_control_mut(arg.clone());
                if ret.is_none() {
                    continue;
                }
                return ret;
            }
        }
        None
    }
    fn find_control(&self, arg: types::FindBy) -> Option<&dyn controls::Control> {
        for child in self.children.as_slice() {
            match arg {
                types::FindBy::Id(ref id) => {
                    if child.as_member().id() == *id {
                        return Some(child.as_ref());
                    }
                }
                types::FindBy::Tag(ref tag) => {
                    if let Some(mytag) = child.as_member().tag() {
                        if tag.as_str() == mytag {
                            return Some(child.as_ref());
                        }
                    }
                }
            }
            if let Some(c) = child.is_container() {
                let ret = c.find_control(arg.clone());
                if ret.is_none() {
                    continue;
                }
                return ret;
            }
        }
        None
    }
}

impl Drawable for WindowsLinearLayout {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let orientation = self.orientation;
        let old_size = control.measured;
        let hp = DEFAULT_PADDING + DEFAULT_PADDING;
        let vp = DEFAULT_PADDING + DEFAULT_PADDING;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut measured = false;
                let w = match control.layout.width {
                    layout::Size::Exact(w) => w,
                    layout::Size::MatchParent => parent_width,
                    layout::Size::WrapContent => {
                        let mut w = 0;
                        for child in self.children.as_mut_slice() {
                            let (cw, _, _) = child.measure(max(0, parent_width as i32 - w as i32 - hp) as u16, max(0, parent_height as i32 - vp) as u16);
                            match orientation {
                                layout::Orientation::Horizontal => {
                                    w += cw;
                                }
                                layout::Orientation::Vertical => {
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
                        for child in self.children.as_mut_slice() {
                            let ch = if measured {
                                child.size().1
                            } else {
                                let (_, ch, _) = child.measure(max(0, parent_width as i32 - hp) as u16, max(0, parent_height as i32 - h as i32 - vp) as u16);
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
impl Spawnable for WindowsLinearLayout {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_orientation(layout::Orientation::Vertical).into_control()
    }
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32LinearLayout").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    let class = winuser::WNDCLASSW {
        style: winuser::CS_DBLCLKS,
        lpfnWndProc: Some(whandler),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: libloaderapi::GetModuleHandleW(ptr::null()),
        hIcon: winuser::LoadIconW(ptr::null_mut(), winuser::IDI_APPLICATION),
        hCursor: winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_ARROW),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };
    winuser::RegisterClassW(&class);
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
            let ll: &mut LinearLayout = mem::transmute(ww);
            let o = ll.inner().inner().inner().inner().inner().orientation;
            let hp = DEFAULT_PADDING + DEFAULT_PADDING;
            let vp = DEFAULT_PADDING + DEFAULT_PADDING;

            let mut x = 0;
            let mut y = 0;
            for child in ll.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().children.as_mut_slice() {
                let (cw, ch, _) = child.measure(cmp::max(0, width as i32 - hp) as u16, cmp::max(0, height as i32 - vp) as u16);
                child.draw(Some((x + DEFAULT_PADDING, y + DEFAULT_PADDING)));
                match o {
                    layout::Orientation::Horizontal if width >= cw => {
                        x += cw as i32;
                        width -= cw;
                    }
                    layout::Orientation::Vertical if height >= ch => {
                        y += ch as i32;
                        height -= ch;
                    }
                    _ => {}
                }
            }

            ll.call_on_size(width, height);
            return 0;
        }
        winuser::WM_CTLCOLORLISTBOX | winuser::WM_CTLCOLORSTATIC => {
            let hdc = wparam as windef::HDC; 
            //wingdi::SetTextColor(hdc, wingdi::RGB(0,0,0));    
            wingdi::SetBkMode(hdc, wingdi::TRANSPARENT as i32);
        
            return wingdi::GetStockObject(wingdi::NULL_BRUSH as i32) as isize;
            //return winuser::GetSysColorBrush(winuser::COLOR_BACKGROUND) as isize;
        }
        _ => {}
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

default_impls_as!(LinearLayout);
