use super::*;
use super::common::*;

const DEFAULT_BOUND: i32 = DEFAULT_PADDING;
const HALF_BOUND: i32 = DEFAULT_BOUND / 2;

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub type Splitted = Member<Control<MultiContainer<WindowsSplitted>>>;

#[repr(C)]
pub struct WindowsSplitted {
    base: common::WindowsControlBase<Splitted>,
    
    orientation: layout::Orientation,
    
    splitter: f32,
    moving: bool,
    cursor: windef::HCURSOR,
    
    first: Box<controls::Control>, 
    second: Box<controls::Control>, 
}

impl WindowsSplitted {
    fn children_sizes(&self) -> (u16, u16) {
        let (w, h) = self.size();
        let target = match self.orientation {
    	    layout::Orientation::Horizontal => w,
        	layout::Orientation::Vertical => h,
    	};
    	(
    	    utils::coord_to_size((target as f32 * self.splitter) as i32 - DEFAULT_PADDING - HALF_BOUND),
    	    utils::coord_to_size((target as f32 * (1.0 - self.splitter)) as i32 - DEFAULT_PADDING - HALF_BOUND),
    	)
    }
    fn draw_children(&mut self) {
    	let mut x = DEFAULT_PADDING;
        let mut y = DEFAULT_PADDING;
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
		if self.base.hwnd.is_null() { return }
		
		let orientation = self.orientation;
		let (first_size, second_size) = self.children_sizes();
		let (width, height) = self.size();
		for (size, child) in [(first_size, self.first.as_mut()), (second_size, self.second.as_mut())].iter_mut() {
            match orientation {
            	layout::Orientation::Horizontal => {
            	    child.measure(
                    	cmp::max(0, *size) as u16, 
                    	cmp::max(0, height as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16
                    );
                },
            	layout::Orientation::Vertical => {
            	    child.measure(
                    	cmp::max(0, width as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16, 
                    	cmp::max(0, *size) as u16
                    );
                },
            }
        }
	}
}

impl SplittedInner for WindowsSplitted {
	fn with_content(first: Box<controls::Control>, second: Box<controls::Control>, orientation: layout::Orientation) -> Box<Splitted> {
		let b = Box::new(Member::with_inner(Control::with_inner(MultiContainer::with_inner(
			WindowsSplitted {
				base: common::WindowsControlBase::new(),
			    orientation: orientation,
				
				splitter: 0.5,
				cursor: ptr::null_mut(),
				moving: false,
    
			    first: first, 
			    second: second, 
			},()), ()),
			MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
		));
		b
	}
	fn set_splitter(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, pos: f32) {
		self.splitter = pos;
		self.base.invalidate();
	}
	fn splitter(&self) -> f32 {
		self.splitter
	}
	fn first(&self) -> &controls::Control { self.first.as_ref() }
	fn second(&self) -> &controls::Control { self.second.as_ref() }
	fn first_mut(&mut self) -> &mut controls::Control { self.first.as_mut() }
	fn second_mut(&mut self) -> &mut controls::Control { self.second.as_mut() }
}

impl MemberInner for WindowsSplitted {
	type Id = common::Hwnd;
	
	fn size(&self) -> (u16, u16) {
        self.base.size()
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
			self.base.invalidate();
	    }
    }
    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl ControlInner for WindowsSplitted {
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
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &controls::Container, px: i32, py: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        let (width, height, _) = self.measure(member, control, pw, ph);
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            common::create_control_hwnd(
                px as i32,
                py as i32,
                width as i32,
                height as i32,
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
        //self.update_children_layout();
        
        let self2: &mut Splitted = unsafe { mem::transmute(selfptr) };
        let (first_size, second_size) = self.children_sizes();
            
        match self.orientation {
            layout::Orientation::Horizontal => {
                let h = utils::coord_to_size(height as i32 - DEFAULT_PADDING - DEFAULT_PADDING);
            	self.first.on_added_to_container(self2, DEFAULT_PADDING, DEFAULT_PADDING, first_size, h); 
                self.second.on_added_to_container(self2, DEFAULT_PADDING + DEFAULT_BOUND + first_size as i32, DEFAULT_PADDING, second_size, h); 
            },
            layout::Orientation::Vertical => {
                let w = utils::coord_to_size(width as i32 - DEFAULT_PADDING - DEFAULT_PADDING);
            	self.first.on_added_to_container(self2, DEFAULT_PADDING, DEFAULT_PADDING, w, first_size); 
                self.second.on_added_to_container(self2, DEFAULT_PADDING, DEFAULT_PADDING + DEFAULT_BOUND + first_size as i32, w, second_size);
            },
        }
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _control: &mut ControlBase, _: &controls::Container) {
        let self2: &mut Splitted = unsafe { utils::base_to_impl_mut(member) };
        
        self.first.on_removed_from_container(self2);    
        self.second.on_removed_from_container(self2);    
            
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
        self.cursor = ptr::null_mut();
    }
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
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

impl HasLayoutInner for WindowsSplitted {
	fn on_layout_changed(&mut self, _base: &mut MemberBase) {
		//self.update_children_layout();
		
		let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
        	self.base.invalidate();
		}
	}
	fn layout_margin(&self, _member: &MemberBase) -> layout::BoundarySize {
	    layout::BoundarySize::AllTheSame(DEFAULT_PADDING)
	}
}

impl ContainerInner for WindowsSplitted {
	fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut controls::Control> {
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
}

impl MultiContainerInner for WindowsSplitted {
	fn len(&self) -> usize {
		2
	}
    fn set_child_to(&mut self, _: &mut MemberBase, index: usize, mut child: Box<controls::Control>) -> Option<Box<controls::Control>> {
    	match index {
	    	0 => {
	    		let hwnd = self.base.hwnd;
			    if !hwnd.is_null() {
			    	let self2 = common::member_from_hwnd::<Splitted>(hwnd);		
			    	let sizes = self.first.size();		    
				    self.first.on_removed_from_container(self2);
				    child.on_added_to_container(self2, DEFAULT_PADDING, DEFAULT_PADDING, sizes.0, sizes.1);
			    }
	    		mem::swap(&mut self.first, &mut child);
	    	},
	    	1 => {
	    		let hwnd = self.base.hwnd;
			    if !hwnd.is_null() {
				    let self2 = common::member_from_hwnd::<Splitted>(hwnd);
				    
				    let mut x = DEFAULT_PADDING;
				    let mut y = DEFAULT_PADDING;
        
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
			        let sizes = self.second.size();		
		    		self.second.on_removed_from_container(self2);
		    		child.on_added_to_container(self2, x, y, sizes.0, sizes.1);
			    }
	    		mem::swap(&mut self.second, &mut child);
	    	},
	    	_ => return None,
    	}
    	
    	Some(child)
    }
    fn remove_child_from(&mut self, _: &mut MemberBase, _: usize) -> Option<Box<controls::Control>> {
    	None
    }
    fn child_at(&self, index: usize) -> Option<&controls::Control> {
    	match index {
    		0 => Some(self.first()),
    		1 => Some(self.second()),
    		_ => None
    	}
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut controls::Control> {
    	match index {
    		0 => Some(self.first_mut()),
    		1 => Some(self.second_mut()),
    		_ => None
    	}
    }
}

impl HasOrientationInner for WindowsSplitted {
	fn layout_orientation(&self) -> layout::Orientation {
		self.orientation
	}
    fn set_layout_orientation(&mut self, _base: &mut MemberBase, orientation: layout::Orientation) {
    	if orientation != self.orientation {
    		self.orientation = orientation;
    		self.reload_cursor();
	    	self.base.invalidate();
    	}
    }
}

impl Drawable for WindowsSplitted {
	fn draw(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
        self.base.draw(coords);
        self.draw_children();
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;
    	
    	let orientation = self.orientation;
    	let old_size = self.base.measured_size;
    	let hp = DEFAULT_PADDING + DEFAULT_PADDING + if orientation == layout::Orientation::Horizontal { DEFAULT_BOUND } else { 0 };
    	let vp = DEFAULT_PADDING + DEFAULT_PADDING + if orientation == layout::Orientation::Vertical { DEFAULT_BOUND } else { 0 };
    	let (first_size, second_size) = self.children_sizes();
    	self.base.measured_size = match member.visibility {
        	types::Visibility::Gone => (0,0),
        	_ => {
        		let mut measured = false;
        		let w = match control.layout.width {
        			layout::Size::Exact(w) => w,
        			layout::Size::MatchParent => parent_width,
        			layout::Size::WrapContent => {
	        			let mut w = 0;
		                for (size, child) in [(first_size, self.first.as_mut()), (second_size, self.second.as_mut())].iter_mut() {
		                    match orientation {
		                    	layout::Orientation::Horizontal => {
		                    	    let (cw, _, _) = child.measure(
        		                    	max(0, *size) as u16, 
        		                    	max(0, parent_height as i32 - vp) as u16
        		                    );
			                    	w += cw;
			                    },
		                    	layout::Orientation::Vertical => {
		                    	    let (cw, _, _) = child.measure(
        		                    	max(0, parent_width as i32 - hp) as u16, 
        		                    	max(0, *size) as u16
        		                    );
			                    	w = max(w, cw);
			                    },
		                    }
		                }
	        			measured = true;
	        			max(0, w as i32 + hp) as u16
        			}
        		};
        		let h = match control.layout.height {
        			layout::Size::Exact(h) => h,
        			layout::Size::MatchParent => parent_height,
        			layout::Size::WrapContent => {
	        			let mut h = 0;
		                for (size, child) in [(first_size, self.first.as_mut()), (second_size, self.second.as_mut())].iter_mut() {
		                    let ch = if measured {
		                    	child.size().1
		                    } else {
		                    	let (_, ch, _) = match orientation {
    		                    	layout::Orientation::Horizontal => {
    		                    	    child.measure(
            		                    	max(0, *size) as u16, 
            		                    	max(0, parent_height as i32 - vp) as u16
            		                    )
    			                    },
    		                    	layout::Orientation::Vertical => {
    		                    	    child.measure(
            		                    	max(0, parent_width as i32 - hp) as u16, 
            		                    	max(0, *size) as u16
            		                    )
    			                    },
    		                    };
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
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
    	self.base.invalidate()
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
        winuser::WM_SIZE | common::WM_UPDATE_INNER => {
	    	let mut width = lparam as u16;
            let mut height = (lparam >> 16) as u16;
            let mut ll: &mut Splitted = mem::transmute(ww);
            {
            	ll.set_skip_draw(true);
            	{
	            	let ll = ll.as_inner_mut().as_inner_mut().as_inner_mut();
	    			ll.update_children_layout();
	            	ll.draw_children();
            	}
    			ll.set_skip_draw(false);            	
            }

            if msg != common::WM_UPDATE_INNER {
            	ll.call_on_resize(width, height);
	        } else {
	        	winuser::InvalidateRect(hwnd, ptr::null_mut(), minwindef::TRUE);
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
    			winuser::SendMessageW(hwnd, common::WM_UPDATE_INNER, 0, packed as isize);
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
