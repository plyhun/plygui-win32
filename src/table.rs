use crate::common::{self, matrix::*, *};
use winapi::um::commctrl;

const CLASS_ID: &str = commctrl::WC_LISTVIEW;

lazy_static! {
    pub static ref WINDOW_CLASS_LVHDR: Vec<u16> = OsStr::new("PlyguiListViewHeader").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    pub static ref WINDOW_CLASS_LV: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub type Table = AMember<AControl<AContainer<AAdapted<ATable<WindowsTable>>>>>;

#[repr(C)]
pub struct WindowsTable {
    base: WindowsControlBase<Table>,
    hwnd_lv: windef::HWND,
    data: Matrix<WinPtr>,
    on_item_click: Option<callbacks::OnItemClick>,
    width: usize, height: usize,
    col_1_needs_init: bool,
    custom_row_height: Option<commctrl::HIMAGELIST>
}

impl WindowsTable {
    fn add_row_inner(&mut self, _base: &mut MemberBase, index: usize) -> Option<&mut Row<isize>> {
        let hwnd = self.hwnd_lv;
        let row = Row {
            cells: self.data.cols.iter_mut().enumerate().map(|(y, col)| {
                let mut lv = commctrl::LVITEMW {
                    mask: commctrl::LVIF_STATE,
                    stateMask: std::u32::MAX,
                    iItem: y as i32, 
                    ..Default::default()
                };
                if 0 == unsafe { winuser::SendMessageW(hwnd, commctrl::LVM_GETITEMW, 0, &lv as *const _ as isize) } {
                    lv.mask = commctrl::LVIF_PARAM;
                    lv.lParam = col.native;
                    if y as isize != unsafe { winuser::SendMessageW(hwnd, commctrl::LVM_INSERTITEMW, 0, &lv as *const _ as isize) } {
                        unsafe { common::log_error(); }
                        panic!("Could not insert a table row at index [{}, {}]", index, y);
                    }
                }
                None
            }).collect(),
            native: 0 as WinPtr,
            control: None,
            height: self.data.default_row_height,
        };
        self.data.rows.insert(index, row);
        self.resize_rows(index, self.data.default_row_height, true);
        self.data.row_at_mut(index)
    }
    fn add_column_inner(&mut self, base: &mut MemberBase, index: usize, initial: bool) -> Option<&mut Column<isize>> {
        let (member, control, adapter, _) = unsafe { Table::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().col_1_needs_init |= 1 == index;
        let indices = &[index];
        let item = adapter.adapter.spawn_item_view(indices, this);
        let title = adapter.adapter.alt_text_at(indices);
        let mut title = title.map(|title| OsStr::new(title).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>())
            .unwrap_or_else(|| OsStr::new(common::string_of_pixel_len(5).as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>());
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
        let hdi = commctrl::HDITEMW {
            mask: commctrl::HDI_FORMAT | commctrl::HDI_DI_SETITEM,
            fmt: commctrl::HDF_OWNERDRAW,
            ..Default::default()
        };
        if 0 == unsafe { winuser::SendMessageW(hdr_hwnd as windef::HWND, commctrl::HDM_SETITEMW, index, &hdi as *const _ as isize) } {
            unsafe { common::log_error(); }
            panic!("Could not insert a column headed at index {}", index);
        }
        self.data.cols.insert(index, Column {
            control: item.map(|mut item| {
            	let width = utils::coord_to_size(pw as i32 - DEFAULT_PADDING);
            	let height = utils::coord_to_size(ph as i32 - DEFAULT_PADDING);
                item.set_layout_width(layout::Size::Exact(width));
                item.set_layout_height(self.data.default_row_height);// TODO customize column header height
                item.on_added_to_container(this, 0, 0, width, height);
                item
            }),
            native: index as isize,
            width: layout::Size::MatchParent,
        });
        self.data.rows.iter_mut().for_each(|row| {
            row.cells.insert(index, None);
        });
        self.resize_column(control, index, self.data.cols[index].width, initial);
        self.resize_rows(index, self.data.default_row_height, true);
        self.data.column_at_mut(index)
    }
    fn resize_rows(&mut self, index: usize, size: layout::Size, force: bool) {
        if force || self.data.default_row_height != size {
            let height = match size {
                layout::Size::WrapContent => {
                    let height = self.data.rows.iter()
                        .flat_map(|row| row.cells.iter())
                        .filter(|cell| cell.is_some())
                        .map(|cell| cell.as_ref().unwrap().control.as_ref())
                        .filter(|control| control.is_some())
                        .map(|control| control.unwrap().size().1)
                        .fold(0, |s, i| if s > i {s} else {i});
                    if height > 0 { Some(height) } else { None }
                },
                layout::Size::MatchParent => None,
                layout::Size::Exact(value) => Some(value)
            };
            self.custom_row_height.map(|il| unsafe { 
                    commctrl::ImageList_Destroy(il) 
            }).filter(|res| *res == 0).map(|_| unsafe {common::log_error(); 0}).or_else(|| None);
            self.custom_row_height = height.map(|height| unsafe {
                let il = commctrl::ImageList_Create(1, height as i32, commctrl::ILC_COLOR, 0, 1);
                winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETIMAGELIST, commctrl::LVSIL_SMALL as usize, il as isize);
                il
            }).or(None);
            if !force {
                self.data.row_at_mut(index).map(|row| row.height = size);
            }
        } else {
            let row_height = self.data.default_row_height;
            self.data.row_at_mut(index).map(|mut row| row.height = row_height);
        }
    }
    fn resize_column(&mut self, base: &ControlBase, index: usize, size: layout::Size, skip_match_parent: bool) {
        let col_1_needs_init = self.col_1_needs_init;
        match size {
            layout::Size::Exact(width) => {
                if minwindef::TRUE != unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETCOLUMNWIDTH, index, width as isize) as i32 } {
                    unsafe { common::log_error(); }
                    panic!("Could not resize a table column at index [{}] to {}px", index, width);
                }
            },
            layout::Size::WrapContent => {
                if minwindef::TRUE != unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETCOLUMNWIDTH, index, commctrl::LVSCW_AUTOSIZE as isize) as i32 } {
                    unsafe { common::log_error(); }
                    panic!("Could not resize a table column at index [{}] to fit content", index);
                }
            },
            layout::Size::MatchParent => {
                if skip_match_parent {
                    // evenly distributed by default
                } else {
                    let width = base.measured.0 / self.data.cols.len() as u16; // must be > 0
                    (0..self.data.cols.len()).for_each(|x| {
                        if minwindef::TRUE != unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETCOLUMNWIDTH, x, width as isize) as i32 } {
                            unsafe { common::log_error(); }
                            panic!("Could not resize a table column at index [{}] to {}px", index, width);
                        }
                    });
                }
            },
        }
        self.data.column_at_mut(index).map(|col| col.width = size);
        self.col_1_needs_init = col_1_needs_init;
    }
    fn add_cell_inner(&mut self, base: &mut MemberBase, x: usize, y: usize) {
        if self.data.row_at_mut(x).is_none() {
            self.add_row_inner(base, x);
        }
        let (member, control, adapter, _) = unsafe { Table::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
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
                let row_height = self.data.rows.get_mut(x).map(|row| {
                    item.set_layout_width(layout::Size::Exact(w));
                    item.set_layout_height(row.height);
                    item.on_added_to_container(this, 0, 0, pw, ph);
                    row.cells.insert(y, Some(Cell {
                        control: Some(item),
                        native: y as isize,
                    }));
                    row.height
                }).unwrap_or(self.data.default_row_height);
                self.resize_rows(y, row_height, true);
            }
        }).unwrap_or_else(|| {});
    }
    fn remove_column_inner(&mut self, member: &mut MemberBase, index: usize) {
        let hwnd = self.base.hwnd;
        self.data.rows.iter_mut().enumerate().for_each(|(x, row)| {
            remove_cell_from_row(hwnd, row, member, x, index);
        });
        if minwindef::TRUE == unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_DELETECOLUMN, index, 0) as i32 } {
            self.data.cols.remove(index);
        } else {
            panic!("Could not delete column {}", index);
        }
    }
    fn remove_row_inner(&mut self, member: &mut MemberBase, index: usize) {
        let hwnd = self.base.hwnd;
        self.data.rows.get_mut(index).map(|row| (0..row.cells.len()).rev().for_each(|y| {
            remove_cell_from_row(hwnd, row, member, index, y);
        }));
        self.data.rows.remove(index);
    }
    fn remove_cell_inner(&mut self, member: &mut MemberBase, x: usize, y: usize) {
        let hwnd = self.base.hwnd;
        self.data.rows.get_mut(x).map(|row| {
            remove_cell_from_row(hwnd, row, member, x, y);
        });
    }
    fn change_column_inner(&mut self, base: &mut MemberBase, index: usize) {
        self.remove_column_inner(base, index);
        self.add_column_inner(base, index, false);
    }
    fn change_cell_inner(&mut self, base: &mut MemberBase, x: usize, y: usize) {
        self.remove_cell_inner(base, x, y);
        self.add_cell_inner(base, x, y);
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
		
		/*let (w, _) = common::size_hwnd(self.hwnd_lv);
    	
    	let mut rc: windef::RECT = Default::default();
    	
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
            custom_row_height: None,
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
    fn set_column_width(&mut self, _: &mut MemberBase, control: &mut ControlBase, _: &mut AdaptedBase, index: usize, size: layout::Size) {
        self.resize_column(control, index, size, false)
    }
    fn set_row_height(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &mut AdaptedBase, index: usize, size: layout::Size) {
        self.resize_rows(index, size, false)
    }
   /* fn resize(&mut self, member: &mut MemberBase, control: &mut ControlBase, adapted: &mut AdaptedBase, width: usize, height: usize) -> (usize, usize) {
        let old_size = self.size(member, control, adapted);
        let (max_width, max_height) = (cmp::max(width, old_size.0), cmp::max(height, old_size.1));
        let (min_width, min_height) = (cmp::min(width, old_size.0), cmp::min(height, old_size.1));
        (min_width..max_width).rev().for_each(|x| 
            if self.data.cols.len() > x {
                if old_size.0 > x {
                    self.remove_column_inner(member, x);
                }
            } else {
                if old_size.0 < x {
                     self.add_column_inner(member, x, false);
                }
            }
        );
        (0..self.data.rows.len()).for_each(|x| {
            let height = self.data.rows[x].cells.len();
            (min_height..max_height).rev().for_each(|y| 
                if height > y {
                    if old_size.1 > y {
                        self.remove_cell_inner(member, x, y);
                    }
                } else {
                    if old_size.1 < y {
                         self.add_cell_inner(member, x, y);
                    }
                }
            );
        });
        old_size
    } */
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
                        self.add_column_inner(base, at[0], false);
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
	            winuser::WS_BORDER | winuser::WS_EX_CONTROLPARENT | winuser::WS_CLIPCHILDREN | winuser::WS_VISIBLE/* | commctrl::LVS_EX_FULLROWSELECT*/ 
                    | commctrl::LVS_NOSORTHEADER/* | commctrl::LVS_NOCOLUMNHEADER*/ | commctrl::LVS_REPORT | winuser::WS_CHILD | commctrl::LVS_OWNERDRAWFIXED ,
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
            winuser::SendMessageW(hwnd_lv, commctrl::LVM_SETEXTENDEDLISTVIEWSTYLE, 0, (commctrl::LVS_EX_DOUBLEBUFFER | winuser::WS_EX_CLIENTEDGE | commctrl::LVS_EX_BORDERSELECT | commctrl::LVS_EX_TRANSPARENTBKGND) as isize);
            commctrl::SetWindowSubclass(hwnd_lv, Some(ahandler), common::subclass_id(WINDOW_CLASS_LV.as_ptr()) as usize, selfptr as usize);
            let hwnd_hdr = winuser::SendMessageW(hwnd_lv, commctrl::LVM_GETHEADER, 0, 0) as windef::HWND;
            commctrl::SetWindowSubclass(hwnd_hdr, Some(hdrhandler), common::subclass_id(WINDOW_CLASS_LVHDR.as_ptr()) as usize, selfptr as usize);
            (hwnd, hwnd_lv, id)
        };
        self.base.hwnd = hwnd;
        self.hwnd_lv = hwnd_lv;
        self.base.subclass_id = id;
        self.col_1_needs_init |= self.width > 1;
        //self.data.default_row_height = layout::Size::Exact(50);
        control.coords = Some((px, py));
        
        if 0 == unsafe { winuser::SendMessageW(self.hwnd_lv, commctrl::LVM_SETITEMCOUNT, self.width, commctrl::LVSICF_NOINVALIDATEALL) } {
            unsafe { common::log_error(); }
        }
        unsafe { 
        	winuser::SetWindowLongPtrW(self.hwnd_lv, winuser::GWLP_USERDATA, selfptr as WinPtr); 
        	self.redraw_visible();
        }
        
        let (member, _, adapter, _) = unsafe { Table::adapter_base_parts_mut(member) };

        adapter.adapter.for_each(&mut (|indexes, node| {
            match node {
                adapter::Node::Leaf => { self.add_cell_inner(member, indexes[0], indexes[1]); },
                adapter::Node::Branch(_) => { self.add_column_inner(member, indexes[0], true); }
            }
        }));
        self.resize_rows(0, self.data.default_row_height, true);
        self.force_scrollbar();
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        self.data.cols.iter_mut().filter(|col| col.control.is_some()).map(|col| col.control.as_mut().unwrap()).for_each(|cntl| cntl.on_removed_from_container(this));
        self.data.cols.clear();
        self.data.rows.iter_mut().flat_map(|col| col.cells.iter_mut()).filter(|cell| cell.is_some()).map(|cell| cell.as_mut().unwrap().control.as_mut()).filter(|cntl| cntl.is_some()).for_each(|mut cntl| cntl.as_mut().unwrap().on_removed_from_container(this));
        self.data.rows.clear();
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
        }
        for row in self.data.rows.as_mut_slice() {
            let maybe = row.control.as_mut().and_then(|control| utils::find_by_mut(control.as_mut(), arg));
            if maybe.is_some() {
                return maybe;
            }
            for cell in row.cells.as_mut_slice() {
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
        }
        for row in self.data.rows.as_slice() {
            let maybe = row.control.as_ref().and_then(|control| utils::find_by(control.as_ref(), arg));
            if maybe.is_some() {
                return maybe;
            }
            for cell in row.cells.as_slice() {
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
unsafe extern "system" fn hdrhandler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as WinPtr);
    }
    match msg {
        commctrl::HDM_LAYOUT => {
            let this: &mut Table = common::member_from_hwnd(hwnd).expect("Cannot get Table from Header HWND");
            let hl = &mut *(lparam as commctrl::LPHDLAYOUT);
            let rect = &mut *(hl.prc);
            let pos = &mut *(hl.pwpos);
            let r = commctrl::DefSubclassProc(hwnd, msg, wparam, lparam);
            pos.cy = 0;
            this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.cols.iter_mut().enumerate().for_each(|(x, col)| {
                col.control.as_mut().map(|col| {
                    let (_, ch) = col.size();
                    pos.cy = cmp::max(pos.cy, ch as i32);
                });
            });
            rect.top = pos.cy;
            r
        }
        _ => commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
    }
}
unsafe extern "system" fn ahandler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as WinPtr);
    }
    match msg {
        winuser::WM_DRAWITEM => {
            let draw_item = &mut *(lparam as winuser::LPDRAWITEMSTRUCT);
            match draw_item.CtlType {
                commctrl::ODT_HEADER => {
                    column_resized(draw_item.itemID as i32, hwnd, true);
                }
                _ => {
                    let this: &mut Table = common::member_from_hwnd(hwnd).expect("Cannot get Table from HWND");
                    if this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().col_1_needs_init {
                        column_resized(0, hwnd, false);
                        this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().col_1_needs_init = false;
                    }
                    redraw_row(draw_item.itemID as i32, hwnd, &mut draw_item.rcItem, None);
                }
            }
            return minwindef::TRUE as isize;
        }
        winuser::WM_NOTIFY => {
    		match (&*(lparam as winuser::LPNMHDR)).code {
    		    //commctrl::HDN_BEGINTRACKW | commctrl::HDN_BEGINTRACKA => return minwindef::TRUE as isize, //temporary disable column resize
    		    commctrl::HDN_ITEMCHANGEDW => {
        		    let header = &mut *(lparam as commctrl::LPNMHEADERW);
    				column_resized(header.iItem, hwnd, false);
    		    }
    		    commctrl::HDN_ITEMCHANGEDA => {
    		        let header = &mut *(lparam as commctrl::LPNMHEADERA);
    				column_resized(header.iItem, hwnd, false);
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
    table.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.handle(msg, wparam, lparam, hwnd)
}
unsafe extern "system" fn handler<T: controls::Table>(this: &mut Table, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
	let hwnd = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.hwnd;
	match msg {
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
unsafe fn column_resized(x: i32, hwnd: windef::HWND, full_redraw: bool) {
    if full_redraw {
        let mut rc = windef::RECT::default();
        redraw_column(x, hwnd, &mut rc, Some(true));
    } else {
        let width = winuser::SendMessageW(hwnd, commctrl::LVM_GETCOLUMNWIDTH, x as usize, 0) as i32;
        if 1 > width {
            return;
        }
        let this: &mut Table = common::member_from_hwnd(hwnd).expect("Cannot get Table from HWND");
        let header_height = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.header_height;
        this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.column_at_mut(x as usize).and_then(|col| col.control.as_mut()).map(|item| {
            let height = match header_height {
                layout::Size::Exact(height) => height,
                _ => item.size().1
            };
            let width = utils::coord_to_size(width - 2);
            item.set_layout_width(layout::Size::Exact(width));
            item.set_layout_height(header_height);
            item.measure(width, height);
            item.draw(None);
        });
        this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.row_at_mut(x as usize).map(|row| {
            let row_height = row.height;
            row.cells.iter_mut().for_each(|cell| {
                cell.as_mut().and_then(|cell| cell.control.as_mut()).map(|item| {
                    let height = match row_height {
                        layout::Size::Exact(height) => height,
                        _ => item.size().1
                    };
                    let width = utils::coord_to_size(width - 2);
                    item.set_layout_width(layout::Size::Exact(width));
                    item.set_layout_height(row_height);
                    item.measure(width, height);
                    item.draw(None);
                });
            })
        });
    }
}
unsafe fn redraw_row(y: i32, hwnd: windef::HWND, rc: &mut windef::RECT, action: Option<bool>) {
    let this: &mut Table = common::member_from_hwnd(hwnd).expect("Cannot get Table from HWND");
    this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.rows.iter_mut().enumerate().for_each(|(x, row)| {
        let row_height = row.height;
        redraw_cell(row.cell_at_mut(y as usize), x as i32, y as i32, hwnd, rc, action, row_height)
    });
}
unsafe fn redraw_column(x: i32, hwnd: windef::HWND, rc: &mut windef::RECT, action: Option<bool>) {
    let this: &mut Table = common::member_from_hwnd(hwnd).expect("Cannot get Table from HWND");
    let header_height = this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.header_height;
    redraw_header(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.column_at_mut(x as usize), x, hwnd, rc, action, header_height);
    this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.rows.iter_mut().enumerate().for_each(|(y, row)| {
        let row_height = row.height;
        redraw_cell(row.cell_at_mut(x as usize), x, y as i32, hwnd, rc, action, row_height);
    });
}
fn redraw_header<T: Sized>(col: Option<&mut Column<T>>, x: i32, hwnd: windef::HWND, rc: &mut windef::RECT, action: Option<bool>, header_height: layout::Size) {
    let hdr_hwnd = unsafe { winuser::SendMessageW(hwnd, commctrl::LVM_GETHEADER, 0, 0) } as windef::HWND;
    if hdr_hwnd.is_null() {
        unsafe { common::log_error(); }
        panic!("Could not get the table header");
    }
    let mut drawn = commctrl::HDITEMW {
        mask: commctrl::HDI_TEXT | commctrl::HDI_WIDTH,// | commctrl::LVIF_PARAM,
        //lParam: unsafe { item.native_id() as isize },
        ..Default::default()
    }; 
    if 0 == unsafe { winuser::SendMessageW(hdr_hwnd, commctrl::HDM_GETITEMW, x as usize, &mut drawn as *mut _ as isize) } {
    	return;
    }
    col.and_then(|cell| cell.control.as_mut()).map(|item| {
        let action = action.unwrap_or(0 != unsafe { winuser::SendMessageW(hdr_hwnd, commctrl::HDM_GETITEMRECT, x as usize, rc as *mut _ as isize) });
        if action {
            let (width, mut height) = item.size();
            if let layout::Size::Exact(row_height) = header_height {
                height = row_height;
            };
            item.set_layout_width(layout::Size::Exact(drawn.cxy as u16 - 2));
            item.set_layout_height(header_height);
            let (tw, th, changed) = item.measure(width, height);
            if changed {
        		let mut title = common::wsz_of_pixel_len(tw as usize);
    		    drawn.mask = commctrl::LVIF_TEXT;// | commctrl::LVIF_PARAM,
                drawn.cchTextMax = title.len() as i32 + 1;
                drawn.pszText = title.as_mut_ptr();
                if 0 == unsafe { winuser::SendMessageW(hdr_hwnd, commctrl::HDM_SETITEMW, x as usize, &drawn as *const _ as isize) } {
                    unsafe { common::log_error(); }
                    println!("Could not insert a table header at index [{}]", x);
                } else {
            		item.draw(None);
                }
        	}
            unsafe {
                winuser::ShowWindow(item.native_id() as windef::HWND, winuser::SW_SHOW);
                winuser::SetWindowPos(
                    item.native_id() as windef::HWND, 
                    ptr::null_mut(), 
                    rc.left + 1, 
                    rc.top + 1, 
                    cmp::max(tw as i32, rc.right - rc.left), 
                    cmp::max(th as i32, rc.bottom - rc.top), 
                    winuser::SWP_NOSIZE | winuser::SWP_NOSENDCHANGING | winuser::SWP_NOREDRAW);
            }
        } else {
            unsafe { winuser::ShowWindow(item.native_id() as windef::HWND, winuser::SW_HIDE); }
        } 
    });
}
fn redraw_cell<T: Sized>(cell: Option<&mut Cell<T>>, x: i32, y: i32, hwnd: windef::HWND, rc: &mut windef::RECT, action: Option<bool>, row_height: layout::Size) {
    let mut drawn = commctrl::LVITEMW {
        mask: commctrl::LVIF_TEXT,// | commctrl::LVIF_PARAM,
        iItem: y, 
        iSubItem: x,
        //lParam: unsafe { item.native_id() as isize },
        ..Default::default()
    };
    if 0 == unsafe { winuser::SendMessageW(hwnd, commctrl::LVM_GETITEMW, 0, &mut drawn as *mut _ as isize) } {
    	return;
    }
    cell.and_then(|cell| cell.control.as_mut()).map(|item| {
        rc.left = commctrl::LVIR_BOUNDS;
    	rc.top = drawn.iSubItem;
    	let action = action.unwrap_or(0 != unsafe { winuser::SendMessageW(hwnd, commctrl::LVM_GETSUBITEMRECT, drawn.iItem as usize, rc as *mut _ as isize) });
        if action {
            let (width, mut height) = item.size();
            if let layout::Size::Exact(row_height) = row_height {
               height = row_height;
            };
            let (tw, th, changed) = item.measure(width, height);
        	if changed {
        		let mut title = common::wsz_of_pixel_len(tw as usize);
    		    drawn.mask = commctrl::LVIF_TEXT;// | commctrl::LVIF_PARAM,
                drawn.cchTextMax = title.len() as i32 + 1;
                drawn.pszText = title.as_mut_ptr();
                if 0 == unsafe { winuser::SendMessageW(hwnd, commctrl::LVM_SETITEMW, 0, &drawn as *const _ as isize) } {
                    unsafe { common::log_error(); }
                    println!("Could not insert a table cell at index [{}, {}]", drawn.iSubItem, drawn.iItem);
                } else {
            		item.draw(None);
                }
        	}
            unsafe {
                winuser::ShowWindow(item.native_id() as windef::HWND, winuser::SW_SHOW);
                winuser::SetWindowPos(
                	item.native_id() as windef::HWND, 
                	ptr::null_mut(), 
                	rc.left + 1, 
                	rc.top + 1, 
                	cmp::max(tw as i32, rc.right - rc.left), 
                	cmp::max(th as i32, rc.bottom - rc.top), 
                	winuser::SWP_NOSIZE | winuser::SWP_NOSENDCHANGING | winuser::SWP_NOREDRAW);
            }
        } else {
        	 unsafe { winuser::ShowWindow(item.native_id() as windef::HWND, winuser::SW_HIDE); }
        } 
    });
}
fn remove_cell_from_row<T: Sized>(hwnd: windef::HWND, row: &mut Row<T>, member: &mut MemberBase, x: usize, y: usize) {
    row.cells.get_mut(y).map(|cell| {
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
    row.cells.remove(y);
}
