use super::*;

use plygui_api::{layout, types, development, callbacks, ids};
use plygui_api::traits::{UiControl, UiSingleContainer, UiHasLabel, UiHasLayout, UiFrame, UiMember, UiContainer};
use plygui_api::members::MEMBER_ID_FRAME;

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

#[repr(C)]
pub struct Frame {
    base: common::WindowsControlBase,
    hwnd_gbox: windef::HWND,
    label: String,
    label_padding: i32,
    child: Option<Box<UiControl>>,
}

impl Frame {
    pub fn new(label: &str) -> Box<Frame> {
        let mut b = Box::new(Frame {
                                 base: common::WindowsControlBase::with_params(invalidate_impl,
                                                                               development::UiMemberFunctions {
                                                                                   fn_member_id: member_id,
                                                                                   fn_is_control: is_control,
                                                                                   fn_is_control_mut: is_control_mut,
                                                                                   fn_size: size,
                                                                               }),
                                 child: None,
                                 hwnd_gbox: 0 as windef::HWND,
                                 label: label.to_owned(),
                                 label_padding: 0,
                             });
        b.set_layout_padding(layout::BoundarySize::AllTheSame(DEFAULT_PADDING).into());
        b
    }
}

impl UiHasLabel for Frame {
    fn label(&self) -> Cow<str> {
        Cow::Borrowed(self.label.as_ref())
    }
    fn set_label(&mut self, label: &str) {
        self.label = label.into();
        if self.base.hwnd != 0 as windef::HWND {
            let control_name = OsStr::new(&self.label)
                .encode_wide()
                .chain(Some(0).into_iter())
                .collect::<Vec<_>>();
            unsafe {
                winuser::SetWindowTextW(self.base.hwnd, control_name.as_ptr());
            }
            self.base.invalidate();
        }
    }
}

impl UiSingleContainer for Frame {
    fn set_child(&mut self, child: Option<Box<UiControl>>) -> Option<Box<UiControl>> {
        let old = self.child.take();
        self.child = child;

        old
    }
    fn child(&self) -> Option<&UiControl> {
        self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut UiControl> {
        //self.child.as_mut().map(|c|c.as_mut()) // WTF ??
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
    fn as_container(&self) -> &UiContainer {
        self
    }
    fn as_container_mut(&mut self) -> &mut UiContainer {
        self
    }
}

impl UiContainer for Frame {
	fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut UiControl> {
		if id_ == self.base.control_base.member_base.id {
			return Some(self)
		}
        if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_by_id_mut(id_);
            }
        }
        None
    }
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&UiControl> {
        if id_ == self.base.control_base.member_base.id {
			return Some(self)
		}
        if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control_by_id(id_);
            }
        }
        None
    }
    fn is_single_mut(&mut self) -> Option<&mut UiSingleContainer> {
        Some(self)
    }
    fn is_single(&self) -> Option<&UiSingleContainer> {
        Some(self)
    }
    fn as_member(&self) -> &UiMember {
        self
    }
    fn as_member_mut(&mut self) -> &mut UiMember {
        self
    }
}

impl UiFrame for Frame {
    fn as_control(&self) -> &UiControl {
        self
    }
    fn as_control_mut(&mut self) -> &mut UiControl {
        self
    }
    fn as_has_label(&self) -> &UiHasLabel {
        self
    }
    fn as_has_label_mut(&mut self) -> &mut UiHasLabel {
        self
    }
    fn as_single_container(&self) -> &UiSingleContainer {
        self
    }
    fn as_single_container_mut(&mut self) -> &mut UiSingleContainer {
        self
    }
}

impl UiControl for Frame {
    fn on_added_to_container(&mut self, parent: &UiContainer, px: i32, py: i32) {
        use plygui_api::development::UiDrawable;

        let selfptr = self as *mut _ as *mut c_void;
        let (pw, ph) = parent.draw_area_size();
        let (lm, tm, rm, bm) = self.base.control_base.layout.margin.into();
        let (hwnd, hwnd_gbox, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (width, height, _) = self.measure(pw, ph);
            let (hwnd, id) = common::create_control_hwnd(px as i32 + lm,
                                        py as i32 + tm,
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
                                        px as i32 + lm,
                                        py as i32 + tm,
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
        self.base.coords = Some((px as i32, py as i32));
        if let Some(ref mut child) = self.child {
        	let (lp, tp, _, _) = self.base.control_base.layout.padding.into();
	        let self2: &mut Frame = unsafe { &mut *(selfptr as *mut Frame) };
        	child.on_added_to_container(self2, lm + lp, tm + tp + self.label_padding);
        }
    }
    fn on_removed_from_container(&mut self, _: &UiContainer) {
        common::destroy_hwnd(self.hwnd_gbox, 0, None);
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.hwnd_gbox = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    fn is_container_mut(&mut self) -> Option<&mut UiContainer> {
        None
    }
    fn is_container(&self) -> Option<&UiContainer> {
        None
    }

    fn parent(&self) -> Option<&types::UiMemberBase> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&types::UiMemberBase> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        self.base.root_mut()
    }
    fn as_has_layout(&self) -> &UiHasLayout {
        self
    }
    fn as_has_layout_mut(&mut self) -> &mut UiHasLayout {
        self
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_FRAME;

        fill_from_markup_base!(self,
                               markup,
                               registry,
                               Frame,
                               [MEMBER_ID_FRAME, MEMBER_TYPE_FRAME]);
        fill_from_markup_label!(self, markup);
    }
}

impl UiHasLayout for Frame {
    fn layout_width(&self) -> layout::Size {
        self.base.control_base.layout.width
    }
    fn layout_height(&self) -> layout::Size {
        self.base.control_base.layout.height
    }
    fn layout_gravity(&self) -> layout::Gravity {
        self.base.control_base.layout.gravity
    }
    fn layout_alignment(&self) -> layout::Alignment {
        self.base.control_base.layout.alignment
    }
    fn layout_padding(&self) -> layout::BoundarySize {
        self.base.control_base.layout.padding
    }
    fn layout_margin(&self) -> layout::BoundarySize {
        self.base.control_base.layout.margin
    }

    fn set_layout_width(&mut self, width: layout::Size) {
        self.base.control_base.layout.width = width;
        self.base.invalidate();
    }
    fn set_layout_height(&mut self, height: layout::Size) {
        self.base.control_base.layout.height = height;
        self.base.invalidate();
    }
    fn set_layout_gravity(&mut self, gravity: layout::Gravity) {
        self.base.control_base.layout.gravity = gravity;
        self.base.invalidate();
    }
    fn set_layout_alignment(&mut self, alignment: layout::Alignment) {
        self.base.control_base.layout.alignment = alignment;
        self.base.invalidate();
    }
    fn set_layout_padding(&mut self, padding: layout::BoundarySizeArgs) {
        self.base.control_base.layout.padding = padding.into();
        self.base.invalidate();
    }
    fn set_layout_margin(&mut self, margin: layout::BoundarySizeArgs) {
        self.base.control_base.layout.margin = margin.into();
        self.base.invalidate();
    }
    fn as_member(&self) -> &UiMember {
        self
    }
    fn as_member_mut(&mut self) -> &mut UiMember {
        self
    }
}

impl UiMember for Frame {
    fn size(&self) -> (u16, u16) {
        let rect = unsafe { common::window_rect(self.base.hwnd) };
        ((rect.right - rect.left) as u16, (rect.bottom - rect.top) as u16)
    }

    fn on_resize(&mut self, handler: Option<callbacks::Resize>) {
        self.base.h_resize = handler;
    }

    fn set_visibility(&mut self, visibility: types::Visibility) {
        self.base.control_base.member_base.visibility = visibility;
        unsafe {
            winuser::ShowWindow(self.base.hwnd,
                                if self.base.control_base.member_base.visibility == types::Visibility::Invisible {
                                    winuser::SW_HIDE
                                } else {
                                    winuser::SW_SHOW
                                });
            self.base.invalidate();
        }
    }
    fn visibility(&self) -> types::Visibility {
        self.base.control_base.member_base.visibility
    }

    fn is_control(&self) -> Option<&UiControl> {
        Some(self)
    }
    fn is_control_mut(&mut self) -> Option<&mut UiControl> {
        Some(self)
    }
    fn as_base(&self) -> &types::UiMemberBase {
        self.base.control_base.member_base.as_ref()
    }
    fn as_base_mut(&mut self) -> &mut types::UiMemberBase {
        self.base.control_base.member_base.as_mut()
    }

    unsafe fn native_id(&self) -> usize {
        self.base.hwnd as usize
    }
}

impl development::UiDrawable for Frame {
    fn draw(&mut self, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.base.coords = coords;
        }
        let (lp, tp, _, _) = self.base.control_base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.control_base.layout.margin.into();
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
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;
    	
    	let old_size = self.base.measured_size;
    	let (lp,tp,rp,bp) = self.base.control_base.layout.padding.into();
    	let (lm,tm,rm,bm) = self.base.control_base.layout.margin.into();
    	let hp = lm + rm + lp + rp;
    	let vp = tm + bm + tp + bp;
    	self.base.measured_size = match self.visibility() {
        	types::Visibility::Gone => (0,0),
        	_ => {
        		let mut measured = false;
		        let w = match self.layout_width() {
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
        		let h = match self.layout_height() {
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
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<UiControl> {
    Frame::new("")
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new(MEMBER_ID_FRAME)
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
    	
	    	let mut width = lparam as u16;
            let mut height = (lparam >> 16) as u16;
            let mut frame: &mut Frame = mem::transmute(ww);
            
            if let Some(ref mut child) = frame.child {
                let (lp, tp, rp, bp) = frame.base.control_base.layout.padding.into();
		        let (lm, tm, rm, bm) = frame.base.control_base.layout.margin.into();
		        let hp = lm + rm + lp + rp;
		    	let vp = tm + bm + tp + bp;
		    	child.measure(max(0, width as i32 - hp) as u16, max(0, height as i32 - vp) as u16);
                child.draw(Some((lp + lm, tp + tm))); 
            }

            if let Some(ref mut cb) = frame.base.h_resize {
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

impl_invalidate!(Frame);
impl_is_control!(Frame);
impl_size!(Frame);
impl_member_id!(MEMBER_ID_FRAME);
