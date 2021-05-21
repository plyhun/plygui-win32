use crate::common::{self, *};

lazy_static! {
    pub static ref WINDOW_CLASS_TREE: Vec<u16> = OsStr::new(commctrl::WC_TREEVIEW).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub type Tree = AMember<AControl<AContainer<AAdapted<ATree<WindowsTree>>>>>;

#[repr(C)]
pub struct WindowsTree {
    base: WindowsControlBase<Tree>,
    hwnd_tree: windef::HWND,
    items: TreeNodeList<winapi::um::commctrl::HTREEITEM>,
    on_item_click: Option<callbacks::OnItemClick>,
}

impl WindowsTree {
    fn add_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize], node: &adapter::Node, y: &mut i32) {
        let (member, control, adapter, _) = unsafe { Tree::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        let scroll_width = unsafe { winuser::GetSystemMetrics(winuser::SM_CXVSCROLL) };
        let this: &mut Tree = unsafe { utils::base_to_impl_mut(member) };
        
        let mut item = adapter.adapter.spawn_item_view(indexes, this);
        
        let mut items = &mut self.items.0;
        let mut parent = None;
        for i in 0..indexes.len() {
            let index = indexes[i];
            let end = i+1 >= indexes.len();
            if end {
            	let item_width = utils::coord_to_size(pw as i32 - scroll_width - 14 /*TODO: WHY???*/ - DEFAULT_PADDING) as u16;
            	let valid_utf8 = vec![b' '; item_width as usize / 32];
				let cstring = CString::new(valid_utf8).unwrap();
                let insert_struct = unsafe {
		        	let mut insert_struct = winapi::um::commctrl::TVINSERTSTRUCTW {
			        	hParent: parent.unwrap_or(ptr::null_mut()),
			        	hInsertAfter: if index == 0 { winapi::um::commctrl::TVI_ROOT } else { items[index-1].native },
			        	u: mem::zeroed()
		        	};
		        	
		        	let insert_item = winapi::um::commctrl::TVITEMEXW {
		        		mask: winapi::um::commctrl::TVIF_TEXT | winapi::um::commctrl::TVIF_PARAM,
		        		pszText: cstring.as_ptr() as *const _ as *mut u16,
		        		//lParam: index as isize,
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
                            }).unwrap().on_added_to_container(this, 0, *y, item_width, utils::coord_to_size(ph as i32) as u16);
                        item.take().unwrap()
                    },
                    branches: vec![],
                    native: ptr::null_mut(),
                });
                items[index].native = unsafe {
                	winuser::SendMessageW(self.hwnd_tree, winapi::um::commctrl::TVM_INSERTITEMW, 0, &insert_struct as *const winapi::um::commctrl::TVINSERTSTRUCTW as isize) as *mut winapi::um::commctrl::TREEITEM
                };
	            let (_, yy) = items[index].root.size();
		        unsafe {
		        	if 0 > winuser::SendMessageW(self.base.hwnd, winapi::um::commctrl::TVM_SETITEMHEIGHT, yy as usize, 0) {
		                common::log_error();
		            }
		        }
                match items[index].node {
                	adapter::Node::Branch(expanded) => unsafe {
                		if 0 == winuser::PostMessageW(self.hwnd_tree, winapi::um::commctrl::TVM_EXPAND, if expanded { winapi::um::commctrl::TVE_EXPAND } else { winapi::um::commctrl::TVE_COLLAPSE }, items[index].native as isize) {
			                common::log_error();
			            }
                	},
                	_ => {}
                }
                
                return;
            } else {
            	parent = Some(items[index].native);
                items = &mut items[index].branches;
            }
        }
        
    }
    fn remove_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize]) {
        let this: &mut Tree = unsafe { utils::base_to_impl_mut(base) };
        let mut items = &mut self.items.0;
        for i in 0..indexes.len() {
            let index = indexes[i];
                
            if i+1 >= indexes.len() {
                let mut item = items.remove(index);
                item.root.on_removed_from_container(this);
                unsafe {
		            if minwindef::TRUE as isize != winuser::SendMessageW(self.hwnd_tree, winapi::um::commctrl::TVM_DELETEITEM, 0, item.native as isize) {
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
            winuser::ShowScrollBar(self.hwnd_tree, winuser::SB_VERT as i32, minwindef::TRUE);
        }
    }
}
impl<O: controls::Tree> NewTreeInner<O> for WindowsTree {
    fn with_uninit(_: &mut mem::MaybeUninit<O>) -> Self {
        WindowsTree {
            base: common::WindowsControlBase::with_wndproc(Some(handler::<O>)),
            hwnd_tree: 0 as windef::HWND,
            items: TreeNodeList(vec![]),
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
	            fn yadder(level: &[TreeNode<*mut winapi::um::commctrl::TREEITEM>], y: &mut i32) {
	                for item in level {
	                    let (_, yy) = item.root.size();
	                    *y += yy as i32;
	                    yadder(item.branches.as_slice(), y);
	                }
	            };
	            yadder(self.items.0.as_slice(), &mut y);        
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
        let (hwnd, hwnd_tree, id) = unsafe {
            self.base.hwnd = parent.native_container_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (width, height, _) = self.measure(member, control, pw, ph);
            let (hwnd, id) = common::create_control_hwnd(
                px,
                py,
                width as i32,
                height as i32,
                self.base.hwnd,
                winuser::WS_EX_CONTROLPARENT | winuser::WS_CLIPCHILDREN,
                WINDOW_CLASS.as_ptr(),
                "",
                0,
                selfptr,
                None,
            );
            let hwnd_tree = winuser::CreateWindowExW(
                0,
                WINDOW_CLASS_TREE.as_ptr(),
                WINDOW_CLASS.as_ptr(),
                winuser::BS_GROUPBOX | winuser::WS_CLIPCHILDREN | winuser::WS_CHILD | winuser::WS_VISIBLE,
                0,
                0,
                width as i32,
                height as i32,
                hwnd,
                ptr::null_mut(),
                common::hinstance(),
                ptr::null_mut(),
            );
            (hwnd, hwnd_tree, id)
        };
        self.base.hwnd = hwnd;
        self.hwnd_tree = hwnd_tree;
        self.base.subclass_id = id;
        control.coords = Some((px, py));
        
        unsafe { winuser::SetWindowLongPtrW(self.hwnd_tree, winuser::GWLP_USERDATA, selfptr as WinPtr); }
        
        let (member, _, adapter, _) = unsafe { Tree::adapter_base_parts_mut(member) };

        let mut y = 0;
        adapter.adapter.for_each(&mut (|indexes, node| {
            self.add_item_inner(member, indexes, node, &mut y);
        }));
        self.force_scrollbar();
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        common::destroy_hwnd(self.hwnd_tree, self.base.subclass_id, None);
        self.base.destroy_control_hwnd();
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
        for child in self.items.0.as_mut_slice() {
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
        for child in self.items.0.as_slice() {
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
    fn native_container_id(&self) -> Self::Id {
        self.hwnd_tree.into()
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
        if let Some((x, y)) = control.coords {
            unsafe {
                winuser::SetWindowPos(self.base.hwnd, ptr::null_mut(), x, y, control.measured.0 as i32, control.measured.1 as i32, 0);
                winuser::SetWindowPos(self.hwnd_tree, ptr::null_mut(), 0, 0, control.measured.0 as i32, control.measured.1 as i32, 0);
            }
        }
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
        unsafe { winuser::RedrawWindow(self.hwnd_tree, ptr::null_mut(), ptr::null_mut(), winuser::RDW_INVALIDATE | winuser::RDW_UPDATENOW) };
        self.base.invalidate();
    }
}
impl Spawnable for WindowsTree {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_adapter(Box::new(types::imp::StringVecAdapter::<crate::imp::Text>::new())).into_control()
    }
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32Tree").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
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
    
    let tree: &mut Tree = mem::transmute(ww);
    let tree2: &mut Tree = mem::transmute(ww);
    tree.inner().inner().inner().inner().inner().base.proc_handler.as_proc().unwrap()(tree2, msg, wparam, lparam)
}

unsafe extern "system" fn handler<T: controls::Tree>(this: &mut Tree, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
	let hwnd = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().native_id().into();
	let hwnd_tree = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().hwnd_tree;
    match msg {
    	winuser::WM_NOTIFY => {
    		match (&*(lparam as winuser::LPNMHDR)).code {
    			winapi::um::commctrl::NM_CUSTOMDRAW => {
    				 let custom_draw = &*(lparam as winapi::um::commctrl::LPNMTVCUSTOMDRAW);
    				 match custom_draw.nmcd.dwDrawStage {               
		                winapi::um::commctrl::CDDS_PREPAINT => return winapi::um::commctrl::CDRF_NOTIFYITEMDRAW,
		                winapi::um::commctrl::CDDS_ITEMPREPAINT => {
		                	let mut drawn = custom_draw.nmcd.dwItemSpec as winapi::um::commctrl::HTREEITEM;
						    
							let mut indexes = Vec::new();
						    
						    let mut retrieve_item = winapi::um::commctrl::TVITEMEXW {
				        		mask: winapi::um::commctrl::TVIF_PARAM,
				        		hItem: drawn,
				        		cchTextMax: 0,
				        		..Default::default()
				        	};
						    
						    let mut parent = None;
			                
			                while {
			                	if 0 == winuser::SendMessageW(hwnd_tree, winapi::um::commctrl::TVM_GETITEMW, 0, &mut retrieve_item as *mut _ as isize) {
			                    	common::log_error();
			                    	parent = None;
			                    } else {
			                    	let parent1 = winuser::SendMessageW(hwnd_tree, winapi::um::commctrl::TVM_GETNEXTITEM, winapi::um::commctrl::TVGN_PARENT, drawn as *mut _ as isize) as *mut winapi::um::commctrl::TREEITEM;
				                    parent = if parent1.is_null() { None } else { Some(parent1) };
			                    }
			                    
			                    let mut i = 0;
			                    let mut index_current = drawn;
			                    while {
			                    	index_current = winuser::SendMessageW(hwnd_tree, winapi::um::commctrl::TVM_GETNEXTITEM, winapi::um::commctrl::TVGN_PREVIOUS, index_current as *mut _ as isize) as *mut winapi::um::commctrl::TREEITEM;
			                    	!index_current.is_null()
			                    } {
				                    i += 1;
			                    }
				                if let Some(parent) = parent {
					                drawn = parent;
					                retrieve_item.hItem = drawn;
				                };
				                indexes.insert(0, i as usize);
				                parent.is_some()
			                } {}
			                
			                let item_view = &mut this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().items;
				            let item = &mut item_view[indexes.as_slice()];
				            
				            let _ = item.root.measure(cmp::max(0, custom_draw.nmcd.rc.right - custom_draw.nmcd.rc.left) as u16, cmp::max(0, custom_draw.nmcd.rc.bottom - custom_draw.nmcd.rc.top) as u16);
			                item.root.draw(Some((custom_draw.nmcd.rc.left + (indexes.len() as i32 * 20), custom_draw.nmcd.rc.top)));
			                //return winapi::um::commctrl::CDRF_SKIPDEFAULT;                 					          
		                }
		                _ => {}
    				 }
    			}
    			winapi::um::commctrl::NM_DBLCLK => {
	    			let mut clicked = winuser::SendMessageW(hwnd_tree, winapi::um::commctrl::TVM_GETNEXTITEM, winapi::um::commctrl::TVGN_CARET, 0) as *mut winapi::um::commctrl::TREEITEM;
		            if clicked.is_null() {
		                common::log_error();
		            } 
				    let mut indexes = Vec::new();
				    
				    let mut retrieve_item = winapi::um::commctrl::TVITEMEXW {
		        		mask: winapi::um::commctrl::TVIF_PARAM,
		        		hItem: clicked,
		        		cchTextMax: 0,
		        		..Default::default()
		        	};
				    
				    let mut parent = None;
	                
	                while {
	                	if 0 == winuser::SendMessageW(hwnd_tree, winapi::um::commctrl::TVM_GETITEMW, 0, &mut retrieve_item as *mut _ as isize) {
	                    	common::log_error();
	                    	parent = None;
	                    } else {
	                    	let parent1 = winuser::SendMessageW(hwnd_tree, winapi::um::commctrl::TVM_GETNEXTITEM, winapi::um::commctrl::TVGN_PARENT, clicked as *mut _ as isize) as *mut winapi::um::commctrl::TREEITEM;
		                    parent = if parent1.is_null() { None } else { Some(parent1) };
	                    }
	                    
	                    let mut i = 0;
	                    let mut index_current = clicked;
	                    while {
	                    	index_current = winuser::SendMessageW(hwnd_tree, winapi::um::commctrl::TVM_GETNEXTITEM, winapi::um::commctrl::TVGN_PREVIOUS, index_current as *mut _ as isize) as *mut winapi::um::commctrl::TREEITEM;
	                    	!index_current.is_null()
	                    } {
		                    i += 1;
	                    }
		                if let Some(parent) = parent {
			                clicked = parent;
			                retrieve_item.hItem = clicked;
		                };
		                indexes.insert(0, i as usize);
		                parent.is_some()
	                } {}
	                
	                let item_view = &mut this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().items;
		            let this = common::member_from_hwnd::<Tree>(hwnd).unwrap();
	                let clicked = &mut item_view[indexes.as_slice()];
			        if let Some(ref mut cb) = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().on_item_click {
			        	let this = common::member_from_hwnd::<Tree>(hwnd).unwrap();
	                    (cb.as_mut())(this, indexes.as_slice(), clicked.root.as_member_mut().is_control_mut().unwrap());
	                }
	    		}
    			_ => {}
    		}
    	}/*
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
        winuser::WM_VSCROLL | winuser::WM_MOUSEWHEEL => {
            winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::FALSE);
        }*/
        winuser::WM_CTLCOLORSTATIC => {
            let hdc = wparam as windef::HDC;
            wingdi::SetTextColor(hdc, wingdi::RGB(0, 0, 0));
            wingdi::SetBkMode(hdc, wingdi::TRANSPARENT as i32);

            return wingdi::GetStockObject(wingdi::NULL_BRUSH as i32) as isize;
        }
        _ => {}
    }

    winuser::DefWindowProcW(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().native_id().into(), msg, wparam, lparam)
}
