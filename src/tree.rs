use crate::common::{self, *};

const CLASS_ID: &str = commctrl::WC_TREEVIEW;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type Tree = AMember<AControl<AContainer<AAdapted<ATree<WindowsTree>>>>>;

struct TreeNode {
    node: adapter::Node,
    item: *mut winapi::um::commctrl::TREEITEM,
    root: Box<dyn controls::Control>,
    branches: Vec<TreeNode>,
}

#[repr(C)]
pub struct WindowsTree {
    base: WindowsControlBase<Tree>,
    items: Vec<TreeNode>,
    on_item_click: Option<callbacks::OnItemClick>,
}

impl WindowsTree {
    fn add_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize], node: &adapter::Node, y: &mut i32) {
        let (member, control, adapter, _) = unsafe { Tree::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        let scroll_width = unsafe { winuser::GetSystemMetrics(winuser::SM_CXVSCROLL) };
        let this: &mut Tree = unsafe { utils::base_to_impl_mut(member) };
        
        let mut item = adapter.adapter.spawn_item_view(indexes, this);
        
        let mut items = &mut self.items;
        let mut parent = None;
        for i in 0..indexes.len() {
            let index = indexes[i];
            let end = i+1 >= indexes.len();
            if end {
                let insert_struct = unsafe {
		        	let mut insert_struct = winapi::um::commctrl::TVINSERTSTRUCTW {
			        	hParent: parent.unwrap_or(ptr::null_mut()),
			        	hInsertAfter: if index == 0 { ptr::null_mut() } else { items[index-1].item },
			        	u: mem::zeroed()
		        	};
		        	
		        	let insert_item = winapi::um::commctrl::TVITEMEXW {
		        		..Default::default()
		        	};
		        	
		        	*(insert_struct.u.itemex_mut()) = insert_item;
		        	
		            insert_struct
		        };
                items.insert(index, TreeNode {
                    node: node.clone(),
                    root: {
                        item.as_mut().map(|item| {
                                item.set_layout_width(layout::Size::WrapContent);
                                item.as_mut()
                            }).unwrap().on_added_to_container(this, 0, *y, utils::coord_to_size(pw as i32 - scroll_width - 14 /*TODO: WHY???*/ - DEFAULT_PADDING) as u16, utils::coord_to_size(ph as i32) as u16);
                        item.take().unwrap()
                    },
                    branches: vec![],
                    item: unsafe {
                    	winuser::SendMessageW(self.base.hwnd, winapi::um::commctrl::TVM_INSERTITEMW, 0, &insert_struct as *const winapi::um::commctrl::TVINSERTSTRUCTW as isize) as *mut winapi::um::commctrl::TREEITEM
                    },
                });
                let (_, yy) = items[index].root.size();
		        unsafe {
		        	if 0 > winuser::SendMessageW(self.base.hwnd, winapi::um::commctrl::TVM_SETITEMHEIGHT, i, yy as isize) {
		                common::log_error();
		            }
		        }
                /*match items[index].node {
                	adapter::Node::Branch(expanded) => {
                		let path = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().store.get_path(iter.as_ref().unwrap()).unwrap();
                		if expanded {
                			this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().boxc.expand_row(&path, false); 
                		} else {
                			this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().boxc.collapse_row(&path); 
                		}
                	},
                	_ => {}
                }*/
                
                return;
            } else {
            	parent = Some(items[index].item);
                items = &mut items[index].branches;
            }
        }
        
    }
    fn remove_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize]) {
        let this: &mut Tree = unsafe { utils::base_to_impl_mut(base) };
        let mut items = &mut self.items;
        for i in 0..indexes.len() {
            let index = indexes[i];
                
            if i+1 >= indexes.len() {
                let mut item = items.remove(index);
                item.root.on_removed_from_container(this);
                unsafe {
		            if minwindef::TRUE as isize != winuser::SendMessageW(self.base.hwnd, winapi::um::commctrl::TVM_DELETEITEM, 0, item.item as isize) {
		                common::log_error();
		            }
		        }
            } else {
                items = &mut items[index].branches;
            }
        }
    }
    fn update_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize], node: &adapter::Node) {
    }
    fn force_scrollbar(&mut self) {
        unsafe {
            winuser::ShowScrollBar(self.base.hwnd, winuser::SB_VERT as i32, minwindef::TRUE);
        }
    }
}
impl<O: controls::Tree> NewTreeInner<O> for WindowsTree {
    fn with_uninit(_: &mut mem::MaybeUninit<O>) -> Self {
        WindowsTree {
            base: WindowsControlBase::with_handler(Some(handler::<O>)),
            items: vec![],
            on_item_click: None,
        }
    }
}
impl TreeInner for WindowsTree {
    fn with_adapter(adapter: Box<dyn types::Adapter>) -> Box<dyn controls::Tree> {
        let mut b: Box<mem::MaybeUninit<Tree>> = Box::new_uninit();
        let mut ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AAdapted::with_inner(
                        ATree::with_inner(
                            <Self as NewTreeInner<Tree>>::with_uninit(b.as_mut())
                        ),
                        adapter,
                        &mut b,
                    ),
                )
            ),
        );
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
}
impl ItemClickableInner for WindowsTree {
    fn item_click(&mut self, indexes: &[usize], item_view: &mut dyn controls::Control, skip_callbacks: bool) {
        if !skip_callbacks{
            let self2 = self.base.as_outer_mut();
            if let Some(ref mut callback) = self.on_item_click {
                (callback.as_mut())(self2, indexes, item_view)
            }
        }
    }
    fn on_item_click(&mut self, callback: Option<callbacks::OnItemClick>) {
        self.on_item_click = callback;
    }
}
impl AdaptedInner for WindowsTree {
    fn on_item_change(&mut self, base: &mut MemberBase, value: adapter::Change) {
        if !self.base.hwnd.is_null() {
            let mut y = 0;
	        {
	            fn yadder(level: &[TreeNode], y: &mut i32) {
	                for item in level {
	                    let (_, yy) = item.root.size();
	                    *y += yy as i32;
	                    yadder(item.branches.as_slice(), y);
	                }
	            };
	            yadder(self.items.as_slice(), &mut y);        
	        }
	        match value {
	            adapter::Change::Added(at, ref node) => {
	                self.add_item_inner(base, at, node, &mut y);
	            },
	            adapter::Change::Removed(at) => {
	                self.remove_item_inner(base, at);
	            },
	            adapter::Change::Edited(at, ref node) => {
	            	self.update_item_inner(base, at, node);
	            },
	        }
	        self.base.invalidate();
            self.force_scrollbar();
        }
    }
}

impl ControlInner for WindowsTree {
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
        self.base.hwnd = unsafe { parent.native_id() as windef::HWND }; // required for measure, as we don't have own hwnd yet
        let (w, h, _) = self.measure(member, control, pw, ph);
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
        );
        control.coords = Some((px as i32, py as i32));
        
        let (member, _, adapter, _) = unsafe { Tree::adapter_base_parts_mut(member) };

        let mut y = 0;
        adapter.adapter.for_each(&mut (|indexes, node| {
            self.add_item_inner(member, indexes, node, &mut y);
        }));
        self.force_scrollbar();
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_TREE;

        fill_from_markup_base!(self, member, markup, registry, Tree, [MEMBER_TYPE_TREE]);
        //fill_from_markup_items!(self, member, markup, registry);
    }
}
impl ContainerInner for WindowsTree {
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        for child in self.items.as_mut_slice() {
            match arg {
                types::FindBy::Id(ref id) => {
                    if child.root.as_member_mut().id() == *id {
                        return Some(child.root.as_mut());
                    }
                }
                types::FindBy::Tag(tag) => {
                    if let Some(mytag) = child.root.as_member_mut().tag() {
                        if tag == mytag {
                            return Some(child.root.as_mut());
                        }
                    }
                }
            }
            if let Some(c) = child.root.is_container_mut() {
                let ret = c.find_control_mut(arg.clone());
                if ret.is_none() {
                    continue;
                }
                return ret;
            }
        }
        None
    }
    fn find_control<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn controls::Control> {
        for child in self.items.as_slice() {
            match arg {
                types::FindBy::Id(ref id) => {
                    if child.root.as_member().id() == *id {
                        return Some(child.root.as_ref());
                    }
                }
                types::FindBy::Tag(tag) => {
                    if let Some(mytag) = child.root.as_member().tag() {
                        if tag == mytag {
                            return Some(child.root.as_ref());
                        }
                    }
                }
            }
            if let Some(c) = child.root.is_container() {
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
impl HasLayoutInner for WindowsTree {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            self.base.invalidate();
        }
    }
}
impl HasNativeIdInner for WindowsTree {
    type Id = common::Hwnd;

    fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}
impl MemberInner for WindowsTree {}

impl HasSizeInner for WindowsTree {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for WindowsTree {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl Drawable for WindowsTree {
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
impl Spawnable for WindowsTree {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_adapter(Box::new(types::imp::StringVecAdapter::<crate::imp::Text>::new())).into_control()
    }
}

unsafe extern "system" fn handler<T: controls::Tree>(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as WinPtr);
    }
    match msg {/*
        winuser::WM_LBUTTONUP => {
            let i = winuser::SendMessageW(hwnd, winapi::um::commctrl::TVM_ITEMFROMPOINT, 0, lparam);
            let tree: &mut Tree = mem::transmute(param);
            let item_view = tree.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().items.get_mut(i as usize).unwrap();
            let tree: &mut Tree = mem::transmute(param); // bck is stupid
            if let Some(ref mut callback) = tree.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().on_item_click {
                let tree: &mut Tree = mem::transmute(param); // bck is still stupid
                (callback.as_mut())(tree, &[i as usize], item_view.as_mut());
            }
        }
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let tree: &mut Tree = mem::transmute(param);
            tree.call_on_size::<T>(width, height);
            
            let mut y = 0;
            let i = cmp::max(0, winuser::SendMessageW(hwnd, winapi::um::commctrl::TVM_GETTOPINDEX, 0, 0)) as usize;
            let tree = tree.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
            for i in i..tree.items.len() {
                let item = &mut tree.items[i].root;
                let (_, ch, _) = item.measure(cmp::max(0, width as i32 - DEFAULT_PADDING) as u16, cmp::max(0, height as i32) as u16);
                item.draw(Some((0, y)));
                y += ch as i32;
            }
            winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::FALSE);
            tree.force_scrollbar();
        }
        winuser::WM_CTLCOLORSTATIC => {
            let hdc = wparam as windef::HDC;
            wingdi::SetTextColor(hdc, wingdi::RGB(0, 0, 0));
            wingdi::SetBkMode(hdc, wingdi::TRANSPARENT as i32);

            return wingdi::GetStockObject(wingdi::NULL_BRUSH as i32) as isize;
        }*/
        winuser::WM_VSCROLL | winuser::WM_MOUSEWHEEL => {
            winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::FALSE);
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}
