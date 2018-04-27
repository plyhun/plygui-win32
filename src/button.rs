use super::*;

use plygui_api::development::{HasInner, Drawable};
use plygui_api::{traits, layout, types, development, callbacks, ids};

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

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new(CLASS_ID)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
}

pub type Button = development::Member<development::Control<WindowsButton>>;

#[repr(C)]
pub struct WindowsButton {
    base: common::WindowsControlBase,
    label: String,
    h_left_clicked: Option<callbacks::Click>,
}

impl development::HasLabelInner for WindowsButton {
    fn label<'a>(&'a self) -> Cow<'a, str> {
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
            self.invalidate();
        }
    }
}

impl development::ClickableInner for WindowsButton {
    fn on_click(&mut self, handle: Option<callbacks::Click>) {
        self.h_left_clicked = handle;
    }
}

impl development::ButtonInner for WindowsButton {
    fn with_label(label: &str) -> types::Dbox<traits::UiButton> {
        let b: Box<traits::UiButton> = Box::new(Button::with_inner(WindowsButton {
            base: common::WindowsControlBase::new(),
            h_left_clicked: None,
            label: label.to_owned(),
        }));
        Box::new(b)
    }
}

impl development::ControlInner for WindowsButton {
    fn on_added_to_container(&mut self, parent: &traits::UiContainer, x: i32, y: i32) {
        use plygui_api::development::OuterDrawable;

        let selfptr = self as *mut _ as *mut c_void;
        let (pw, ph) = parent.size();
        //let (lp,tp,rp,bp) = self.base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.layout.margin.into();
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (w, h, _) = self.measure(pw, ph);
            common::create_control_hwnd(
                x as i32 + lm,
                y as i32 + tm,
                w as i32 - rm,
                h as i32 - bm,
                parent.native_id() as windef::HWND,
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
    fn on_removed_from_container(&mut self, _: &traits::UiContainer) {
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, Some(handler));
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_BUTTON;

        fill_from_markup_base!(
            self,
            markup,
            registry,
            Button,
            [MEMBER_ID_BUTTON, MEMBER_TYPE_BUTTON]
        );
        fill_from_markup_label!(self, markup);
        //fill_from_markup_callbacks!(self, markup, registry, ["on_left_click" => FnMut(&mut UiButton)]);

        if let Some(on_left_click) = markup.attributes.get("on_click") {
            let callback: callbacks::Click = registry.pop_callback(on_left_click.as_attribute()).unwrap();
            self.on_click(Some(callback));
        }
    }
}

impl development::HasLayoutInner for WindowsButton {
    fn layout_width(&self) -> layout::Size {
        self.base.layout.width
    }
    fn layout_height(&self) -> layout::Size {
        self.base.layout.height
    }
    fn layout_gravity(&self) -> layout::Gravity {
        self.base.layout.gravity
    }
    fn layout_alignment(&self) -> layout::Alignment {
        self.base.layout.alignment
    }
    fn layout_padding(&self) -> layout::BoundarySize {
        self.base.layout.padding
    }
    fn layout_margin(&self) -> layout::BoundarySize {
        self.base.layout.margin
    }

    fn set_layout_width(&mut self, width: layout::Size) {
        self.base.layout.width = width;
        self.invalidate();
    }
    fn set_layout_height(&mut self, height: layout::Size) {
        self.base.layout.height = height;
        self.invalidate();
    }
    fn set_layout_gravity(&mut self, gravity: layout::Gravity) {
        self.base.layout.gravity = gravity;
        self.invalidate();
    }
    fn set_layout_alignment(&mut self, alignment: layout::Alignment) {
        self.base.layout.alignment = alignment;
        self.invalidate();
    }
    fn set_layout_padding(&mut self, padding: layout::BoundarySizeArgs) {
        self.base.layout.padding = padding.into();
        self.invalidate();
    }
    fn set_layout_margin(&mut self, margin: layout::BoundarySizeArgs) {
        self.base.layout.margin = margin.into();
        self.invalidate();
    }
}

impl development::MemberInner for WindowsButton {
    type Id = common::Hwnd;
	
	fn size(&self) -> (u16, u16) {
        let rect = unsafe { common::window_rect(self.base.hwnd) };
        (
            (rect.right - rect.left) as u16,
            (rect.bottom - rect.top) as u16,
        )
    }

    fn on_resize(&mut self, handler: Option<callbacks::Resize>) {
        self.base.h_resize = handler;
    }

    fn set_visibility(&mut self, visibility: types::Visibility) {
        self.base.visibility = visibility;
        unsafe {
            winuser::ShowWindow(
                self.base.hwnd,
                if self.base.visibility == types::Visibility::Invisible {
                    winuser::SW_HIDE
                } else {
                    winuser::SW_SHOW
                },
            );
            self.invalidate();
        }
    }
    fn visibility(&self) -> types::Visibility {
        self.base.visibility
    }
    
    fn id(&self) -> ids::Id { self.base.id }

    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl development::Drawable for WindowsButton {
    fn draw(&mut self, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.base.coords = coords;
        }
        //let (lp,tp,rp,bp) = self.base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.layout.margin.into();
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
        }
    }
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = self.base.measured_size;
        let (lp, tp, rp, bp) = self.base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.layout.margin.into();

        self.base.measured_size = match self.base.visibility {
            types::Visibility::Gone => (0, 0),
            _ => unsafe {
                let mut label_size: windef::SIZE = mem::zeroed();
                let w = match self.base.layout.width {
                    layout::Size::MatchParent => parent_width,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => {
                        if label_size.cx < 1 {
                            let label = OsStr::new(self.label.as_str())
                                .encode_wide()
                                .chain(Some(0).into_iter())
                                .collect::<Vec<_>>();
                            wingdi::GetTextExtentPointW(
                                winuser::GetDC(self.base.hwnd),
                                label.as_ptr(),
                                self.label.len() as i32,
                                &mut label_size,
                            );
                        }
                        label_size.cx as u16
                    } 
                };
                let h = match self.base.layout.height {
                    layout::Size::MatchParent => parent_height,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => {
                        if label_size.cy < 1 {
                            let label = OsStr::new(self.label.as_str())
                                .encode_wide()
                                .chain(Some(0).into_iter())
                                .collect::<Vec<_>>();
                            wingdi::GetTextExtentPointW(
                                winuser::GetDC(self.base.hwnd),
                                label.as_ptr(),
                                self.label.len() as i32,
                                &mut label_size,
                            );
                        }
                        label_size.cy as u16
                    } 
                };
                (
                    max(0, w as i32 + lm + rm + lp + rp) as u16,
                    max(0, h as i32 + tm + bm + tp + bp) as u16,
                )
            },
        };
        (
            self.base.measured_size.0,
            self.base.measured_size.1,
            self.base.measured_size != old_size,
        )
    }
    fn invalidate(&mut self) {
    	/*let parent_hwnd = self.base.parent_hwnd();	
		if let Some(parent_hwnd) = parent_hwnd {
			let mparent = common::cast_hwnd::<plygui_api::development::UiMemberCommon>(parent_hwnd);
			let (pw, ph) = mparent.size();
			let this: &mut $typ = mem::transmute(this);
			//let (_,_,changed) = 
			this.measure(pw, ph);
			this.draw(None);		
					
			if mparent.is_control().is_some() {
				let wparent = common::cast_hwnd::<common::WindowsControlBase>(parent_hwnd);
				//if changed {
					wparent.invalidate();
				//} 
			}
			if parent_hwnd != 0 as ::winapi::shared::windef::HWND {
	    		::winapi::um::winuser::InvalidateRect(parent_hwnd, ptr::null_mut(), ::winapi::shared::minwindef::TRUE);
	    	}
	    }*/
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> types::Dbox<traits::UiControl> {
    Button::with_label("").into_control()
}

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let button: &mut Button = mem::transmute(param);
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_LBUTTONDOWN => {
            if let Some(ref mut cb) = button.as_inner_mut().h_left_clicked {
                let mut button2: &mut Button = mem::transmute(param);
                (cb.as_mut())(button2);
            }
        }
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            if let Some(ref mut cb) = button.as_inner_mut().base.h_resize {
                let mut button2: &mut Button = mem::transmute(param);
                (cb.as_mut())(button2, width, height);
            }
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}
