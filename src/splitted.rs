use super::*;

use plygui_api::{development, layout, ids, types, controls, utils};
use plygui_api::development::{Drawable, HasInner};

use winapi::shared::windef;
use winapi::shared::minwindef;
use winapi::um::winuser;
use winapi::um::libloaderapi;
use winapi::ctypes::c_void;

use std::{ptr, mem, cmp};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

const DEFAULT_PADDING: i32 = 6;
const DEFAULT_BOUND: i32 = DEFAULT_PADDING * 2;
const HALF_BOUND: i32 = DEFAULT_BOUND / 2;

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub type Splitted = development::Member<development::Control<development::MultiContainer<WindowsSplitted>>>;

#[repr(C)]
pub struct WindowsSplitted {
    base: common::WindowsControlBase<Splitted>,
    
    gravity_horizontal: layout::Gravity,
    gravity_vertical: layout::Gravity,
    orientation: layout::Orientation,
    
    splitter: f32,
    moving: bool,
    cursor: windef::HCURSOR,
    
    first: Box<controls::Control>, 
    second: Box<controls::Control>, 
}

impl WindowsSplitted {
	fn reload_cursor(&mut self) {
		unsafe { 
			if !self.cursor.is_null() && winuser::DestroyCursor(self.cursor) == 0 {
				common::log_error();
			}
		}
		self.cursor = unsafe { 
        	winuser::LoadCursorW(ptr::null_mut(), match self.orientation {
		        layout::Orientation::Horizontal => winuser::IDC_SIZEWE,
		        layout::Orientation::Vertical => winuser::IDC_SIZENS,
	        }) 
        };
	}
	fn update_children_layout(&mut self) {
		use plygui_api::controls::Container;
		
		if self.base.hwnd.is_null() { return }
		
		let self2 = common::member_from_hwnd::<Splitted>(self.base.hwnd);
		
		let (width, height) = self2.draw_area_size();
		let orientation = self.orientation;
		self.first.set_skip_draw(true);
		self.second.set_skip_draw(true);
		
		match orientation {
			layout::Orientation::Horizontal => {
				let splitter_pos = cmp::min(width as i32, (width as f32 * self.splitter) as i32);
				self.first.set_layout_width(layout::Size::Exact(cmp::max(DEFAULT_BOUND, splitter_pos - HALF_BOUND) as u16));
				self.second.set_layout_width(layout::Size::Exact(cmp::max(DEFAULT_BOUND, width as i32 - splitter_pos - HALF_BOUND) as u16));
				self.first.set_layout_height(layout::Size::MatchParent);
				self.second.set_layout_height(layout::Size::MatchParent);
			},
			layout::Orientation::Vertical => {
				let splitter_pos = cmp::min(height as i32, (height as f32 * self.splitter) as i32);
				self.first.set_layout_width(layout::Size::MatchParent);
				self.second.set_layout_width(layout::Size::MatchParent);
				self.first.set_layout_height(layout::Size::Exact(cmp::max(DEFAULT_BOUND, splitter_pos - HALF_BOUND) as u16));
				self.second.set_layout_height(layout::Size::Exact(cmp::max(DEFAULT_BOUND, height as i32 - splitter_pos - HALF_BOUND) as u16));
			},
		}
		self.first.set_skip_draw(false);
		self.second.set_skip_draw(false);
	}
}

impl development::SplittedInner for WindowsSplitted {
	fn with_content(first: Box<controls::Control>, second: Box<controls::Control>, orientation: layout::Orientation) -> Box<controls::Splitted> {
		use plygui_api::controls::{HasLayout};
		
		let mut b = Box::new(development::Member::with_inner(development::Control::with_inner(development::MultiContainer::with_inner(
			WindowsSplitted {
				base: common::WindowsControlBase::new(),
				gravity_horizontal: Default::default(),
			    gravity_vertical: Default::default(),
			    orientation: orientation,
				
				splitter: 0.5,
				cursor: ptr::null_mut(),
				moving: false,
    
			    first: first, 
			    second: second, 
			},()), ()),
			development::MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
		));
		b.set_layout_padding(layout::BoundarySize::AllTheSame(DEFAULT_PADDING).into());
		b.as_inner_mut().as_inner_mut().as_inner_mut().update_children_layout();
		
		b
	}
	fn set_splitter(&mut self, base: &mut development::MemberControlBase, pos: f32) {
		self.splitter = pos;
		self.base.invalidate(base);
	}
	fn splitter(&self) -> f32 {
		self.splitter
	}
	fn first(&self) -> &controls::Control { self.first.as_ref() }
	fn second(&self) -> &controls::Control { self.second.as_ref() }
	fn first_mut(&mut self) -> &mut controls::Control { self.first.as_mut() }
	fn second_mut(&mut self) -> &mut controls::Control { self.second.as_mut() }
}

impl development::MemberInner for WindowsSplitted {
	type Id = common::Hwnd;
	
	fn size(&self) -> (u16, u16) {
        let rect = unsafe { common::window_rect(self.base.hwnd) };
        (
            (rect.right - rect.left) as u16,
            (rect.bottom - rect.top) as u16,
        )
    }

    fn on_set_visibility(&mut self, base: &mut development::MemberBase) {
	    let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
        	use plygui_api::development::Drawable;
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

impl development::ControlInner for WindowsSplitted {
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
    fn on_added_to_container(&mut self, base: &mut development::MemberControlBase, parent: &controls::Container, px: i32, py: i32) {
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
        self.reload_cursor();
        self.update_children_layout();
        
        let mut x = lp + lm;
        let mut y = tp + tm;
        
        let self2: &mut Splitted = unsafe { mem::transmute(selfptr) };
            
        self.first.on_added_to_container(self2, x, y); 
        
        let (xx, yy) = self.first.size();
        match self.orientation {
            layout::Orientation::Horizontal => {
            	x += xx as i32;
            	x += DEFAULT_BOUND;
            },
            layout::Orientation::Vertical => {
            	y += yy as i32;
	            y += DEFAULT_BOUND;
            },
        }    
        
        self.second.on_added_to_container(self2, x, y);     
    }
    fn on_removed_from_container(&mut self, base: &mut development::MemberControlBase, _: &controls::Container) {
        let self2: &mut Splitted = unsafe { utils::base_to_impl_mut(&mut base.member) };
        
        self.first.on_removed_from_container(self2);    
        self.second.on_removed_from_container(self2);    
            
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
        self.cursor = ptr::null_mut();
    }
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut MemberControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_SPLITTED;

        fill_from_markup_base!(
            self,
            base,
            markup,
            registry,
            Splitted,
            [MEMBER_TYPE_SPLITTED]
        );
        fill_from_markup_children!(self, &mut base.member, markup, registry);
    }
}

impl development::HasLayoutInner for WindowsSplitted {
	fn on_layout_changed(&mut self, base: &mut development::MemberBase) {
		self.update_children_layout();
		
		let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
        	use plygui_api::development::{Drawable, ControlInner};
			let base = self.cast_base_mut(base);
			self.invalidate(base);
		}
	}
}

impl development::ContainerInner for WindowsSplitted {
	fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut controls::Control> {
		use plygui_api::development::SplittedInner;
		
		if self.first().as_member().id() == id_ {
			return Some(self.first_mut());
		}
		if self.second().as_member().id() == id_ {
			return Some(self.second_mut());
		}
		
		let self2: &mut WindowsSplitted = unsafe { mem::transmute(self as *mut WindowsSplitted) }; // bck is stupid
		if let Some(c) = self.first_mut().is_container_mut() {
            let ret = c.find_control_by_id_mut(id_);
            if ret.is_some() {
                return ret;
            }
        }
		if let Some(c) = self2.second_mut().is_container_mut() {
            let ret = c.find_control_by_id_mut(id_);
            if ret.is_some() {
                return ret;
            }
        }
		
        None
    }
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&controls::Control> {
    	use plygui_api::development::SplittedInner;
    	
        if self.first().as_member().id() == id_ {
			return Some(self.first());
		}
		if self.second().as_member().id() == id_ {
			return Some(self.second());
		}
		
		if let Some(c) = self.first().is_container() {
            let ret = c.find_control_by_id(id_);
            if ret.is_some() {
                return ret;
            }
        }
		if let Some(c) = self.second().is_container() {
            let ret = c.find_control_by_id(id_);
            if ret.is_some() {
                return ret;
            }
        }
		
        None
    }
    fn gravity(&self) -> (layout::Gravity, layout::Gravity) {
    	(self.gravity_horizontal, self.gravity_vertical)
    }
    fn set_gravity(&mut self, _: &mut development::MemberBase, _: layout::Gravity, _: layout::Gravity) {}
}

impl development::MultiContainerInner for WindowsSplitted {
	fn len(&self) -> usize {
		2
	}
    fn set_child_to(&mut self, _: &mut development::MemberBase, index: usize, mut child: Box<controls::Control>) -> Option<Box<controls::Control>> {
    	use plygui_api::controls::HasLayout;
    	
    	match index {
	    	0 => {
	    		let hwnd = self.base.hwnd;
			    if !hwnd.is_null() {
			    	let self2 = common::member_from_hwnd::<Splitted>(hwnd);
				    
				    let (lp, tp, _, _) = self2.as_has_layout().layout_padding().into();
			        let (lm, tm, _, _) = self2.as_has_layout().layout_margin().into();
			        
		    		self.first.on_removed_from_container(self2);
				    child.on_added_to_container(self2, lp + lm, tp + tm);
			    }
	    		mem::swap(&mut self.first, &mut child);
	    	},
	    	1 => {
	    		let hwnd = self.base.hwnd;
			    if !hwnd.is_null() {
				    let self2 = common::member_from_hwnd::<Splitted>(hwnd);
				    
				    let (lp, tp, _, _) = self2.as_has_layout().layout_padding().into();
			        let (lm, tm, _, _) = self2.as_has_layout().layout_margin().into();
			        let mut x = lp + lm;
				    let mut y = tp + tm;
        
			        let (xx, yy) = self.first.size();
					match self.orientation {
					    layout::Orientation::Horizontal => { 
					    	x += xx as i32;
					    	x += DEFAULT_BOUND;
					    },
					    layout::Orientation::Vertical => {
					    	y += yy as i32;
						    y += DEFAULT_BOUND;
					    },
					} 
			        
		    		self.second.on_removed_from_container(self2);
		    		child.on_added_to_container(self2, x, y);
			    }
	    		mem::swap(&mut self.second, &mut child);
	    	},
	    	_ => return None,
    	}
    	
    	Some(child)
    }
    fn remove_child_from(&mut self, _: &mut development::MemberBase, _: usize) -> Option<Box<controls::Control>> {
    	None
    }
    fn child_at(&self, index: usize) -> Option<&controls::Control> {
    	use plygui_api::development::SplittedInner;
    	
    	match index {
    		0 => Some(self.first()),
    		1 => Some(self.second()),
    		_ => None
    	}
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut controls::Control> {
    	use plygui_api::development::SplittedInner;
    	
    	match index {
    		0 => Some(self.first_mut()),
    		1 => Some(self.second_mut()),
    		_ => None
    	}
    }
}

impl development::HasOrientationInner for WindowsSplitted {
	fn layout_orientation(&self) -> layout::Orientation {
		self.orientation
	}
    fn set_layout_orientation(&mut self, base: &mut development::MemberBase, orientation: layout::Orientation) {
    	if orientation != self.orientation {
    		use plygui_api::development::{Drawable, ControlInner};
    		
    		self.orientation = orientation;
    		self.reload_cursor();
    		
	    	let base = self.cast_base_mut(base);
			self.invalidate(base);
    	}
    }
}

impl development::Drawable for WindowsSplitted {
	fn draw(&mut self, base: &mut development::MemberControlBase, coords: Option<(i32, i32)>) {
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
            for ref mut child in [self.first.as_mut(), self.second.as_mut()].iter_mut() {
                child.draw(Some((x, y)));
                let (xx, yy) = child.size();
                match self.orientation {
                    layout::Orientation::Horizontal => {
                    	x += xx as i32;
	                    x += DEFAULT_BOUND;
                    },
                    layout::Orientation::Vertical => {
                    	y += yy as i32;
	                    y += DEFAULT_BOUND;
                    },
                }
            }
        }
    }
    fn measure(&mut self, base: &mut development::MemberControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;
    	
    	let orientation = self.orientation;
    	let old_size = self.base.measured_size;
    	let (lp,tp,rp,bp) = base.control.layout.padding.into();
    	let (lm,tm,rm,bm) = base.control.layout.margin.into();
    	let hp = lm + rm + lp + rp + if orientation == layout::Orientation::Horizontal { DEFAULT_BOUND } else { 0 };
    	let vp = tm + bm + tp + bp + if orientation == layout::Orientation::Vertical { DEFAULT_BOUND } else { 0 };
    	self.base.measured_size = match base.member.visibility {
        	types::Visibility::Gone => (0,0),
        	_ => {
        		let mut measured = false;
        		let w = match base.control.layout.width {
        			layout::Size::Exact(w) => w,
        			layout::Size::MatchParent => parent_width,
        			layout::Size::WrapContent => {
	        			let mut w = 0;
		                for child in [self.first.as_mut(), self.second.as_mut()].iter_mut() {
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
		                for child in [self.first.as_mut(), self.second.as_mut()].iter_mut() {
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
    fn invalidate(&mut self, base: &mut development::MemberControlBase) {
    	self.base.invalidate(base)
    }
}

/*#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Splitted::with_content(layout::Orientation::Vertical).into_control()
}*/

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32Splitted")
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
    use plygui_api::controls::{HasOrientation, Member};
    
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
	    	let mut width = lparam as u16;
            let mut height = (lparam >> 16) as u16;
            let mut ll: &mut Splitted = mem::transmute(ww);
            let o = ll.layout_orientation();
			let (lp, tp, rp, bp) = ll.is_control().unwrap().layout_padding().into();
	        let (lm, tm, rm, bm) = ll.is_control().unwrap().layout_margin().into();
	        let hp = lm + rm + lp + rp + if o == layout::Orientation::Horizontal { DEFAULT_BOUND } else { 0 };
	    	let vp = tm + bm + tp + bp + if o == layout::Orientation::Vertical { DEFAULT_BOUND } else { 0 };
    	
            let mut x = 0;
            let mut y = 0;
            {
            	use plygui_api::development::OuterDrawable;
            	
            	ll.set_skip_draw(true);
            	ll.as_inner_mut().as_inner_mut().as_inner_mut().update_children_layout();
            	ll.set_skip_draw(false);
            	
            	let ll = ll.as_inner_mut().as_inner_mut().as_inner_mut();
    			for child in [ll.first.as_mut(), ll.second.as_mut()].iter_mut() {
	            	let (cw, ch, _) = child.measure(cmp::max(0, width as i32 - hp) as u16, cmp::max(0, height as i32 - vp) as u16);
	                child.draw(Some((x + lp + lm, y + tp + tm))); 
	                match o {
	                    layout::Orientation::Horizontal if width >= cw => {
	                        x += cw as i32;
	                        x += DEFAULT_BOUND;
	                        width -= cw;
	                        width -= cmp::min(width as i32, DEFAULT_BOUND) as u16;
	                    }
	                    layout::Orientation::Vertical if height >= ch => {
	                        y += ch as i32;
	                        y += DEFAULT_BOUND;
	                        height -= ch;
	                        height -= cmp::min(height as i32, DEFAULT_BOUND) as u16;
	                    }
	                    _ => {}
	                }
	            }
            }

            if let Some(ref mut cb) = ll.base_mut().handler_resize {
                let mut ll2: &mut Splitted = mem::transmute(ww);
                (cb.as_mut())(ll2, width, height);
            }
            return 0;
        }
        winuser::WM_MOUSEMOVE => {
	        let mut x = lparam as u16;
            let mut y = (lparam >> 16) as u16;
            let mut updated = false;
            
            let mut ll: &mut Splitted = mem::transmute(ww);
            let (width, height) = ll.size();
            
            match ll.layout_orientation() {
            	layout::Orientation::Horizontal => {
	            	if x > DEFAULT_BOUND as u16 && x < (width - DEFAULT_BOUND as u16) {
	            		winuser::SetCursor(ll.as_inner_mut().as_inner_mut().as_inner_mut().cursor);
	            		
	            		if wparam == winuser::MK_LBUTTON && true {
	            			ll.as_inner_mut().as_inner_mut().as_inner_mut().splitter = x as f32 / width as f32;
	            			updated = true;
	            		}
	            	}
            	},
            	layout::Orientation::Vertical => {
	            	if y > DEFAULT_BOUND as u16 && y < (height - DEFAULT_BOUND as u16) {
	            		winuser::SetCursor(ll.as_inner_mut().as_inner_mut().as_inner_mut().cursor);
	            		
	            		if wparam == winuser::MK_LBUTTON && ll.as_inner_mut().as_inner_mut().as_inner_mut().moving {
	            			ll.as_inner_mut().as_inner_mut().as_inner_mut().splitter = y as f32 / height as f32;
	            			updated = true;
	            		}
	            	}
            	},
            }
            
            if updated {
            	let packed = ((height as i32) << 16) + width as i32;
    			winuser::SendMessageW(hwnd, winuser::WM_SIZE, 0, packed as isize);
            }
            return 0;
        },
        winuser::WM_LBUTTONDOWN => {
			let mut ll: &mut Splitted = mem::transmute(ww);
            
            winuser::SetCursor(ll.as_inner_mut().as_inner_mut().as_inner_mut().cursor);
			ll.as_inner_mut().as_inner_mut().as_inner_mut().moving = true;
			winuser::SetCapture(hwnd);
			return 0;
        }
		winuser::WM_LBUTTONUP => {
			let mut ll: &mut Splitted = mem::transmute(ww);
            
            winuser::ReleaseCapture();
			ll.as_inner_mut().as_inner_mut().as_inner_mut().moving = false;
			return 0;
		}
        _ => {}
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl_all_defaults!(Splitted);
