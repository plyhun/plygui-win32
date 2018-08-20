use super::*;

use plygui_api::{controls, layout, types, callbacks, utils};
use plygui_api::development::*;

use winapi::shared::windef;
use winapi::shared::minwindef;
use winapi::um::winuser;
use winapi::um::wingdi;
use winapi::um::commctrl;
use winapi::ctypes::c_void;

use std::{ptr, mem, str};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::borrow::Cow;
use std::cmp::max;

pub const CLASS_ID: &str = "Button";
const DEFAULT_PADDING: i32 = 6;

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
}

pub type Button = Member<Control<WindowsButton>>;

#[repr(C)]
pub struct WindowsButton {
    base: common::WindowsControlBase<Button>,
    label: String,
    h_left_clicked: Option<callbacks::Click>,
}

impl HasLabelInner for WindowsButton {
    fn label<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(self.label.as_ref())
    }
    fn set_label(&mut self, base: &mut MemberBase, label: &str) {
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
            let base = self.cast_base_mut(base);
			self.invalidate(base);
        }
    }
}

impl ClickableInner for WindowsButton {
    fn on_click(&mut self, handle: Option<callbacks::Click>) {
        self.h_left_clicked = handle;
    }
}

impl ButtonInner for WindowsButton {
    fn with_label(label: &str) -> Box<Button> {
    	use plygui_api::controls::HasLayout;
    	
        let mut b: Box<Button> = Box::new(Member::with_inner(Control::with_inner(
        		WindowsButton {
		            base: common::WindowsControlBase::new(),
		            h_left_clicked: None,
		            label: label.to_owned(),
		        }, ()),
        		MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        b.set_layout_padding(layout::BoundarySize::AllTheSame(DEFAULT_PADDING).into());
        b
    }
}

impl ControlInner for WindowsButton {
    fn on_added_to_container(&mut self, base: &mut MemberControlBase, parent: &controls::Container, x: i32, y: i32) {
    	let selfptr = base as *mut _ as *mut c_void;
        let (pw, ph) = parent.draw_area_size();
        //let (lp,tp,rp,bp) = self.base.layout.padding.into();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (w, h, _) = self.measure(base, pw, ph);
            common::create_control_hwnd(
                x as i32 + lm,
                y as i32 + tm,
                w as i32 - rm - lm,
                h as i32 - bm - tm,
                self.base.hwnd,
                0,
                WINDOW_CLASS.as_ptr(),
                self.label.as_str(),
                winuser::BS_PUSHBUTTON | winuser::WS_TABSTOP,
                selfptr,
                Some(handler),
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
    }
    fn on_removed_from_container(&mut self, _: &mut MemberControlBase, _: &controls::Container) {
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, Some(handler));
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }
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
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut MemberControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_BUTTON;
		fill_from_markup_base!(
            self,
            base,
            markup,
            registry,
            Button,
            [MEMBER_TYPE_BUTTON]
        );
        fill_from_markup_label!(self, &mut base.member, markup);
        fill_from_markup_callbacks!(self, markup, registry, [on_click => plygui_api::callbacks::Click]);
    }
}

impl HasLayoutInner for WindowsButton {
	fn on_layout_changed(&mut self, base: &mut MemberBase) {
		let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
        	let base = self.cast_base_mut(base);
			self.invalidate(base);
		}
	}
}

impl MemberInner for WindowsButton {
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

impl Drawable for WindowsButton {
    fn draw(&mut self, base: &mut MemberControlBase, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.base.coords = coords;
        }
        //let (lp,tp,rp,bp) = base.control.layout.padding.into();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();
        if let Some((x, y)) = self.base.coords {
            unsafe {
                winuser::SetWindowPos(
                    self.base.hwnd,
                    ptr::null_mut(),
                    x + lm,
                    y + tm,
                    self.base.measured_size.0 as i32 - rm - lm,
                    self.base.measured_size.1 as i32 - bm - tm,
                    0,
                );
            }
        }
    }
    fn measure(&mut self, base: &mut MemberControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = self.base.measured_size;
        
        let (lp, tp, rp, bp) = base.control.layout.padding.into();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();

        self.base.measured_size = match base.member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut label_size: windef::SIZE = unsafe { mem::zeroed() };
                let w = match base.control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        if label_size.cx < 1 {
                            let label = OsStr::new(self.label.as_str())
                                .encode_wide()
                                .chain(Some(0).into_iter())
                                .collect::<Vec<_>>();
                            unsafe { wingdi::GetTextExtentPointW(
                                winuser::GetDC(self.base.hwnd),
                                label.as_ptr(),
                                self.label.len() as i32,
                                &mut label_size,
                            ); }
                        }
                        label_size.cx as i32 + lm + rm + lp + rp
                    } 
                };
                let h = match base.control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.cy < 1 {
                            let label = OsStr::new(self.label.as_str())
                                .encode_wide()
                                .chain(Some(0).into_iter())
                                .collect::<Vec<_>>();
                            unsafe { 
                            	wingdi::GetTextExtentPointW(
	                                winuser::GetDC(self.base.hwnd),
	                                label.as_ptr(),
	                                self.label.len() as i32,
	                                &mut label_size,
	                            );
                            }
                        }
                        label_size.cy as i32 + tm + bm + tp + bp
                    } 
                };
                (max(0, w) as u16, max(0, h) as u16)
            },
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn invalidate(&mut self, base: &mut MemberControlBase) {
    	self.base.invalidate(base)
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Button::with_label("").into_control()
}

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_LBUTTONDOWN => {
            let button: &mut Button = mem::transmute(param);
		    if let Some(ref mut cb) = button.as_inner_mut().as_inner_mut().h_left_clicked {
                let mut button2: &mut Button = mem::transmute(param);
                (cb.as_mut())(button2);
            }
		    return 0;
        }
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let button: &mut Button = mem::transmute(param);
		    if let Some(ref mut cb) = button.base_mut().handler_resize {
                let mut button2: &mut Button = mem::transmute(param);
                (cb.as_mut())(button2, width, height);
            }
		    return 0;
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}

impl_all_defaults!(Button);
