use super::*;

use plygui_api::{layout, types, development, callbacks};
use plygui_api::traits::{UiControl, UiClickable, UiHasLabel, UiHasLayout, UiButton, UiMember, UiContainer};
use plygui_api::members::MEMBER_ID_BUTTON;

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

#[repr(C)]
pub struct Button {
    base: common::WindowsControlBase,
    label: String,
    h_left_clicked: Option<callbacks::Click>,
}

impl Button {
    pub fn new(label: &str) -> Box<Button> {
        let mut b = Box::new(Button {
                                 base: common::WindowsControlBase::with_params(invalidate_impl,
                                                                               development::UiMemberFunctions {
                                                                                   fn_member_id: member_id,
                                                                                   fn_is_control: is_control,
                                                                                   fn_is_control_mut: is_control_mut,
                                                                                   fn_size: size,
                                                                               }),
                                 h_left_clicked: None,
                                 label: label.to_owned(),
                             });
        b.set_layout_padding(layout::BoundarySize::AllTheSame(DEFAULT_PADDING).into());
        b
    }
}

impl UiHasLabel for Button {
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
            self.base.invalidate();
        }
    }
}

impl UiClickable for Button {
    fn on_click(&mut self, handle: Option<callbacks::Click>) {
        self.h_left_clicked = handle;
    }
}

impl UiButton for Button {
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
    fn as_clickable(&self) -> &UiClickable {
        self
    }
    fn as_clickable_mut(&mut self) -> &mut UiClickable {
        self
    }
}

impl UiControl for Button {
    fn on_added_to_container(&mut self, parent: &UiContainer, x: i32, y: i32) {
        use plygui_api::development::UiDrawable;

        let selfptr = self as *mut _ as *mut c_void;
        let (pw, ph) = parent.size();
        //let (lp,tp,rp,bp) = self.base.control_base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.control_base.layout.margin.into();
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (w, h, _) = self.measure(pw, ph);
            common::create_control_hwnd(x as i32 + lm,
                                        y as i32 + tm,
                                        w as i32 - rm,
                                        h as i32 - bm,
                                        parent.native_id() as windef::HWND,
                                        0,
                                        WINDOW_CLASS.as_ptr(),
                                        self.label.as_str(),
                                        winuser::BS_PUSHBUTTON | winuser::WS_TABSTOP,
                                        selfptr,
                                        Some(handler))
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
    }
    fn on_removed_from_container(&mut self, _: &UiContainer) {
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, Some(handler));
        self.base.hwnd = 0 as windef::HWND;
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
        use plygui_api::markup::MEMBER_TYPE_BUTTON;

        fill_from_markup_base!(self,
                               markup,
                               registry,
                               Button,
                               [MEMBER_ID_BUTTON, MEMBER_TYPE_BUTTON]);
        fill_from_markup_label!(self, markup);
        //fill_from_markup_callbacks!(self, markup, registry, ["on_left_click" => FnMut(&mut UiButton)]);

        if let Some(on_left_click) = markup.attributes.get("on_click") {
            let callback: callbacks::Click = registry
                .pop_callback(on_left_click.as_attribute())
                .unwrap();
            self.on_click(Some(callback));
        }
    }
}

impl UiHasLayout for Button {
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

impl UiMember for Button {
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

impl development::UiDrawable for Button {
    fn draw(&mut self, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.base.coords = coords;
        }
        //let (lp,tp,rp,bp) = self.base.control_base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.control_base.layout.margin.into();
        if let Some((x, y)) = self.base.coords {
            unsafe {
                winuser::SetWindowPos(self.base.hwnd,
                                      ptr::null_mut(),
                                      x + lm,
                                      y + tm,
                                      self.base.measured_size.0 as i32 - rm,
                                      self.base.measured_size.1 as i32 - bm,
                                      0);
            }
        }
    }
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = self.base.measured_size;
        let (lp, tp, rp, bp) = self.base.control_base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.control_base.layout.margin.into();

        self.base.measured_size = match self.visibility() {
            types::Visibility::Gone => (0, 0),
            _ => unsafe {
                let mut label_size: windef::SIZE = mem::zeroed();
                let w = match self.layout_width() {
                    layout::Size::MatchParent => parent_width,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => {
                        if label_size.cx < 1 {
                            let label = OsStr::new(self.label.as_str())
                                .encode_wide()
                                .chain(Some(0).into_iter())
                                .collect::<Vec<_>>();
                            wingdi::GetTextExtentPointW(winuser::GetDC(self.base.hwnd),
                                                        label.as_ptr(),
                                                        self.label.len() as i32,
                                                        &mut label_size);
                        }
                        label_size.cx as u16
                    } 
                };
                let h = match self.layout_height() {
                    layout::Size::MatchParent => parent_height,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => {
                        if label_size.cy < 1 {
                            let label = OsStr::new(self.label.as_str())
                                .encode_wide()
                                .chain(Some(0).into_iter())
                                .collect::<Vec<_>>();
                            wingdi::GetTextExtentPointW(winuser::GetDC(self.base.hwnd),
                                                        label.as_ptr(),
                                                        self.label.len() as i32,
                                                        &mut label_size);
                        }
                        label_size.cy as u16
                    } 
                };
                (max(0, w as i32 + lm + rm + lp + rp) as u16, max(0, h as i32 + tm + bm + tp + bp) as u16)
            },
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<UiControl> {
    Button::new("")
}

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let button: &mut Button = mem::transmute(param);
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_LBUTTONDOWN => {
            if let Some(ref mut cb) = button.h_left_clicked {
                let mut button2: &mut Button = mem::transmute(param);
                (cb.as_mut())(button2);
            }
        }
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            if let Some(ref mut cb) = button.base.h_resize {
                let mut button2: &mut Button = mem::transmute(param);
                (cb.as_mut())(button2, width, height);
            }
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}

impl_invalidate!(Button);
impl_is_control!(Button);
impl_size!(Button);
impl_member_id!(MEMBER_ID_BUTTON);
