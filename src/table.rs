use crate::common::{self, *};
use winapi::um::commctrl;

const CLASS_ID: &str = commctrl::WC_LISTVIEW;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type Table = AMember<AControl<AContainer<AAdapted<ATable<WindowsTable>>>>>;

#[repr(C)]
pub struct WindowsTable {
    base: WindowsControlBase<Table>,
    columns: Vec<TableColumn<WinPtr>>,
    on_item_click: Option<callbacks::OnItemClick>,
    width: usize, height: usize,
}

impl WindowsTable {
    fn add_column_inner(&mut self, base: &mut MemberBase, index: usize) {
        let (member, control, adapter, _) = unsafe { Table::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        
        let item = adapter.adapter.spawn_item_view(&[index], this);
        let title = item.as_ref().and_then(|item| item.is_has_label()).map(|has_label| has_label.label().into_owned()).unwrap_or(format!("{}", index));
        let mut title = OsStr::new(title.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();

        let lvc = commctrl::LVCOLUMNW {
            mask: commctrl::LVCF_FMT | commctrl::LVCF_WIDTH | commctrl::LVCF_TEXT | commctrl::LVCF_SUBITEM,
            fmt: commctrl::LVCFMT_LEFT,
            pszText: title.as_mut_ptr(),
            cx: (pw as usize / self.width) as i32,
            iSubItem: index as i32,
            ..Default::default()
        };
        if index as isize != unsafe { winuser::SendMessageW(self.base.hwnd, commctrl::LVM_INSERTCOLUMNW, index, &lvc as *const _ as isize) } {
            unsafe { common::log_error(); }
            panic!("Could not insert a table column at index {}", index);
        } else {
            self.columns.insert(index, TableColumn {
                cells: std::iter::repeat_with(|| None).take(self.height).collect::<Vec<_>>(),
                control: item.map(|mut item| {
                    item.on_added_to_container(this, 0, 0, utils::coord_to_size(pw as i32 - DEFAULT_PADDING) as u16, utils::coord_to_size(ph as i32 - DEFAULT_PADDING) as u16);
                    item
                }),
                native: index as isize,
            });
        }
    }
    fn add_cell_inner(&mut self, base: &mut MemberBase, x: usize, y: usize) {
        let (member, control, adapter, _) = unsafe { Table::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;

        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        
        adapter.adapter.spawn_item_view(&[x, y], this).map(|mut item| {
            let title = item.is_has_label().map(|has_label| has_label.label().into_owned()).unwrap_or(format!("[{}, {}]", x, y));
            let mut title = OsStr::new(title.as_str()).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
            
            let lv = commctrl::LVITEMW {
                mask: commctrl::LVIF_TEXT,
                iItem: y as i32, 
                iSubItem: x as i32 + 1,
                pszText: title.as_mut_ptr(),
                ..Default::default()
            };
            if 0 == unsafe { winuser::SendMessageW(self.base.hwnd, commctrl::LVM_SETITEMW, 0, &lv as *const _ as isize) } {
                unsafe { common::log_error(); }
               // panic!("Could not insert a table cell at index [{}, {}]", x, y);
            } else {
                item.on_added_to_container(this, 0, 0, utils::coord_to_size(pw as i32 - DEFAULT_PADDING) as u16, utils::coord_to_size(ph as i32 - DEFAULT_PADDING) as u16);
                self.columns.get_mut(x).map(|column| {
                    column.cells.insert(y, Some(TableCell {
                        control: Some(item),
                        native: y as isize,
                    }));
                });
            }
        }).unwrap_or_else(|| {});
    }
    fn remove_column_inner(&mut self, base: &mut MemberBase, index: usize) {
        
    }
    fn remove_cell_inner(&mut self, base: &mut MemberBase, x: usize, y: usize) {
        
    }
    fn change_column_inner(&mut self, base: &mut MemberBase, index: usize) {
        
    }
    fn change_cell_inner(&mut self, base: &mut MemberBase, x: usize, y: usize) {
        
    }
    fn force_scrollbar(&mut self) {
        unsafe {
            winuser::ShowScrollBar(self.base.hwnd, winuser::SB_VERT as i32, minwindef::TRUE);
        }
    }
}
impl<O: controls::Table> NewTableInner<O> for WindowsTable {
    fn with_uninit_params(_: &mut mem::MaybeUninit<O>, width: usize, height: usize) -> Self {
        WindowsTable {
            base: WindowsControlBase::with_handler(Some(handler::<O>)),
            columns: vec![],
            on_item_click: None,
            width, height
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
        if !self.base.hwnd.is_null() {
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
            winuser::WS_EX_CONTROLPARENT | winuser::WS_CLIPCHILDREN | winuser::WS_VISIBLE | winuser::WS_BORDER | winuser::WS_CHILD | winapi::um::commctrl::LVS_REPORT,
            selfptr,
        );
        control.coords = Some((px as i32, py as i32));
        
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
//        for ref mut child in self.columns.as_mut_slice() {
//            let self2: &mut Table = unsafe { utils::base_to_impl_mut(member) };
//            child.on_removed_from_container(self2);
//        }
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_LIST;

        fill_from_markup_base!(self, member, markup, registry, Table, [MEMBER_TYPE_LIST]);
        //fill_from_markup_items!(self, member, markup, registry);
    }
}
impl ContainerInner for WindowsTable {
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        for column in self.columns.as_mut_slice() {
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
        for column in self.columns.as_slice() {
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
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            self.base.invalidate();
        }
    }
}
impl HasNativeIdInner for WindowsTable {
    type Id = common::Hwnd;

    fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
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
impl Spawnable for WindowsTable {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_adapter_initial_size(Box::new(types::imp::StringVecAdapter::<crate::imp::Text>::new()), 0, 0).into_control()
    }
}

unsafe extern "system" fn handler<T: controls::Table>(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as WinPtr);
    }
   /* match msg {
        winuser::WM_LBUTTONUP => {
            let i = winuser::SendMessageW(hwnd, winuser::LB_ITEMFROMPOINT, 0, lparam);
            let table: &mut Table = mem::transmute(param);
            let table_inner = table.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
            let item_view = table_inner.items.get_mut(i as usize).unwrap();
            if let Some(ref mut callback) = table_inner.on_item_click {
                let table: &mut Table = mem::transmute(param);
                (callback.as_mut())(table, &[i as usize], item_view.as_mut());
            }
        }
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let table: &mut Table = mem::transmute(param);
            table.call_on_size::<T>(width, height);
            
            let mut y = 0;
            let i = cmp::max(0, winuser::SendMessageW(hwnd, winuser::LB_GETTOPINDEX, 0, 0)) as usize;
            let table = table.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
            for i in i..table.items.len() {
                let item = &mut table.items[i];
                let (_, ch, _) = item.measure(cmp::max(0, width as i32 - DEFAULT_PADDING) as u16, cmp::max(0, height as i32) as u16);
                item.draw(Some((0, y)));
                y += ch as i32;
            }
            winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::FALSE);
            table.force_scrollbar();
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
    }*/

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}
