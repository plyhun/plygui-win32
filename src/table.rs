use crate::common::{self, *};
use winapi::um::commctrl;

const CLASS_ID: &str = commctrl::WC_LISTVIEW;

lazy_static! {
    pub static ref WINDOW_CLASS_LV: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub type Table = AMember<AControl<AContainer<AAdapted<ATable<WindowsTable>>>>>;

#[repr(C)]
pub struct WindowsTable {
    base: WindowsControlBase<Table>,
    hwnd_lv: windef::HWND,
    data: TableData<WinPtr>,
    on_item_click: Option<callbacks::OnItemClick>,
    width: usize, height: usize,
    col_1_needs_init: bool,
}

impl WindowsTable {
    fn add_column_inner(&mut self, base: &mut MemberBase, index: usize) {
        let (member, control, adapter, _) = unsafe { Table::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().col_1_needs_init |= 1 == index;
        let item = adapter.adapter.spawn_item_view(&[index], this);
        let title = common::string_of_pixel_len(5);
        let mut title = OsStr::new(title.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();

        let lvc = commctrl::LVCOLUMNW {
            mask: commctrl::LVCF_FMT | commctrl::LVCF_WIDTH | commctrl::LVCF_TEXT | commctrl::LVCF_SUBITEM ,
            fmt: commctrl::LVCFMT_LEFT,
            pszText: title.as_mut_ptr(),
            cx: (pw as usize / self.width) as i32,
            iSubItem: index as i32,
            ..Default::default()
        };
        if index as isize != unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_INSERTCOLUMNW, index, &lvc as *const _ as isize) } {
            unsafe { common::log_error(); }
            panic!("Could not insert a table column at index {}", index);
        }
        let hdr_hwnd = unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_GETHEADER, 0, 0) };
        if 0 == hdr_hwnd {
            unsafe { common::log_error(); }
            panic!("Could not get the table header");
        }
        //unsafe { set_header_height(hdr_hwnd as windef::HWND, -1); }
        let hdi = commctrl::HDITEMW {
            mask: commctrl::HDI_FORMAT | commctrl::HDI_DI_SETITEM,
            fmt: commctrl::HDF_OWNERDRAW,
            ..Default::default()
        };
        if 0 == unsafe { winuser::SendMessageW(hdr_hwnd as windef::HWND, commctrl::HDM_SETITEMW, index, &hdi as *const _ as isize) } {
            unsafe { common::log_error(); }
            panic!("Could not insert a column headed at index {}", index);
        }
        self.data.cols.insert(index, TableColumn {
            cells: std::iter::repeat_with(|| None).enumerate().take(self.height).map(|(y, none)| {
                let mut lv = commctrl::LVITEMW {
                    mask: commctrl::LVIF_STATE,
                    stateMask: std::u32::MAX,
                    iItem: y as i32, 
                    ..Default::default()
                };
                if 0 == unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_GETITEMW, 0, &lv as *const _ as isize) } {
                    lv.mask = commctrl::LVIF_PARAM;
                    lv.lParam = item.as_ref().map(|item| unsafe { item.native_id() as isize }).unwrap_or(0);
                    if y as isize != unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_INSERTITEMW, 0, &lv as *const _ as isize) } {
                        unsafe { common::log_error(); }
                        panic!("Could not insert a table row at index [{}, {}]", index, y);
                    } 
                }
                none
            }).collect::<Vec<_>>(),
            control: item.map(|mut item| {
            	let width = utils::coord_to_size(pw as i32 - DEFAULT_PADDING);
            	let height = utils::coord_to_size(ph as i32 - DEFAULT_PADDING);
            	item.on_added_to_container(this, 0, 0, width, height);
                item
            }),
            native: index as isize,
        });
    }
    fn add_cell_inner(&mut self, base: &mut MemberBase, x: usize, y: usize) {
        let (member, _, adapter, _) = unsafe { Table::adapter_base_parts_mut(base) };

        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        adapter.adapter.spawn_item_view(&[x, y], this).map(|mut item| {
            let title = common::string_of_pixel_len(5);
            let mut title = OsStr::new(title.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
            
            let lv = commctrl::LVITEMW {
                mask: commctrl::LVIF_TEXT,// | commctrl::LVIF_PARAM,
                iItem: y as i32, 
                iSubItem: x as i32,
                cchTextMax: title.len() as i32,
                pszText: title.as_mut_ptr(),
                //lParam: unsafe { item.native_id() as isize },
                ..Default::default()
            };
            if 0 == unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETITEMW, 0, &lv as *const _ as isize) } {
                unsafe { common::log_error(); }
                panic!("Could not insert a table cell at index [{}, {}]", x, y);
            } else {
                let mut rc = windef::RECT {
                    left: commctrl::LVIR_BOUNDS,
                	top: lv.iSubItem,
                	..Default::default()
                };
                if 0 == unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_GETSUBITEMRECT, lv.iItem as usize, &mut rc as *mut _ as isize) } {
                    unsafe { common::log_error(); }
                    panic!("Could not get cell rect at index [{}, {}]", x, y);
                }
                let w = utils::coord_to_size(rc.right - rc.left - 2);
                let h = utils::coord_to_size(rc.bottom - rc.top - 2);
                item.set_layout_width(layout::Size::Exact(w));
                item.set_layout_height(layout::Size::Exact(h));
                item.on_added_to_container(this, 0, 0, w, h);
                self.data.cols.get_mut(x).map(|column| {
                    column.cells.insert(y, Some(TableCell {
                        control: Some(item),
                        native: y as isize,
                    }));
                });
            }
        }).unwrap_or_else(|| {});
    }
    fn remove_column_inner(&mut self, member: &mut MemberBase, index: usize) {
        let hwnd = self.base.hwnd;
        self.data.cols.get_mut(index).map(|col| (0..col.cells.len()).rev().for_each(|y| {
            remove_cell_from_col(hwnd, col, member, index, y);
        }));
        if minwindef::TRUE == unsafe { winuser::SendMessageW(self.base.hwnd, commctrl::LVM_DELETECOLUMN, index, 0) as i32 } {
            self.data.cols.remove(index);
        } else {
            panic!("Could not delete column {}", index);
        }
    }
    fn remove_cell_inner(&mut self, member: &mut MemberBase, x: usize, y: usize) {
        let hwnd = self.base.hwnd;
        self.data.cols.get_mut(x).map(|col| {
            remove_cell_from_col(hwnd, col, member, x, y);
        });
    }
    fn change_column_inner(&mut self, base: &mut MemberBase, index: usize) {
        
    }
    fn change_cell_inner(&mut self, base: &mut MemberBase, x: usize, y: usize) {
        
    }
    fn force_scrollbar(&mut self) {
        unsafe {
            winuser::ShowScrollBar(self.hwnd_lv, winuser::SB_VERT as i32, minwindef::TRUE);
        }
    }
    unsafe fn redraw_visible(&mut self) {
    	let color = winuser::GetSysColor(winuser::COLOR_3DFACE);
		winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETBKCOLOR, 0, color as isize);
		winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETTEXTCOLOR, 0, color as isize);
		winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETTEXTBKCOLOR, 0, color as isize);
		
		let (w, _) = common::size_hwnd(self.hwnd_lv);
    	
    	/*let mut rc: windef::RECT = Default::default();
    	
    	unsafe fn redraw_breath(items: &mut Vec<TreeNode<commctrl::HTREEITEM>>, hwnd_tree: windef::HWND, hwnd: windef::HWND, rc: &mut windef::RECT, w: u16) {
    		for item in items {
    			redraw_item(item.native, hwnd_tree, hwnd, rc, None);		                
    			redraw_breath(&mut item.branches, hwnd_tree, hwnd, rc, w);
    		}
    	}
    	redraw_breath(&mut self.items.0, self.hwnd_tree, self.hwnd_lv, &mut rc, w);*/
    }
}
impl<O: controls::Table> NewTableInner<O> for WindowsTable {
    fn with_uninit_params(_: &mut mem::MaybeUninit<O>, width: usize, height: usize) -> Self {
        WindowsTable {
            base: WindowsControlBase::with_wndproc(Some(handler::<O>)),
            hwnd_lv: 0 as windef::HWND,
            data: Default::default(),
            on_item_click: None,
            width, height,
            col_1_needs_init: false,
        }
    }
}
impl TableInner for WindowsTable {
    fn with_adapter_initial_size(adapter: Box<dyn types::Adapter>, width: usize, height: usize) -> Box<dyn controls::Table> {
        let mut b: Box<mem::MaybeUninit<Table>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AAdapted::with_inner(
                        ATable::with_inner(
                            <Self as NewTableInner<Table>>::with_uninit_params(b.as_mut(), width, height)
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
    fn resize(&mut self, member: &mut MemberBase, control: &mut ControlBase, adapted: &mut AdaptedBase, width: usize, height: usize) -> (usize, usize) {
        let oldSize = self.size(member, control, adapted);
        let (maxWidth, maxHeight) = (cmp::max(width, oldSize.0), cmp::max(height, oldSize.1));
        let (minWidth, minHeight) = (cmp::min(width, oldSize.0), cmp::min(height, oldSize.1));
        (minWidth..maxWidth).rev().for_each(|x| 
            if self.data.cols.len() > x {
                if oldSize.0 > x {
                    self.remove_column_inner(member, x);
                }
            } else {
                if oldSize.0 < x {
                     self.add_column_inner(member, x);
                }
            }
        );
        oldSize
    }
}
impl ItemClickableInner for WindowsTable {
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
impl AdaptedInner for WindowsTable {
    fn on_item_change(&mut self, base: &mut MemberBase, value: adapter::Change) {
        if !self.hwnd_lv.is_null() {
            match value {
                adapter::Change::Added(at, node) => {
                    if adapter::Node::Leaf == node || at.len() > 1 {
                        self.add_cell_inner(base, at[0], at[1]);
                    } else {
                        self.add_column_inner(base, at[0]);
                    }
                },
                adapter::Change::Removed(at) => {
                    if at.len() > 1 {
                        self.remove_cell_inner(base, at[0], at[1]);
                    } else {
                        self.remove_column_inner(base, at[0]);
                    }
                },
                adapter::Change::Edited(at, node) => {
                    if adapter::Node::Leaf == node || at.len() > 1 {
                        self.change_cell_inner(base, at[0], at[1]);
                    } else {
                        self.change_column_inner(base, at[0]);
                    }
                },
            }
            self.base.invalidate();
            self.force_scrollbar();
        }
    }
}

impl ControlInner for WindowsTable {
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
        self.base.hwnd = unsafe { parent.native_container_id() as windef::HWND }; // required for measure, as we don't have own hwnd yet
        let (hwnd, hwnd_lv, id) = unsafe {
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
            let hwnd_lv = winuser::CreateWindowExW(
                commctrl::LVS_EX_DOUBLEBUFFER,
                WINDOW_CLASS_LV.as_ptr(),
                WINDOW_CLASS.as_ptr(),
	            winuser::WS_BORDER | winuser::WS_EX_CONTROLPARENT | winuser::WS_CLIPCHILDREN | winuser::WS_VISIBLE | commctrl::LVS_EX_DOUBLEBUFFER/* | commctrl::LVS_NOCOLUMNHEADER */
                     | winuser::WS_EX_CLIENTEDGE | winuser::WS_CHILD | commctrl::LVS_REPORT/* | commctrl::LVS_EX_BORDERSELECT | commctrl::LVS_EX_TRANSPARENTBKGND*/ | commctrl::LVS_OWNERDRAWFIXED ,
                0,
                0,
                width as i32,
                height as i32,
                hwnd,
                ptr::null_mut(),
                common::hinstance(),
                ptr::null_mut(),
            );
            common::set_default_font(hwnd_lv);
            commctrl::SetWindowSubclass(hwnd_lv, Some(ahandler), common::subclass_id(WINDOW_CLASS_LV.as_ptr()) as usize, selfptr as usize);
            (hwnd, hwnd_lv, id)
        };
        self.base.hwnd = hwnd;
        self.hwnd_lv = hwnd_lv;
        self.base.subclass_id = id;
        self.col_1_needs_init |= self.width > 1;
        control.coords = Some((px, py));
        
        if 0 == unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETITEMCOUNT, self.width, commctrl::LVSICF_NOINVALIDATEALL) } {
            unsafe { common::log_error(); }
        }
        //unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETEXTENDEDLISTVIEWSTYLE , 0, (commctrl::LVS_EX_DOUBLEBUFFER | commctrl::LVS_EX_BORDERSELECT | commctrl::LVS_EX_TRANSPARENTBKGND) as isize); }
        unsafe { 
        	winuser::SetWindowLongPtrW(self.hwnd_lv, winuser::GWLP_USERDATA, selfptr as WinPtr); 
        	self.redraw_visible();
        }
        
        let (member, _, adapter, _) = unsafe { Table::adapter_base_parts_mut(member) };

        adapter.adapter.for_each(&mut (|indexes, node| {
            match node {
                adapter::Node::Leaf => self.add_cell_inner(member, indexes[0], indexes[1]),
                adapter::Node::Branch(_) => self.add_column_inner(member, indexes[0])
            }
        }));
        self.force_scrollbar();
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        self.data.cols.iter_mut().flat_map(|col| col.cells.iter_mut()).filter(|cell| cell.is_some()).map(|cell| cell.as_mut().unwrap().control.as_mut()).filter(|cntl| cntl.is_some()).for_each(|mut cntl| cntl.as_mut().unwrap().on_removed_from_container(this));
        self.data.cols.clear();
        common::destroy_hwnd(self.hwnd_lv, self.base.subclass_id, None);
        self.hwnd_lv = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_TABLE;

        fill_from_markup_base!(self, member, markup, registry, Table, [MEMBER_TYPE_TABLE]);
        //fill_from_markup_items!(self, member, markup, registry);
    }
}
impl ContainerInner for WindowsTable {
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        for column in self.data.cols.as_mut_slice() {
            let maybe = column.control.as_mut().and_then(|control| utils::find_by_mut(control.as_mut(), arg));
            if maybe.is_some() {
                return maybe;
            }
            for cell in column.cells.as_mut_slice() {
                if let Some(cell) = cell {
                    let maybe = cell.control.as_mut().and_then(|control| utils::find_by_mut(control.as_mut(), arg));
                    if maybe.is_some() {
                        return maybe;
                    }
                }
            }
        }
        None
    }
    fn find_control<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn controls::Control> {
        for column in self.data.cols.as_slice() {
            let maybe = column.control.as_ref().and_then(|control| utils::find_by(control.as_ref(), arg));
            if maybe.is_some() {
                return maybe;
            }
            for cell in column.cells.as_slice() {
                if let Some(cell) = cell {
                    let maybe = cell.control.as_ref().and_then(|control| utils::find_by(control.as_ref(), arg));
                    if maybe.is_some() {
                        return maybe;
                    }
                }
            }
        }
        None
    }
}
impl HasLayoutInner for WindowsTable {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        let hwnd = self.hwnd_lv;
        if !hwnd.is_null() {
            self.base.invalidate();
        }
    }
}
impl HasNativeIdInner for WindowsTable {
    type Id = common::Hwnd;

    fn native_id(&self) -> Self::Id {
        self.hwnd_lv.into()
    }
}
impl MemberInner for WindowsTable {}

impl HasSizeInner for WindowsTable {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for WindowsTable {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl Drawable for WindowsTable {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        if let Some((x, y)) = control.coords {
            unsafe {
                winuser::SetWindowPos(self.base.hwnd, ptr::null_mut(), x, y, control.measured.0 as i32, control.measured.1 as i32, 0);
                winuser::SetWindowPos(self.hwnd_lv, ptr::null_mut(), 0, 0, control.measured.0 as i32, control.measured.1 as i32, 0);
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
        self.base.invalidate()
    }
}
impl Spawnable for WindowsTable {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_adapter_initial_size(Box::new(types::imp::StringVecAdapter::<crate::imp::Text>::new()), 0, 0).into_control()
    }
}
unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32Table").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
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
unsafe extern "system" fn ahandler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as WinPtr);
    }
    match msg {
        winuser::WM_DRAWITEM => {
            let draw_item = &mut *(lparam as winuser::LPDRAWITEMSTRUCT);
            let this: &mut Table = common::member_from_hwnd(hwnd).expect("Cannot get Table from HWND");
            if this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().col_1_needs_init {
                column_resized(0, hwnd);
                this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().col_1_needs_init = false;
            }
            redraw_row(draw_item.itemID as i32, hwnd, &mut draw_item.rcItem, None);
            return minwindef::TRUE as isize;
        }
        winuser::WM_NOTIFY => {
    		match (&*(lparam as winuser::LPNMHDR)).code {
    		    //commctrl::HDN_BEGINTRACKW | commctrl::HDN_BEGINTRACKA => return minwindef::TRUE as isize, //temporary disable column resize
    		    commctrl::HDN_ITEMCHANGEDW => {
        		    let header = &mut *(lparam as commctrl::LPNMHEADERW);
    				column_resized(header.iItem, hwnd);
    		    }
    		    commctrl::HDN_ITEMCHANGEDA => {
    		        let header = &mut *(lparam as commctrl::LPNMHEADERA);
    				column_resized(header.iItem, hwnd);
    		    }                
    			_ => {}
            }
        }
        winuser::WM_VSCROLL | winuser::WM_MOUSEWHEEL => {
            winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::FALSE);
        }
        _ => {}
    }
    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
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
    let table: &mut Table = mem::transmute(ww);
    let table2: &mut Table = mem::transmute(ww);
    if let Some(wproc) = table.inner().inner().inner().inner().inner().base.proc_handler.as_proc() {
	    wproc(table2, msg, wparam, lparam)
    } else if let Some(whandler) = table.inner().inner().inner().inner().inner().base.proc_handler.as_handler() {
    	whandler(hwnd, msg, wparam, lparam, 0, 0)
    } else {
	    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
unsafe extern "system" fn handler<T: controls::Table>(this: &mut Table, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
	let hwnd = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.hwnd;
	match msg {
        winuser::WM_MEASUREITEM => {
		    let mis: &mut winuser::MEASUREITEMSTRUCT = mem::transmute(lparam);
		    //mis.itemHeight = 5;
	    }
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;
            
            this.call_on_size::<T>(width, height);
            
            let table = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
			table.redraw_visible();
        }
        winuser::WM_VSCROLL | winuser::WM_MOUSEWHEEL => {
            winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::FALSE);
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}
unsafe fn column_resized(y: i32, hwnd: windef::HWND) {
    let width = winuser::SendMessageW(hwnd, commctrl::LVM_GETCOLUMNWIDTH, y as usize, 0) as i32;
    if 1 > width {
        return;
    }
    let this: &mut Table = common::member_from_hwnd(hwnd).expect("Cannot get Table from HWND");
    this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.column_at_mut(y as usize).map(|column| column.cells.iter_mut().for_each(|cell| {
        cell.as_mut().and_then(|cell| cell.control.as_mut()).map(|item| {
            let (_, height) = item.size();
            let width = utils::coord_to_size(width - 2);
            item.set_layout_width(layout::Size::Exact(width));
            item.measure(width, height);
            item.draw(None);
        });
    }));
    /*
    let mut rc = windef::RECT {
        left: commctrl::LVIR_BOUNDS,
    	//top: lv.iSubItem + 1, // 0 stands for the whole row
    	..Default::default()
    };
    if 0 == winuser::SendMessageW(hwnd, commctrl::LVM_GETITEMRECT, y as usize, &mut rc as *mut _ as isize) {
        //common::log_error();
        return;
        //panic!("Could not get column rect at index {}", y);
    }
    let width = utils::coord_to_size(rc.right - rc.left);
    let this: &mut Table = common::member_from_hwnd(hwnd).expect("Cannot get Table from HWND");
    this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.column_at_mut(y as usize).map(|column| column.cells.iter_mut().for_each(|cell| {
        cell.as_mut().and_then(|cell| cell.control.as_mut()).map(|item| {
            let (_, height) = item.size();
            item.set_layout_width(layout::Size::Exact(width));
            item.measure(cmp::max(0, width), height);
            item.draw(Some((rc.left, rc.top)));
        });    
    }));
    */
}
unsafe fn set_header_height(hdr_hwnd: windef::HWND, height: i32) {
    let hdc = winuser::GetDC(hdr_hwnd);
    let hfont = winuser::SendMessageW(hdr_hwnd, winuser::WM_GETFONT, 0, 0);
    let hobject = wingdi::SelectObject(hdc, hfont as *mut c_void);
    let mut logfont = wingdi::LOGFONTW {
        ..Default::default()
    };
    wingdi::GetObjectW(hfont as *mut c_void, mem::size_of::<wingdi::LOGFONTW>() as i32, &mut logfont as *mut _ as *mut c_void);
    logfont.lfHeight = height;
    let hfont = wingdi::CreateFontIndirectW(&logfont);
    wingdi::SelectObject(hdc, hobject as *mut c_void);
    winuser::ReleaseDC(hdr_hwnd, hdc);
    winuser::SendMessageW(hdr_hwnd, winuser::WM_SETFONT, hfont as usize, 0);
    wingdi::DeleteObject(hfont as *mut c_void);
}
unsafe fn redraw_row(y: i32, hwnd: windef::HWND, rc: &mut windef::RECT, action: Option<bool>) {
    let this: &mut Table = common::member_from_hwnd(hwnd).expect("Cannot get Table from HWND");
    this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.column_at_mut(y as usize).map(|column| column.cells.iter_mut().enumerate().for_each(|(x, cell)| {
        let mut drawn = commctrl::LVITEMW {
            mask: commctrl::LVIF_TEXT,// | commctrl::LVIF_PARAM,
            iItem: x as i32, 
            iSubItem: y as i32,
            //lParam: unsafe { item.native_id() as isize },
            ..Default::default()
        };
        if 0 == winuser::SendMessageW(hwnd, commctrl::LVM_GETITEMW, 0, &mut drawn as *mut _ as isize) {
        	return;
        }
        cell.as_mut().and_then(|cell| cell.control.as_mut()).map(|item| {
            rc.left = commctrl::LVIR_BOUNDS;
        	rc.top = drawn.iSubItem;
        	let action = action.unwrap_or(0 != winuser::SendMessageW(hwnd, commctrl::LVM_GETSUBITEMRECT, drawn.iItem as usize, rc as *mut _ as isize));
            if action {
                let this = common::member_from_hwnd::<Table>(hwnd).unwrap();
                let (pw, ph) = this.inner().base.measured;
                let (tw, th, changed) = item.measure(pw, ph);
            	if changed {
            		let mut title = common::wsz_of_pixel_len(tw as usize);
        		    drawn.mask = commctrl::LVIF_TEXT;// | commctrl::LVIF_PARAM,
                    drawn.cchTextMax = title.len() as i32 + 1;
                    drawn.pszText = title.as_mut_ptr();
                    if 0 == winuser::SendMessageW(hwnd, commctrl::LVM_SETITEMW, 0, &drawn as *const _ as isize) {
                        common::log_error();
                        println!("Could not insert a table cell at index [{}, {}]", drawn.iSubItem, drawn.iItem);
                    } else {
                		item.draw(None);
                    }
            	}
                winuser::ShowWindow(item.native_id() as windef::HWND, winuser::SW_SHOW);
                winuser::SetWindowPos(
                	item.native_id() as windef::HWND, 
                	ptr::null_mut(), 
                	rc.left + 1, 
                	rc.top + 1, 
                	cmp::max(tw as i32, rc.right - rc.left), 
                	cmp::max(th as i32, rc.bottom - rc.top), 
                	winuser::SWP_NOSIZE | winuser::SWP_NOSENDCHANGING | winuser::SWP_NOREDRAW);
            } else {
            	 winuser::ShowWindow(item.native_id() as windef::HWND, winuser::SW_HIDE);
            }
        });    
    }));
}
fn remove_cell_from_col<T: Sized>(hwnd: windef::HWND, col: &mut TableColumn<T>, member: &mut MemberBase, x: usize, y: usize) {
    col.cells.get_mut(y).map(|cell| {
        cell.as_mut().map(|cell| cell.control.as_mut().map(|ref mut control| {
            let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
            control.on_removed_from_container(this);
            let lv = commctrl::LVITEMW {
                mask: commctrl::LVIF_TEXT,// | commctrl::LVIF_PARAM,
                iItem: y as i32, 
                iSubItem: x as i32,
                cchTextMax: 0,
                pszText: ptr::null_mut(),
                //lParam: unsafe { item.native_id() as isize },
                ..Default::default()
            };
            if 0 == unsafe { winuser::SendMessageW(hwnd, commctrl::LVM_SETITEMW, 0, &lv as *const _ as isize) } {
                unsafe { common::log_error(); }
                panic!("Could not clear a table cell at index [{}, {}]", x, y);
            }
        }));
    });
    col.cells.remove(y);
}
