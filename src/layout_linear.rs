use super::*;
use super::common::*;

use plygui_api::{layout, ids, types, development, callbacks};
use plygui_api::traits::{UiControl, UiHasLayout, UiMultiContainer, UiLinearLayout, UiMember, UiContainer, UiHasOrientation};
use plygui_api::members::MEMBER_ID_LAYOUT_LINEAR;

use winapi::shared::windef;
use winapi::shared::minwindef;
use winapi::um::winuser;
use winapi::um::libloaderapi;
use winapi::ctypes::c_void;

use std::{ptr, mem};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

#[repr(C)]
pub struct LinearLayout {
    base: WindowsControlBase,
    orientation: layout::Orientation,
    children: Vec<Box<UiControl>>,
}

impl LinearLayout {
    pub fn new(orientation: layout::Orientation) -> Box<LinearLayout> {
        Box::new(LinearLayout {
                     base: common::WindowsControlBase::with_params(invalidate_impl,
                                                                   development::UiMemberFunctions {
                                                                       fn_member_id: member_id,
                                                                       fn_is_control: is_control,
                                                                       fn_is_control_mut: is_control_mut,
                                                                       fn_size: size,
                                                                   }),
                     orientation: orientation,
                     children: Vec::new(),
                 })
    }
}

impl UiMember for LinearLayout {
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

    fn size(&self) -> (u16, u16) {
        let rect = unsafe { window_rect(self.base.hwnd) };
        ((rect.right - rect.left) as u16, (rect.bottom - rect.top) as u16)
    }

    fn on_resize(&mut self, handler: Option<callbacks::Resize>) {
        self.base.h_resize = handler;
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

impl UiHasOrientation for LinearLayout {
    fn layout_orientation(&self) -> layout::Orientation {
        self.orientation
    }
    fn set_layout_orientation(&mut self, orientation: layout::Orientation) {
        self.orientation = orientation;
        self.base.invalidate();
    }
}

impl UiHasLayout for LinearLayout {
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

impl UiControl for LinearLayout {
    fn is_container_mut(&mut self) -> Option<&mut UiContainer> {
        Some(self)
    }
    fn is_container(&self) -> Option<&UiContainer> {
        Some(self)
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
    fn on_added_to_container(&mut self, parent: &UiContainer, px: i32, py: i32) {
        use plygui_api::development::UiDrawable;

        let selfptr = self as *mut _ as *mut c_void;
        let (pw, ph) = parent.size();
        let (width, height, _) = self.measure(pw, ph);
        let (lp, tp, _, _) = self.base.control_base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.control_base.layout.margin.into();
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            common::create_control_hwnd(px as i32 + lm,
                                        py as i32 + tm,
                                        width as i32 - rm,
                                        height as i32 - bm,
                                        parent.native_id() as windef::HWND,
                                        winuser::WS_EX_CONTROLPARENT,
                                        WINDOW_CLASS.as_ptr(),
                                        "",
                                        0,
                                        selfptr,
                                        None)
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
        self.base.coords = Some((px as i32, py as i32));
        let mut x = lp;
        let mut y = tp;
        for ref mut child in self.children.as_mut_slice() {
            let self2: &mut LinearLayout = unsafe { mem::transmute(selfptr) };
            child.on_added_to_container(self2, x, y);
            let (xx, yy) = child.size();
            match self.orientation {
                layout::Orientation::Horizontal => x += xx as i32,
                layout::Orientation::Vertical => y += yy as i32,
            }
        }
    }
    fn on_removed_from_container(&mut self, _: &UiContainer) {
        let selfptr = self as *mut _ as *mut c_void;
        for ref mut child in self.children.as_mut_slice() {
            let self2: &mut LinearLayout = unsafe { mem::transmute(selfptr) };
            child.on_removed_from_container(self2);
        }
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }
    fn as_has_layout(&self) -> &UiHasLayout {
        self
    }
    fn as_has_layout_mut(&mut self) -> &mut UiHasLayout {
        self
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT;

        fill_from_markup_base!(self,
                               markup,
                               registry,
                               LinearLayout,
                               [MEMBER_ID_LAYOUT_LINEAR, MEMBER_TYPE_LINEAR_LAYOUT]);
        fill_from_markup_children!(self, markup, registry);
    }
}

impl UiContainer for LinearLayout {
    fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut UiControl> {
        if self.as_base().id() == id_ {
            return Some(self);
        }
        for child in self.children.as_mut_slice() {
            if child.as_base().id() == id_ {
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
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&UiControl> {
        if self.as_base().id() == id_ {
            return Some(self);
        }
        for child in self.children.as_slice() {
            if child.as_base().id() == id_ {
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
    fn is_multi_mut(&mut self) -> Option<&mut UiMultiContainer> {
        Some(self)
    }
    fn is_multi(&self) -> Option<&UiMultiContainer> {
        Some(self)
    }
    fn as_member(&self) -> &UiMember {
        self
    }
    fn as_member_mut(&mut self) -> &mut UiMember {
        self
    }
}

impl UiMultiContainer for LinearLayout {
    fn len(&self) -> usize {
        self.children.len()
    }
    fn set_child_to(&mut self, index: usize, child: Box<UiControl>) -> Option<Box<UiControl>> {
        //TODO yes this is ineffective, need a way to swap old item with new
        self.children.insert(index, child);
        if (index + 1) >= self.children.len() {
            return None;
        }
        Some(self.children.remove(index + 1))
    }
    fn remove_child_from(&mut self, index: usize) -> Option<Box<UiControl>> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }
    fn child_at(&self, index: usize) -> Option<&Box<UiControl>> {
        self.children.get(index)
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut Box<UiControl>> {
        self.children.get_mut(index)
    }
    fn as_container(&self) -> &UiContainer {
        self
    }
    fn as_container_mut(&mut self) -> &mut UiContainer {
        self
    }
}

impl UiLinearLayout for LinearLayout {
    fn as_control(&self) -> &UiControl {
        self
    }
    fn as_control_mut(&mut self) -> &mut UiControl {
        self
    }
    fn as_multi_container(&self) -> &UiMultiContainer {
        self
    }
    fn as_multi_container_mut(&mut self) -> &mut UiMultiContainer {
        self
    }
    fn as_has_orientation(&self) -> &UiHasOrientation {
        self
    }
    fn as_has_orientation_mut(&mut self) -> &mut UiHasOrientation {
        self
    }
}

impl development::UiDrawable for LinearLayout {
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
                                      self.base.measured_size.0 as i32 - rm,
                                      self.base.measured_size.1 as i32 - bm,
                                      0);
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
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let old_size = self.base.measured_size;
        let (lp, tp, rp, bp) = self.base.control_base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.control_base.layout.margin.into();
        self.base.measured_size = match self.visibility() {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut w = parent_width;
                let mut h = parent_height;

                if let layout::Size::Exact(ew) = self.layout_width() {
                    w = ew;
                }
                if let layout::Size::Exact(eh) = self.layout_height() {
                    w = eh;
                }
                let (mut ww, mut wm, mut hh, mut hm) = (0, 0, 0, 0);
                for ref mut child in self.children.as_mut_slice() {
                    let (cw, ch, _) = child.measure(w, h);
                    ww += cw;
                    hh += ch;
                    wm = max(wm, cw);
                    hm = max(hm, ch);
                }

                match self.orientation {
                    layout::Orientation::Vertical => {
                        if let layout::Size::WrapContent = self.layout_height() {
                            h = hh;
                        }
                        if let layout::Size::WrapContent = self.layout_width() {
                            w = wm;
                        }
                    }
                    layout::Orientation::Horizontal => {
                        if let layout::Size::WrapContent = self.layout_height() {
                            h = hm;
                        }
                        if let layout::Size::WrapContent = self.layout_width() {
                            w = ww;
                        }
                    }
                }
                (max(0, w as i32 + lm + rm + lp + rp) as u16, max(0, h as i32 + tm + bm + tp + bp) as u16)
            }
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
}

unsafe impl common::WindowsContainer for LinearLayout {
    unsafe fn hwnd(&self) -> windef::HWND {
        self.base.hwnd
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<UiControl> {
    LinearLayout::new(layout::Orientation::Vertical)
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new(MEMBER_ID_LAYOUT_LINEAR)
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
            let mut width = lparam as u16;
            let mut height = (lparam >> 16) as u16;
            let mut ll: &mut LinearLayout = mem::transmute(ww);
            let o = ll.orientation;

            let mut x = 0;
            let mut y = 0;
            for ref mut child in ll.children.as_mut_slice() {
                let (cw, ch, _) = child.measure(width, height);
                child.draw(Some((x, y))); //TODO padding
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

            if let Some(ref mut cb) = ll.base.h_resize {
                let mut ll2: &mut LinearLayout = mem::transmute(ww);
                (cb.as_mut())(ll2, width, height);
            }
        }
        winuser::WM_DESTROY => {
            winuser::PostQuitMessage(0);
            return 0;
        }
        /*winuser::WM_NOTIFY => {
        	let hdr: winuser::LPNMHDR = mem::transmute(lparam);
        	println!("notify for {:?}", hdr);
        },
        winuser::WM_COMMAND => {
        	let hdr: winuser::LPNMHDR = mem::transmute(lparam);
        	
        	println!("command for {:?}", hdr);
        }*/
        _ => {}
    }

    winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl_invalidate!(LinearLayout);
impl_is_control!(LinearLayout);
impl_size!(LinearLayout);
impl_member_id!(MEMBER_ID_LAYOUT_LINEAR);
