use super::*;

use plygui_api::{layout, types, development, ids, traits, utils};
use plygui_api::development::{Drawable, HasInner};

use winapi::shared::windef;
use winapi::shared::minwindef;
use winapi::um::winuser;
use winapi::um::wingdi;
use winapi::um::libloaderapi;
use winapi::ctypes::c_void;

use std::{ptr, mem, str};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::borrow::Cow;

const DEFAULT_PADDING: i32 = 6;

lazy_static! {
	pub static ref WINDOW_CLASS_GBOX: Vec<u16> = OsStr::new("Button")
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };    
}

pub type Frame = development::Member<development::Control<development::SingleContainer<WindowsFrame>>>;

#[repr(C)]
pub struct WindowsFrame {
    base: common::WindowsControlBase,
    hwnd_gbox: windef::HWND,
    label: String,
    label_padding: i32,
    gravity_horizontal: layout::Gravity,
    gravity_vertical: layout::Gravity,
    child: Option<Box<traits::UiControl>>,
}

impl development::FrameInner for WindowsFrame {
	fn with_label(label: &str) -> Box<traits::UiFrame> {
		use plygui_api::traits::UiHasLayout;
		
		let mut b = Box::new(development::Member::with_inner(development::Control::with_inner(development::SingleContainer::with_inner(
			WindowsFrame {
				base: common::WindowsControlBase::new(),
				child: None,
                hwnd_gbox: 0 as windef::HWND,
                gravity_horizontal: Default::default(),
			    gravity_vertical: Default::default(),
			    label: label.to_owned(),
                label_padding: 0,
			}, ()), ()),
        	development::MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
		));
		b.set_layout_padding(layout::BoundarySize::AllTheSame(DEFAULT_PADDING).into());
        b
	}
}

impl development::HasLayoutInner for WindowsFrame {
	fn on_layout_changed(&mut self, _: &mut layout::Attributes) {
		let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
			self.invalidate(utils::member_control_base_mut(common::member_from_hwnd::<LinearLayout>(hwnd)));
		}
	}
}

impl development::HasLabelInner for WindowsFrame {
	fn label<'a>(&'a self) -> ::std::borrow::Cow<'a, str> {
		Cow::Borrowed(self.label.as_ref())
	}
    fn set_label(&mut self, label: &str) {
    	self.label = label.into();
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
        	let control_name = OsStr::new(&self.label)
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect::<Vec<_>>();
            unsafe {
                winuser::SetWindowTextW(self.base.hwnd, control_name.as_ptr());
            }
            self.invalidate(utils::member_control_base_mut(common::member_from_hwnd::<Button>(hwnd)));
        }
    }
}

impl development::SingleContainerInner for WindowsFrame {
	fn set_child(&mut self, child: Option<Box<traits::UiControl>>) -> Option<Box<traits::UiControl>> {
		let old = self.child.take();
        self.child = child;

        old
	}
    fn child(&self) -> Option<&traits::UiControl> {
    	self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut traits::UiControl> {
    	//self.child.as_mut().map(|c|c.as_mut()) // WTF ??
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
}

impl development::ContainerInner for WindowsFrame {
	fn find_control_by_id_mut(&mut self, id: ids::Id) -> Option<&mut traits::UiControl> {
		if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_by_id_mut(id);
            }
        }
        None
	}
    fn find_control_by_id(&self, id: ids::Id) -> Option<&traits::UiControl> {
    	if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control_by_id(id);
            }
        }
        None
    }
    
    fn gravity(&self) -> (layout::Gravity, layout::Gravity) {
    	(self.gravity_horizontal, self.gravity_vertical)
    }
    fn set_gravity(&mut self, base: &mut development::MemberBase, w: layout::Gravity, h: layout::Gravity) {
    	if self.gravity_horizontal != w || self.gravity_vertical != h {
    		self.gravity_horizontal = w;
    		self.gravity_vertical = h;
    		self.invalidate(unsafe { mem::transmute(base) });
    	}
    }
}

impl development::ControlInner for WindowsFrame {
	fn on_added_to_container(&mut self, base: &mut development::MemberControlBase, parent: &traits::UiContainer, px: i32, py: i32) {
		let selfptr = base as *mut _ as *mut c_void;
        let (pw, ph) = parent.draw_area_size();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();
        let (hwnd, hwnd_gbox, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (width, height, _) = self.measure(base, pw, ph);
            let (hwnd, id) = common::create_control_hwnd(px + lm,
                                        py + tm,
                                        width as i32 - rm - lm,
                                        height as i32 - bm - tm,
                                        self.base.hwnd,
                                        winuser::WS_EX_CONTROLPARENT,
                                        WINDOW_CLASS.as_ptr(),
                                        "",
                                        0,
                                        selfptr,
                                        None);
	        let hwnd_gbox = winuser::CreateWindowExW(0,
                                        WINDOW_CLASS_GBOX.as_ptr(),
                                        OsStr::new(self.label.as_str())
									        .encode_wide()
									        .chain(Some(0).into_iter())
									        .collect::<Vec<_>>().as_ptr(),
                                        winuser::BS_GROUPBOX | winuser::WS_CHILD | winuser::WS_VISIBLE,
                                        px + lm,
                                        py + tm,
                                        width as i32 - rm - lm,
                                        height as i32 - bm - tm,
                                        self.base.hwnd,
                                        ptr::null_mut(),
                                        common::hinstance(),
                                        ptr::null_mut());
	        (hwnd, hwnd_gbox, id)
        };
        self.base.hwnd = hwnd;
        self.hwnd_gbox = hwnd_gbox;
        self.base.subclass_id = id;
        self.base.coords = Some((px, py));
        if let Some(ref mut child) = self.child {
        	let (lp, tp, _, _) = base.control.layout.padding.into();
	        let self2: &mut Frame = unsafe { &mut *(selfptr as *mut Frame) };
        	child.on_added_to_container(self2, lm + lp, tm + tp + self.label_padding);
        }
	}
    fn on_removed_from_container(&mut self, _: &mut development::MemberControlBase, _: &traits::UiContainer) {
    	common::destroy_hwnd(self.hwnd_gbox, 0, None);
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.hwnd_gbox = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }
    
    fn parent(&self) -> Option<&traits::UiMember> {
		self.base.parent().map(|p| p.as_member())
	}
    fn parent_mut(&mut self) -> Option<&mut traits::UiMember> {
    	self.base.parent_mut().map(|p| p.as_member_mut())
    }
    fn root(&self) -> Option<&traits::UiMember> {
    	self.base.root().map(|p| p.as_member())
    }
    fn root_mut(&mut self) -> Option<&mut traits::UiMember> {
    	self.base.root_mut().map(|p| p.as_member_mut())
    }
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut MemberControlBase, markup: &super::markup::Markup, registry: &mut super::markup::MarkupRegistry) {
    	use plygui_api::markup::MEMBER_TYPE_FRAME;

        fill_from_markup_base!(self,
                               markup,
                               registry,
                               Frame,
                               [MEMBER_ID_FRAME, MEMBER_TYPE_FRAME]);
        fill_from_markup_label!(self, markup);
    }
}

impl development::MemberInner for WindowsFrame {
	type Id = common::Hwnd;
	
	fn size(&self, _: &development::MemberBase) -> (u16, u16) {
        let rect = unsafe { common::window_rect(self.base.hwnd) };
        (
            (rect.right - rect.left) as u16,
            (rect.bottom - rect.top) as u16,
        )
    }

    fn on_set_visibility(&mut self, base: &mut development::MemberBase) {
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

impl development::Drawable for WindowsFrame {
	fn draw(&mut self, base: &mut development::MemberControlBase, coords: Option<(i32, i32)>) {
		if coords.is_some() {
            self.base.coords = coords;
        }
        let (lp, tp, _, _) = base.control.layout.padding.into();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();
        if let Some((x, y)) = self.base.coords {
            unsafe {
                winuser::SetWindowPos(self.base.hwnd,
                                      ptr::null_mut(),
                                      x + lm,
                                      y + tm,
                                      self.base.measured_size.0 as i32 - rm - lm,
                                      self.base.measured_size.1 as i32 - bm - tm,
                                      0);
                winuser::SetWindowPos(self.hwnd_gbox,
                                      ptr::null_mut(),
                                      x + lm,
                                      y + tm,
                                      self.base.measured_size.0 as i32 - rm - lm,
                                      self.base.measured_size.1 as i32 - bm - tm,
                                      0);
            }
            if let Some(ref mut child) = self.child {
                child.draw(Some((lp + lm, tp + tm + self.label_padding)));
                child.size();
            }
        }
	}
    fn measure(&mut self, base: &mut development::MemberControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	use std::cmp::max;
    	
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
	        			if let Some(ref mut child) =  self.child {
		                    let (cw, _, _) = child.measure(
		                    	max(0, parent_width as i32 - hp) as u16, 
		                    	max(0, parent_height as i32 - vp) as u16
		                    );
		                    w += cw as i32;
		                    measured = true;
		                }
	        			max(0, w as i32 + hp) as u16
        			}
        		};
        		let h = match base.control.layout.height {
        			layout::Size::Exact(h) => h,
        			layout::Size::MatchParent => parent_height,
        			layout::Size::WrapContent => {
	        			let mut h = 0;
		                if let Some(ref mut child) =  self.child {
		                    let ch = if measured {
		                    	child.size().1
		                    } else {
		                    	let (_, ch, _) = child.measure(
			                    	max(0, parent_width as i32 - hp) as u16, 
			                    	max(0, parent_height as i32 - vp) as u16
			                    );
		                    	ch
		                    };
		                    h += ch as i32;
		                    let mut label_size: windef::SIZE = unsafe { mem::zeroed() };
			        		let label = OsStr::new(self.label.as_str())
	                                .encode_wide()
	                                .chain(Some(0).into_iter())
	                                .collect::<Vec<_>>();
	                            unsafe { wingdi::GetTextExtentPointW(winuser::GetDC(self.base.hwnd),
	                                                        label.as_ptr(),
	                                                        self.label.len() as i32,
	                                                        &mut label_size); }
	                        self.label_padding = label_size.cy as i32;
	                        h += self.label_padding;
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
    fn invalidate(&mut self, _: &mut development::MemberControlBase) {
    	let parent_hwnd = self.base.parent_hwnd();	
		if let Some(parent_hwnd) = parent_hwnd {
			let mparent = common::member_base_from_hwnd(parent_hwnd);
			let (pw, ph) = mparent.as_member().size();
			let this = common::member_from_hwnd::<Button>(self.base.hwnd);
			let (_,_,changed) = self.measure(utils::member_control_base_mut(this), pw, ph);
			self.draw(utils::member_control_base_mut(this), None);		
					
			if let Some(cparent) = mparent.as_member_mut().is_control_mut() {
				if changed {
					cparent.invalidate();
				} 
			}
			if parent_hwnd != 0 as ::winapi::shared::windef::HWND {
	    		unsafe { ::winapi::um::winuser::InvalidateRect(parent_hwnd, ptr::null_mut(), ::winapi::shared::minwindef::TRUE); }
	    	}
	    }
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<traits::UiControl> {
    Frame::with_label("").into_control()
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32Frame")
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
            use std::cmp::max;
	    	use plygui_api::traits::UiMember;
	    	
	    	let mut width = lparam as u16;
            let mut height = (lparam >> 16) as u16;
            let mut frame: &mut Frame = mem::transmute(ww);
            let label_padding = frame.as_inner().as_inner().as_inner().label_padding;
            let (lp, tp, rp, bp) = frame.is_control().unwrap().layout_padding().into();
	        let (lm, tm, rm, bm) = frame.is_control().unwrap().layout_margin().into();
	        let hp = lm + rm + lp + rp;
	    	let vp = tm + bm + tp + bp;
		    	
            if let Some(ref mut child) = frame.as_inner_mut().as_inner_mut().as_inner_mut().child {
                child.measure(max(0, width as i32 - hp) as u16, max(0, height as i32 - vp) as u16);
                child.draw(Some((lp + lm, tp + tm + label_padding))); 
            }

            if let Some(ref mut cb) = frame.base_mut().handler_resize {
                let mut frame2: &mut Frame = mem::transmute(ww);
                (cb.as_mut())(frame2, width, height);
            }
        }
        winuser::WM_DESTROY => {
            winuser::PostQuitMessage(0);
            return 0;
        }

        _ => {}
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl_all_defaults!(Frame);
