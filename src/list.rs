use crate::common::{self, *};

const CLASS_ID: &str = commctrl::WC_LISTBOX;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type List = AMember<AControl<AContainer<AAdapted<AList<WindowsList>>>>>;

#[repr(C)]
pub struct WindowsList {
    base: WindowsControlBase<List>,
    items: Vec<Box<dyn controls::Control>>,
    on_item_click: Option<callbacks::OnItemClick>,
}

impl WindowsList {
    fn add_item_inner(&mut self, base: &mut MemberBase, i: usize, y: &mut i32) {
        let (member, control, adapter, _) = unsafe { List::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        let scroll_width = unsafe { winuser::GetSystemMetrics(winuser::SM_CXVSCROLL) };
        let this: &mut List = unsafe { utils::base_to_impl_mut(member) };
        
        let mut item = adapter.adapter.spawn_item_view(i, this);
        item.on_added_to_container(this, 0, *y, utils::coord_to_size(pw as i32 - scroll_width - 14 /*TODO: WHY???*/ - DEFAULT_PADDING) as u16, utils::coord_to_size(ph as i32) as u16);
                
        let (_, yy) = item.size();
        self.items.push(item);
        *y += yy as i32;
        
        unsafe {
            if i as isize != winuser::SendMessageW(self.base.hwnd, winuser::LB_ADDSTRING, 0, WINDOW_CLASS.as_ptr() as isize) {
                common::log_error();
            }
            if winuser::LB_ERR == winuser::SendMessageW(self.base.hwnd, winuser::LB_SETITEMHEIGHT, i, yy as isize) {
                common::log_error();
            }
        }
    }
    fn remove_item_inner(&mut self, base: &mut MemberBase, i: usize) {
        let this: &mut List = unsafe { utils::base_to_impl_mut(base) };
        self.items.remove(i).on_removed_from_container(this); 
        unsafe {
            if i as isize != winuser::SendMessageW(self.base.hwnd, winuser::LB_DELETESTRING, 0, WINDOW_CLASS.as_ptr() as isize) {
                common::log_error();
            }
        }
    }
    fn force_scrollbar(&mut self) {
        unsafe {
            winuser::ShowScrollBar(self.base.hwnd, winuser::SB_VERT as i32, minwindef::TRUE);
        }
    }
}
impl<O: controls::List> NewListInner<O> for WindowsList {
    fn with_uninit(u: &mut mem::MaybeUninit<O>) -> Self {
        WindowsList {
            base: WindowsControlBase::with_handler(Some(handler::<O>)),
            items: vec![],
            on_item_click: None,
        }
    }
}
impl ListInner for WindowsList {
    fn with_adapter(adapter: Box<dyn types::Adapter>) -> Box<dyn controls::List> {
        let len = adapter.len();
        let mut b: Box<mem::MaybeUninit<List>> = Box::new_uninit();
        let mut ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AAdapted::with_inner(
                        AList::with_inner(
                            <Self as NewListInner<List>>::with_uninit(b.as_mut())
                        ),
                        adapter,
                        &mut b,
                    ),
                )
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        );
        ab.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().items = Vec::with_capacity(len);
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
}
impl ItemClickableInner for WindowsList {
    fn item_click(&mut self, i: usize, item_view: &mut dyn controls::Control, skip_callbacks: bool) {
        if !skip_callbacks{
            let self2 = self.base.as_outer_mut();
            if let Some(ref mut callback) = self.on_item_click {
                (callback.as_mut())(self2, i, item_view)
            }
        }
    }
    fn on_item_click(&mut self, callback: Option<callbacks::OnItemClick>) {
        self.on_item_click = callback;
    }
}
impl AdaptedInner for WindowsList {
    fn on_item_change(&mut self, base: &mut MemberBase, value: types::Change) {
        if !self.base.hwnd.is_null() {
            let mut y = 0;
            {
                for item in self.items.as_slice() {
                    let (_, yy) = item.size();
                    y += yy as i32;
                }
            }
            match value {
                types::Change::Added(at) => {
                    self.add_item_inner(base, at, &mut y);
                },
                types::Change::Removed(at) => {
                    self.remove_item_inner(base, at);
                },
                types::Change::Edited(_) => {
                },
            }
            self.base.invalidate();
            self.force_scrollbar();
        }
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
            self.base.create_control_hwnd(
                px as i32,
                py as i32,
                w as i32,
                h as i32,
                self.base.hwnd,
                0,
                WINDOW_CLASS.as_ptr(),
                "",
                winuser::WS_EX_CONTROLPARENT | winuser::WS_CLIPCHILDREN | winuser::LBS_OWNERDRAWVARIABLE | winuser::WS_THICKFRAME | winuser::WS_VSCROLL | winuser::WS_EX_RIGHTSCROLLBAR,
                selfptr,
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
        control.coords = Some((px as i32, py as i32));
        
        let (member, _, adapter, _) = unsafe { List::adapter_base_parts_mut(member) };

        let mut y = 0;
        for i in 0..adapter.adapter.len() {
            self.add_item_inner(member, i, &mut y);
        }
        self.force_scrollbar();
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        for ref mut child in self.items.as_mut_slice() {
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
        //fill_from_markup_items!(self, member, markup, registry);
    }
}
impl ContainerInner for WindowsList {
    fn find_control_mut(&mut self, arg: types::FindBy) -> Option<&mut dyn controls::Control> {
        for child in self.items.as_mut_slice() {
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
        for child in self.items.as_slice() {
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
}
impl HasNativeIdInner for WindowsList {
    type Id = common::Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}
impl MemberInner for WindowsList {}

impl HasSizeInner for WindowsList {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
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
impl Spawnable for WindowsList {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_adapter(Box::new(types::imp::StringVecAdapter::<crate::imp::Text>::new())).into_control()
    }
}

unsafe extern "system" fn handler<T: controls::List>(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_LBUTTONUP => {
            let i = winuser::SendMessageW(hwnd, winuser::LB_ITEMFROMPOINT, 0, lparam);
            let list: &mut List = mem::transmute(param);
            let item_view = list.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().items.get_mut(i as usize).unwrap();
            let list: &mut List = mem::transmute(param); // bck is stupid
            if let Some(ref mut callback) = list.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().on_item_click {
                let list: &mut List = mem::transmute(param); // bck is still stupid
                (callback.as_mut())(list, i as usize, item_view.as_mut());
            }
        }
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let list: &mut List = mem::transmute(param);
            list.call_on_size::<T>(width, height);
            
            let mut y = 0;
            let i = cmp::max(0, winuser::SendMessageW(hwnd, winuser::LB_GETTOPINDEX, 0, 0)) as usize;
            let list = list.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
            for i in i..list.items.len() {
                let item = &mut list.items[i];
                let (_, ch, _) = item.measure(cmp::max(0, width as i32 - DEFAULT_PADDING) as u16, cmp::max(0, height as i32) as u16);
                item.draw(Some((0, y)));
                y += ch as i32;
            }
            winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::FALSE);
            list.force_scrollbar();
        }
        winuser::WM_CTLCOLORSTATIC => {
            let hdc = wparam as windef::HDC;
            wingdi::SetTextColor(hdc, wingdi::RGB(0, 0, 0));
            wingdi::SetBkMode(hdc, wingdi::TRANSPARENT as i32);

            return wingdi::GetStockObject(wingdi::NULL_BRUSH as i32) as isize;
        }
        winuser::WM_VSCROLL | winuser::WM_MOUSEWHEEL => {
            winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::FALSE);
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}

default_impls_as!(List);
