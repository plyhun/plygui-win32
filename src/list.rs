use crate::common::{self, *};

const CLASS_ID: &str = commctrl::WC_LISTBOX;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type List = Member<Control<Adapter<WindowsList>>>;

#[repr(C)]
pub struct WindowsList {
    base: WindowsControlBase<List>,
    children: Vec<Box<dyn controls::Control>>,
}

impl AdapterViewInner for WindowsList {
    fn with_adapter(adapter: Box<dyn types::Adapter>) -> Box<List> {
        let b = Box::new(Member::with_inner(
            Control::with_inner(
                Adapter::with_inner(
                    WindowsList {
                        base: WindowsControlBase::new(),
                        children: Vec::with_capacity(adapter.len()),
                    },
                    adapter,
                ),
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        b
    }
}

impl ControlInner for WindowsList {
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
        let (w, h, _) = self.measure(member, control, pw, ph);
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            common::create_control_hwnd(
                px as i32,
                py as i32,
                w as i32,
                h as i32,
                self.base.hwnd,
                0,
                WINDOW_CLASS.as_ptr(),
                "",
                winuser::WS_EX_CONTROLPARENT | winuser::WS_CLIPCHILDREN | winuser::LBS_OWNERDRAWVARIABLE | winuser::WS_BORDER | winuser::WS_VSCROLL | winuser::WS_EX_RIGHTSCROLLBAR,
                selfptr,
                Some(handler),
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
        control.coords = Some((px as i32, py as i32));
        
       let (member, _, adapter) = List::adapter_base_parts_mut(member);
        
        let mut x = 0;
        let mut y = 0;
        for i in 0..adapter.adapter.len() {
            let self2: &mut List = unsafe { utils::base_to_impl_mut(member) };
            let mut item = adapter.adapter.spawn_item_view(i, self2);
            item.on_added_to_container(self2, x, y, utils::coord_to_size(pw as i32) as u16, utils::coord_to_size(ph as i32) as u16);
            let (xx, yy) = item.size();
            self.children.push(item);
            y += yy as i32;
            
            unsafe {
                if i as isize != winuser::SendMessageW(self.base.hwnd, winuser::LB_ADDSTRING, 0, WINDOW_CLASS.as_ptr() as isize) {
                    common::log_error();
                }
                if winuser::LB_ERR == winuser::SendMessageW(self.base.hwnd, winuser::LB_SETITEMHEIGHT, i, yy as isize) {
                    common::log_error();
                }
            }
        }
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        for ref mut child in self.children.as_mut_slice() {
            let self2: &mut List = unsafe { utils::base_to_impl_mut(member) };
            child.on_removed_from_container(self2);
        }
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_TABLE;

        fill_from_markup_base!(self, member, markup, registry, List, [MEMBER_TYPE_TABLE]);
        fill_from_markup_children!(self, member, markup, registry);
    }
}
impl ContainerInner for WindowsList {
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
impl HasLayoutInner for WindowsList {
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
impl HasNativeIdInner for WindowsList {
    type Id = common::Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}
impl MemberInner for WindowsList {}

impl HasSizeInner for WindowsList {
    fn on_size_set(&mut self, base: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        use plygui_api::controls::HasLayout;

        let this = base.as_any_mut().downcast_mut::<List>().unwrap();
        this.set_layout_width(layout::Size::Exact(width));
        this.set_layout_width(layout::Size::Exact(height));
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for WindowsList {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl Drawable for WindowsList {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
        /*let mut x = DEFAULT_PADDING;
        let mut y = DEFAULT_PADDING;
        for row in self.rows.as_mut_slice() {
            for child in row.cols.as_mut_slice() {
                if let Some(child) = child {
                    child.draw(Some((x, y)));
                    /*let (xx, yy) = child.size();
                    match self.orientation {
                        layout::Orientation::Horizontal => x += xx as i32,
                        layout::Orientation::Vertical => y += yy as i32,
                    }*/
                }
            }
        }*/
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING,
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING,
                };
                (cmp::max(0, w as i32) as u16, cmp::max(0, h as i32) as u16)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate()
    }
}

/*#[allow(dead_code)]
pub(crate) fn spawn() -> Box<dyn controls::Control> {
    List::with_dimensions(0, 0).into_control()
}*/

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_LBUTTONUP => {
            let x = lparam as u16;
            let y = (lparam >> 16) as u16;
            println!("clicked {}/{}", x,y);
        }
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let list: &mut List = mem::transmute(param);
            list.call_on_size(width, height);
            return 0;
        }
        winuser::WM_COMMAND => {
            let id = wparam as u16;
            let param = (wparam >> 16) as u16;
            
            match param as u32 {
                winuser::WM_MEASUREITEM => {
                    println!("Measure");
                    return 0;
                }
                winuser::WM_DRAWITEM => {
                    println!("Draw");
                    return 0;
                }
                _ => {}
            }
        }
        winuser::WM_MEASUREITEM => {
            println!("Measure2");
            return 0;
        }
        winuser::WM_DRAWITEM => {
            println!("Draw2");
            return 0;
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}

default_impls_as!(List);
