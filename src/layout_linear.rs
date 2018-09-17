use super::*;
use super::common::*;

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
	//pub static ref INSTANCE: winuser::HINSTANCE = unsafe { kernel32::GetModuleHandleW(ptr::null()) };
}

pub type LinearLayout = Member<Control<MultiContainer<WindowsLinearLayout>>>;

#[repr(C)]
pub struct WindowsLinearLayout {
    base: WindowsControlBase<LinearLayout>,
    orientation: layout::Orientation,
    children: Vec<Box<controls::Control>>,
}

impl LinearLayoutInner for WindowsLinearLayout {
    fn with_orientation(orientation: layout::Orientation) -> Box<LinearLayout> {
        let b = Box::new(Member::with_inner(Control::with_inner(MultiContainer::with_inner(WindowsLinearLayout {
                                                                                               base: WindowsControlBase::new(),
                                                                                               orientation: orientation,
                                                                                               children: Vec::new(),
                                                                                           },
                                                                                           ()),
                                                                ()),
                                            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)));
        b
    }
}

impl HasOrientationInner for WindowsLinearLayout {
    fn layout_orientation(&self) -> layout::Orientation {
        self.orientation
    }
    fn set_layout_orientation(&mut self, _base: &mut MemberBase, orientation: layout::Orientation) {
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
    fn set_child_to(&mut self, base: &mut MemberBase, index: usize, child: Box<controls::Control>) -> Option<Box<controls::Control>> {
        let old = self.remove_child_from(base, index);

        self.children.insert(index, child);
        if !self.base.hwnd.is_null() {
            let (w, h) = self.size();
            self.children
                .get_mut(index)
                .unwrap()
                .on_added_to_container(common::member_from_hwnd::<LinearLayout>(self.base.hwnd),
                                       w as i32 - DEFAULT_PADDING,
                                       h as i32 - DEFAULT_PADDING,
                                       utils::coord_to_size(w as i32 - DEFAULT_PADDING),
                                       utils::coord_to_size(h as i32 - DEFAULT_PADDING));
            self.base.invalidate();
        }
        old
    }
    fn remove_child_from(&mut self, _base: &mut MemberBase, index: usize) -> Option<Box<controls::Control>> {
        if index < self.children.len() {
            let mut old = self.children.remove(index);
            if !self.base.hwnd.is_null() {
                old.on_removed_from_container(common::member_from_hwnd::<LinearLayout>(self.base.hwnd));
                self.base.invalidate();
            }
            Some(old)
        } else {
            None
        }
    }
    fn child_at(&self, index: usize) -> Option<&controls::Control> {
        self.children.get(index).map(|c| c.as_ref())
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut controls::Control> {
        //self.children.get_mut(index).map(|c| c.as_mut()) //the anonymous lifetime #1 does not necessarily outlive the static lifetime
        if let Some(c) = self.children.get_mut(index) {
            Some(c.as_mut())
        } else {
            None
        }
    }
}
impl ControlInner for WindowsLinearLayout {
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
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &controls::Container, px: i32, py: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        let (width, height, _) = self.measure(member, control, pw, ph);
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            common::create_control_hwnd(px as i32,
                                        py as i32,
                                        width as i32,
                                        height as i32,
                                        parent.native_id() as windef::HWND,
                                        winuser::WS_EX_CONTROLPARENT,
                                        WINDOW_CLASS.as_ptr(),
                                        "",
                                        0,
                                        selfptr,
                                        None)
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
        self.base.coords = Some((px as i32, py as i32));
        let mut x = DEFAULT_PADDING;
        let mut y = DEFAULT_PADDING;
        for ref mut child in self.children.as_mut_slice() {
            let self2: &mut LinearLayout = unsafe { utils::base_to_impl_mut(member) };
            child.on_added_to_container(self2,
                                        x,
                                        y,
                                        utils::coord_to_size(pw as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16,
                                        utils::coord_to_size(ph as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16);
            let (xx, yy) = child.size();
            match self.orientation {
                layout::Orientation::Horizontal => x += xx as i32,
                layout::Orientation::Vertical => y += yy as i32,
            }
        }
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &controls::Container) {
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

        fill_from_markup_base!(self,
                               member,
                               markup,
                               registry,
                               LinearLayout,
                               [MEMBER_TYPE_LINEAR_LAYOUT]);
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
impl MemberInner for WindowsLinearLayout {
    type Id = common::Hwnd;

    fn size(&self) -> (u16, u16) {
        self.base.size()
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            unsafe {
                winuser::ShowWindow(self.base.hwnd,
                                    if base.visibility == types::Visibility::Visible {
                                        winuser::SW_SHOW
                                    } else {
                                        winuser::SW_HIDE
                                    });
            }
            self.base.invalidate();
        }
    }
    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}
impl ContainerInner for WindowsLinearLayout {
    fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut controls::Control> {
        for child in self.children.as_mut_slice() {
            if child.as_member().id() == id_ {
                return Some(child.as_mut());
            } else if let Some(c) = child.is_container_mut() {
                let ret = c.find_control_by_id_mut(id_);
                if ret.is_none() {
                    continue;
                }
                return ret;
            }
        }
        None
    }
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&controls::Control> {
        for child in self.children.as_slice() {
            if child.as_member().id() == id_ {
                return Some(child.as_ref());
            } else if let Some(c) = child.is_container() {
                let ret = c.find_control_by_id(id_);
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
    fn draw(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
        self.base.draw(coords);
        let mut x = DEFAULT_PADDING;
        let mut y = DEFAULT_PADDING;
        for ref mut child in self.children.as_mut_slice() {
            child.draw(Some((x, y)));
            let (xx, yy) = child.size();
            match self.orientation {
                layout::Orientation::Horizontal => x += xx as i32,
                layout::Orientation::Vertical => y += yy as i32,
            }
        }
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let orientation = self.orientation;
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
                        for child in self.children.as_mut_slice() {
                            let (cw, _, _) = child.measure(max(0, parent_width as i32 - hp) as u16,
                                                           max(0, parent_height as i32 - vp) as u16);
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
                                let (_, ch, _) = child.measure(max(0, parent_width as i32 - hp) as u16,
                                                               max(0, parent_height as i32 - vp) as u16);
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
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate()
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    LinearLayout::with_orientation(layout::Orientation::Vertical).into_control()
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32LinearLayout")
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
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
            use std::cmp::max;

            let mut width = lparam as u16;
            let mut height = (lparam >> 16) as u16;
            let mut ll: &mut LinearLayout = mem::transmute(ww);
            let o = ll.as_inner().as_inner().as_inner().orientation;
            let hp = DEFAULT_PADDING + DEFAULT_PADDING;
            let vp = DEFAULT_PADDING + DEFAULT_PADDING;

            let mut x = 0;
            let mut y = 0;
            for child in ll.as_inner_mut()
                    .as_inner_mut()
                    .as_inner_mut()
                    .children
                    .as_mut_slice() {
                let (cw, ch, _) = child.measure(max(0, width as i32 - hp) as u16,
                                                max(0, height as i32 - vp) as u16);
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

            ll.call_on_resize(width, height);
            return 0;
        }
        _ => {}
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl_all_defaults!(LinearLayout);
