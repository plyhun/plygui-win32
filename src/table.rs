use crate::common::{self, *};

const CLASS_ID: &str = ::winapi::um::commctrl::WC_LISTVIEW;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type Table = Member<Control<MultiContainer<WindowsTable>>>;

struct WindowsTableRow {
	cols: Vec<Box<dyn controls::Control>>,
}

impl Clone for WindowsTableRow {
	fn clone(&self) -> Self {
		WindowsTableRow {
			cols: Vec::with_capacity(self.cols.len())
		}
	}
}

#[repr(C)]
pub struct WindowsTable {
    base: WindowsControlBase<Table>,
    rows: Vec<WindowsTableRow>,
    cols_len: usize,
}

impl TableInner for WindowsTable {
    fn with_dimensions(rows: usize, cols: usize) -> Box<Table> {
        let b = Box::new(Member::with_inner(
            Control::with_inner(
                MultiContainer::with_inner(
                    WindowsTable {
                        base: WindowsControlBase::new(),
                        rows: vec![WindowsTableRow {
		                        cols: Vec::with_capacity(cols)
	                        }; rows],
                        cols_len: cols,
                    },
                    (),
                ),
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        b
    }
    fn row_len(&self) -> usize { self.rows.len() }
    fn column_len(&self) -> usize { self.cols_len }
    fn table_child_at(&self, row: usize, col: usize) -> Option<&dyn controls::Control> { 
    	if self.rows.get(row).is_some() {
    		self.rows.get(row).map(|row| row.cols.get(col).map(|c| c.as_ref())).unwrap() 
    	} else {
    		None
    	}
    }
    fn table_child_at_mut(&mut self, row: usize, col: usize) -> Option<&mut dyn controls::Control> { 
    	if self.rows.get(row).is_some() {
    		self.rows.get_mut(row).map(|row| row.cols.get_mut(col).map(|c| c.as_mut())).unwrap()
    	} else {
    		None
    	}
    }
    
    fn set_table_child_to(&mut self, base: &mut MemberBase, row: usize, col: usize, child: Box<dyn controls::Control>) -> Option<Box<dyn controls::Control>> {
	    None
    }
    fn remove_table_child_from(&mut self, base: &mut MemberBase, row: usize, col: usize) -> Option<Box<dyn controls::Control>> {
	    None
    }
    
    fn add_row(&mut self) -> usize { 0 }
    fn add_column(&mut self) -> usize { 
	    if !self.base.hwnd.is_null() {
	    	let mut col: commctrl::LVCOLUMNW = unsafe { mem::zeroed() };
	    	col.mask = commctrl::LVCF_FMT | commctrl::LVCF_WIDTH;
	    	col.fmt = commctrl::LVCFMT_CENTER | commctrl::LVCFMT_FIXED_WIDTH;
	    	
	    	unsafe { if 0 > winuser::SendMessageW(self.base.hwnd, commctrl::LVM_INSERTCOLUMNW, self.cols_len, mem::transmute(&col)) {
		    	common::log_error();
	    	}};
	    }
	    self.cols_len += 1;
	    self.cols_len
    }
    fn insert_row(&mut self, row: usize) -> usize { 0 }
    fn insert_column(&mut self, col: usize) -> usize { 0 }
    fn delete_row(&mut self, row: usize) -> usize { 0 }
    fn delete_column(&mut self, col: usize) -> usize { 0 }
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
                winuser::WS_EX_CONTROLPARENT | winuser::WS_CLIPCHILDREN | commctrl::LVS_REPORT | commctrl::LVS_OWNERDRAWFIXED | commctrl::LVS_EX_DOUBLEBUFFER,
                selfptr,
                Some(handler),
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
        control.coords = Some((px as i32, py as i32));
        
        for col in 0..self.cols_len {
        	self.add_column();
        }
        
        let mut x = DEFAULT_PADDING;
        let mut y = DEFAULT_PADDING;
        for row in self.rows.as_mut_slice() {
	        for child in row.cols.as_mut_slice() {
	            let self2: &mut Table = unsafe { utils::base_to_impl_mut(member) };
	            child.on_added_to_container(
	                self2,
	                x,
	                y,
	                utils::coord_to_size(pw as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16,
	                utils::coord_to_size(ph as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16,
	            );
	            let (xx, yy) = child.size();
	            /*match self.orientation {
	                layout::Orientation::Horizontal => x += xx as i32,
	                layout::Orientation::Vertical => y += yy as i32,
	            }*/
	        }
        }
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        for row in self.rows.as_mut_slice() {
	        for child in row.cols.as_mut_slice() {
	            let self2: &mut Table = unsafe { utils::base_to_impl_mut(member) };
	            child.on_removed_from_container(self2);
	        }
        }
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_TABLE;

        fill_from_markup_base!(self, member, markup, registry, Table, [MEMBER_TYPE_TABLE]);
        fill_from_markup_children!(self, member, markup, registry);
    }
}
impl HasLayoutInner for WindowsTable {
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
impl HasNativeIdInner for WindowsTable {
    type Id = common::Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}
impl MemberInner for WindowsTable {}

impl HasSizeInner for WindowsTable {
    fn on_size_set(&mut self, base: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        use plygui_api::controls::HasLayout;

        let this = base.as_any_mut().downcast_mut::<Table>().unwrap();
        this.set_layout_width(layout::Size::Exact(width));
        this.set_layout_width(layout::Size::Exact(height));
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for WindowsTable {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl ContainerInner for WindowsTable {
    fn find_control_mut(&mut self, arg: types::FindBy) -> Option<&mut dyn controls::Control> {
        for row in self.rows.as_mut_slice() {
	        for child in row.cols.as_mut_slice() {
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
        }
        None
    }
    fn find_control(&self, arg: types::FindBy) -> Option<&dyn controls::Control> {
        for row in self.rows.as_slice() {
	        for child in row.cols.as_slice() {
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
        }
        None
    }
}

impl Drawable for WindowsTable {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
        let mut x = DEFAULT_PADDING;
        let mut y = DEFAULT_PADDING;
        for row in self.rows.as_mut_slice() {
	        for child in row.cols.as_mut_slice() {
	        	child.draw(Some((x, y)));
	        	/*let (xx, yy) = child.size();
	            match self.orientation {
	                layout::Orientation::Horizontal => x += xx as i32,
	                layout::Orientation::Vertical => y += yy as i32,
	            }*/
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
                    layout::Size::WrapContent => {
                        defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => {
                        defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING
                    }
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

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<dyn controls::Control> {
    Table::with_dimensions(0, 0).into_control()
}

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let table: &mut Table = mem::transmute(param);
            table.call_on_size(width, height);
            return 0;
        }
        winuser::WM_MEASUREITEM => {}
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}

default_impls_as!(Table);
