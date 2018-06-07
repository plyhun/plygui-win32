use super::*;
use super::common::*;

use plygui_api::{layout, ids, types, controls, utils};
use plygui_api::development::*;

use winapi::shared::windef;
use winapi::shared::minwindef;
use winapi::um::winuser;
use winapi::um::libloaderapi;
use winapi::ctypes::c_void;

use std::{ptr, mem};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

const DEFAULT_PADDING: i32 = 6;

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
	//pub static ref INSTANCE: winuser::HINSTANCE = unsafe { kernel32::GetModuleHandleW(ptr::null()) };
}

pub type LinearLayout = Member<Control<MultiContainer<WindowsLinearLayout>>>;

#[repr(C)]
pub struct WindowsLinearLayout {
    base: WindowsControlBase<LinearLayout>,
    gravity_horizontal: layout::Gravity,
    gravity_vertical: layout::Gravity,
    orientation: layout::Orientation,
    children: Vec<Box<controls::Control>>,
}

impl LinearLayoutInner for WindowsLinearLayout {
	fn with_orientation(orientation: layout::Orientation) -> Box<controls::LinearLayout> {
		use plygui_api::controls::HasLayout;
		
		let mut b = Box::new(Member::with_inner(Control::with_inner(MultiContainer::with_inner(
			WindowsLinearLayout {
				base: WindowsControlBase::new(),
				gravity_horizontal: Default::default(),
			    gravity_vertical: Default::default(),
			    orientation: orientation,
				children: Vec::new(),
			},()), ()),
			MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
		));
		b.set_layout_padding(layout::BoundarySize::AllTheSame(DEFAULT_PADDING).into());
		b
	}
}

impl HasOrientationInner for WindowsLinearLayout {
	fn layout_orientation(&self) -> layout::Orientation {
		self.orientation
	}
    fn set_layout_orientation(&mut self, base: &mut MemberBase, orientation: layout::Orientation) {
    	if orientation != self.orientation {
    		self.orientation = orientation;
	    	let base = self.cast_base_mut(base);
			self.invalidate(base);
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
        	let base = self.cast_base_mut(base);
        	let (_, _, rp, bp) = base.control.layout.padding.into();
	        let (_, _, rm, bm) = base.control.layout.margin.into();
	        self.children.get_mut(index).unwrap().on_added_to_container(common::member_from_hwnd::<LinearLayout>(self.base.hwnd), w as i32 - rp - rm, h as i32 - bp - bm);
	        self.invalidate(base);
        }
        old
    }
    fn remove_child_from(&mut self, base: &mut MemberBase, index: usize) -> Option<Box<controls::Control>> {
        if index < self.children.len() {
            let mut old = self.children.remove(index);
	        if !self.base.hwnd.is_null() {
	        	let base = self.cast_base_mut(base);
	        	old.on_removed_from_container(common::member_from_hwnd::<LinearLayout>(self.base.hwnd));
		        self.invalidate(base);
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
    fn on_added_to_container(&mut self, base: &mut MemberControlBase, parent: &controls::Container, px: i32, py: i32) {
        let selfptr = base as *mut _ as *mut c_void;
        let (pw, ph) = parent.draw_area_size();
        let (width, height, _) = self.measure(base, pw, ph);
        let (lp, tp, _, _) = base.control.layout.padding.into();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            common::create_control_hwnd(
                px as i32 + lm,
                py as i32 + tm,
                width as i32 - rm - lm,
                height as i32 - bm - tm,
                parent.native_id() as windef::HWND,
                winuser::WS_EX_CONTROLPARENT,
                WINDOW_CLASS.as_ptr(),
                "",
                0,
                selfptr,
                None,
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
        self.base.coords = Some((px as i32, py as i32));
        let mut x = lp + lm;
        let mut y = tp + tm;
        for ref mut child in self.children.as_mut_slice() {
            let self2: &mut LinearLayout = unsafe { utils::base_to_impl_mut(&mut base.member) };
            child.on_added_to_container(self2, x, y);
            let (xx, yy) = child.size();
            match self.orientation {
                layout::Orientation::Horizontal => x += xx as i32,
                layout::Orientation::Vertical => y += yy as i32,
            }
        }
    }
    fn on_removed_from_container(&mut self, base: &mut MemberControlBase, _: &controls::Container) {
        for ref mut child in self.children.as_mut_slice() {
            let self2: &mut LinearLayout = unsafe { utils::base_to_impl_mut(&mut base.member) };
            child.on_removed_from_container(self2);
        }
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut MemberControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT;

        fill_from_markup_base!(
            self,
            base,
            markup,
            registry,
            LinearLayout,
            [MEMBER_TYPE_LINEAR_LAYOUT]
        );
        fill_from_markup_children!(self, &mut base.member, markup, registry);
    }
}
impl HasLayoutInner for WindowsLinearLayout {
	fn on_layout_changed(&mut self, base: &mut MemberBase) {
		let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
			let base = self.cast_base_mut(base);
			self.invalidate(base);
		}
	}
}
impl MemberInner for WindowsLinearLayout {
	type Id = common::Hwnd;
	
	fn size(&self) -> (u16, u16) {
        let rect = unsafe { common::window_rect(self.base.hwnd) };
        (
            (rect.right - rect.left) as u16,
            (rect.bottom - rect.top) as u16,
        )
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
	    let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
		    unsafe {
	            winuser::ShowWindow(
	                self.base.hwnd,
	                if base.visibility == types::Visibility::Visible {
	                    winuser::SW_SHOW
	                } else {
	                    winuser::SW_HIDE
	                },
	            );
	        }
			self.invalidate(utils::member_control_base_mut(common::member_from_hwnd::<Button>(hwnd)));
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
    fn gravity(&self) -> (layout::Gravity, layout::Gravity) {
    	(self.gravity_horizontal, self.gravity_vertical)
    }
    fn set_gravity(&mut self, base: &mut MemberBase, w: layout::Gravity, h: layout::Gravity) {
    	if self.gravity_horizontal != w || self.gravity_vertical != h {
    		self.gravity_horizontal = w;
    		self.gravity_vertical = h;
    		self.invalidate(unsafe { mem::transmute(base) });
    	}
    }
}

impl Drawable for WindowsLinearLayout {
    fn draw(&mut self, base: &mut MemberControlBase, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.base.coords = coords;
        }
        let (lp, tp, _, _) = base.control.layout.padding.into();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();
        if let Some((x, y)) = self.base.coords {
            unsafe {
                winuser::SetWindowPos(
                    self.base.hwnd,
                    ptr::null_mut(),
                    x + lm,
                    y + tm,
                    self.base.measured_size.0 as i32 - rm,
                    self.base.measured_size.1 as i32 - bm,
                    0,
                );
            }
            let mut x = lp;
            let mut y = tp;
            for ref mut child in self.children.as_mut_slice() {
                child.draw(Some((x, y)));
                let (xx, yy) = child.size();
                match self.orientation {
                    layout::Orientation::Horizontal => x += xx as i32,
                    layout::Orientation::Vertical => y += yy as i32,
                }
            }
        }
    }
    fn measure(&mut self, base: &mut MemberControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;
    	
    	let orientation = self.orientation;
    	let old_size = self.base.measured_size;
    	let (lp,tp,rp,bp) = base.control.layout.padding.into();
    	let (lm,tm,rm,bm) = base.control.layout.margin.into();
    	let hp = lm + rm + lp + rp;
    	let vp = tm + bm + tp + bp;
    	self.base.measured_size = match base.member.visibility {
        	types::Visibility::Gone => (0,0),
        	_ => {
        		let mut measured = false;
        		let w = match base.control.layout.width {
        			layout::Size::Exact(w) => w,
        			layout::Size::MatchParent => parent_width,
        			layout::Size::WrapContent => {
	        			let mut w = 0;
		                for child in self.children.as_mut_slice() {
		                    let (cw, _, _) = child.measure(
		                    	max(0, parent_width as i32 - hp) as u16, 
		                    	max(0, parent_height as i32 - vp) as u16
		                    );
		                    match orientation {
		                    	layout::Orientation::Horizontal => {
			                    	w += cw;
			                    },
		                    	layout::Orientation::Vertical => {
			                    	w = max(w, cw);
			                    },
		                    }
		                }
	        			measured = true;
	        			max(0, w as i32 + hp) as u16
        			}
        		};
        		let h = match base.control.layout.height {
        			layout::Size::Exact(h) => h,
        			layout::Size::MatchParent => parent_height,
        			layout::Size::WrapContent => {
	        			let mut h = 0;
		                for child in self.children.as_mut_slice() {
		                    let ch = if measured {
		                    	child.size().1
		                    } else {
		                    	let (_, ch, _) = child.measure(
			                    	max(0, parent_width as i32 - hp) as u16, 
			                    	max(0, parent_height as i32 - vp) as u16
			                    );
		                    	ch
		                    };
		                    match orientation {
		                    	layout::Orientation::Horizontal => {
			                    	h = max(h, ch);
			                    },
		                    	layout::Orientation::Vertical => {
			                    	h += ch;
			                    },
		                    }
		                }
	        			max(0, h as i32 + vp) as u16
        			}
        		};
        		(w, h)
        	}
        };
        (
            self.base.measured_size.0,
            self.base.measured_size.1,
            self.base.measured_size != old_size,
        )
    }
    fn invalidate(&mut self, base: &mut MemberControlBase) {
    	self.base.invalidate(base)
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
    let class = winuser::WNDCLASSEXW {
        cbSize: mem::size_of::<winuser::WNDCLASSEXW>() as minwindef::UINT,
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
        hIconSm: ptr::null_mut(),
    };
    winuser::RegisterClassExW(&class);
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
            use plygui_api::controls::Member;
            use std::cmp::max;
    	
	    	let mut width = lparam as u16;
            let mut height = (lparam >> 16) as u16;
            let mut ll: &mut LinearLayout = mem::transmute(ww);
            let o = ll.as_inner().as_inner().as_inner().orientation;
			let (lp, tp, rp, bp) = ll.is_control().unwrap().layout_padding().into();
	        let (lm, tm, rm, bm) = ll.is_control().unwrap().layout_margin().into();
	        let hp = lm + rm + lp + rp;
	    	let vp = tm + bm + tp + bp;
	    	
            let mut x = 0;
            let mut y = 0;
            for child in ll.as_inner_mut().as_inner_mut().as_inner_mut().children.as_mut_slice() {
            	let (cw, ch, _) = child.measure(max(0, width as i32 - hp) as u16, max(0, height as i32 - vp) as u16);
                child.draw(Some((x + lp + lm, y + tp + tm))); 
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

            if let Some(ref mut cb) = ll.base_mut().handler_resize {
                let mut ll2: &mut LinearLayout = mem::transmute(ww);
                (cb.as_mut())(ll2, width, height);
            }
            return 0;
        }
        _ => {}
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl_all_defaults!(LinearLayout);
