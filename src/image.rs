use crate::common::{self, *};

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new("STATIC").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

const DEFAULT_CONTENT_PADDING: i32 = 0;

pub type Image = AMember<AControl<AImage<WindowsImage>>>;

#[repr(C)]
pub struct WindowsImage {
    base: WindowsControlBase<Image>,

    bmp: windef::HBITMAP,
    scale: types::ImageScalePolicy,
}

impl WindowsImage {
    fn install_image(&mut self, content: image::DynamicImage) {
        unsafe {
            common::image_to_native(&content, &mut self.bmp);
        }
    }
    fn remove_image(&mut self) {
        unsafe {
            wingdi::DeleteObject(self.bmp as *mut c_void);
        }
        self.bmp = ptr::null_mut();
    }
    fn scaled_image_size(&self, pw: u16, ph: u16) -> (i32, i32) {
        let hoffs = DEFAULT_CONTENT_PADDING;
        let voffs = DEFAULT_CONTENT_PADDING;
        let hdiff = hoffs + DEFAULT_CONTENT_PADDING;
        let vdiff = voffs + DEFAULT_CONTENT_PADDING;
        let inner_h = pw as i32 - hdiff;
        let inner_v = ph as i32 - vdiff;

        let mut bm: wingdi::BITMAP = unsafe { mem::zeroed() };

        unsafe {
            wingdi::GetObjectW(self.bmp as *mut c_void, mem::size_of::<wingdi::BITMAP>() as i32, &mut bm as *mut _ as *mut c_void);
        }

        match self.scale {
            types::ImageScalePolicy::FitCenter => {
                let (wrate, hrate) = (inner_h as f32 / bm.bmWidth as f32, inner_v as f32 / bm.bmHeight as f32);
                let less_rate = fmin(wrate, hrate);

                ((bm.bmWidth as f32 * less_rate) as i32, (bm.bmHeight as f32 * less_rate) as i32)
            }
            types::ImageScalePolicy::CropCenter => (cmp::min(pw as i32, bm.bmWidth) - hdiff, cmp::min(ph as i32, bm.bmHeight) - vdiff),
        }
    }
}

impl Drop for WindowsImage {
    fn drop(&mut self) {
        self.remove_image();
    }
}
impl HasImageInner for WindowsImage {
    fn image(&self, _: &MemberBase) -> Cow<image::DynamicImage> {
        todo!()
    }
    fn set_image(&mut self, _: &mut MemberBase, arg0: Cow<image::DynamicImage>) {
        self.install_image(arg0.into_owned())
    }
}
impl<O: controls::Image> NewImageInner<O> for WindowsImage {
    fn with_uninit(_: &mut mem::MaybeUninit<O>) -> Self {
        WindowsImage {
            base: WindowsControlBase::with_handler(Some(handler::<O>)),
            bmp: ptr::null_mut(),
            scale: types::ImageScalePolicy::FitCenter,
        }
    }
}
impl ImageInner for WindowsImage {
    fn with_content(content: image::DynamicImage) -> Box<dyn controls::Image> {
        let mut b: Box<mem::MaybeUninit<Image>> = Box::new_uninit();
        let mut ab = AMember::with_inner(
            AControl::with_inner(
                AImage::with_inner(
                    <Self as NewImageInner<Image>>::with_uninit(b.as_mut())
                )
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        );
        ab.inner_mut().inner_mut().inner_mut().install_image(content);
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
    fn set_scale(&mut self, _member: &mut MemberBase, policy: types::ImageScalePolicy) {
        if self.scale != policy {
            self.scale = policy;
            self.base.invalidate();
        }
    }
    fn scale(&self) -> types::ImageScalePolicy {
        self.scale
    }
}

impl ControlInner for WindowsImage {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        self.base.hwnd = unsafe { parent.native_id() as windef::HWND }; // required for measure, as we don't have own hwnd yet
        let (w, h, _) = self.measure(member, control, pw, ph);
        self.base.create_control_hwnd(
            x as i32,
            y as i32,
            w as i32,
            h as i32,
            self.base.hwnd,
            0,
            WINDOW_CLASS.as_ptr(),
            "",
            winuser::SS_BITMAP | winuser::SS_CENTERIMAGE | winuser::WS_TABSTOP,
            selfptr
        );
    }
    fn on_removed_from_container(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        destroy_hwnd(self.base.hwnd, self.base.subclass_id, self.base.proc_handler.as_handler());
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

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

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_IMAGE;
        fill_from_markup_base!(self, member, markup, registry, Image, [MEMBER_TYPE_IMAGE]);
        //TODO image source
    }
}

impl HasLayoutInner for WindowsImage {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl HasNativeIdInner for WindowsImage {
    type Id = common::Hwnd;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl HasSizeInner for WindowsImage {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for WindowsImage {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for WindowsImage {}

impl Drawable for WindowsImage {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, pw: u16, ph: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => pw as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        let (w, _) = self.scaled_image_size(pw, ph);
                        w
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => ph as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        let (_, h) = self.scaled_image_size(pw, ph);
                        h
                    }
                };
                (cmp::max(0, w) as u16, cmp::max(0, h) as u16)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate()
    }
}
impl Spawnable for WindowsImage {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_content(image::DynamicImage::ImageRgba8(image::ImageBuffer::new(0, 0))).into_control()
    }
}

unsafe extern "system" fn handler<T: controls::Image>(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            let i: &mut Image = mem::transmute(param);
            i.call_on_size::<T>(width, height);
        }
        winuser::WM_PAINT => {
            use plygui_api::controls::HasSize;

            let i: &mut Image = mem::transmute(param);
            let (pw, ph) = i.size();
            let i = i.inner_mut().inner_mut().inner_mut();
            let hoffs = DEFAULT_CONTENT_PADDING;
            let voffs = DEFAULT_CONTENT_PADDING;
            let hdiff = hoffs + DEFAULT_CONTENT_PADDING;
            let vdiff = voffs + DEFAULT_CONTENT_PADDING;
            let inner_h = pw as i32 - hdiff;
            let inner_v = ph as i32 - vdiff;

            let (dst_w, dst_h) = i.scaled_image_size(pw, ph);

            let mut bm: wingdi::BITMAP = mem::zeroed();
            let mut ps: winuser::PAINTSTRUCT = mem::zeroed();

            let hdc = winuser::BeginPaint(hwnd, &mut ps);
            let hdc_mem = wingdi::CreateCompatibleDC(hdc);
            wingdi::SelectObject(hdc_mem, i.bmp as *mut c_void); //let hbm_old =
            wingdi::GetObjectW(i.bmp as *mut c_void, mem::size_of::<wingdi::BITMAP>() as i32, &mut bm as *mut _ as *mut c_void);

            let blendfunc = wingdi::BLENDFUNCTION {
                BlendOp: 0,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: 1,
            };

            let (dst_x, dst_y, src_x, src_y, src_w, src_h) = match i.scale {
                types::ImageScalePolicy::FitCenter => {
                    let xoffs = (pw as i32 - dst_w) / 2;
                    let yoffs = (ph as i32 - dst_h) / 2;
                    (xoffs, yoffs, 0, 0, bm.bmWidth, bm.bmHeight)
                }
                types::ImageScalePolicy::CropCenter => {
                    let half_diff_h = (bm.bmWidth - pw as i32) / 2;
                    let half_diff_v = (bm.bmHeight - ph as i32) / 2;
                    (
                        hoffs + cmp::min(hoffs, half_diff_h).abs(),
                        voffs + cmp::min(voffs, half_diff_v).abs(),
                        cmp::max(0, half_diff_h),
                        cmp::max(0, half_diff_v),
                        cmp::min(bm.bmWidth, inner_h),
                        cmp::min(bm.bmHeight, inner_v),
                    )
                }
            };
            wingdi::GdiAlphaBlend(hdc, dst_x, dst_y, dst_w, dst_h, hdc_mem, src_x, src_y, src_w, src_h, blendfunc);

            wingdi::DeleteDC(hdc_mem);
            winuser::EndPaint(hwnd, &ps);
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}

fn fmin(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}
/*fn fmax(a: f32, b: f32) -> f32 {
    // leave for future non-centered fit
    if a > b {
        a
    } else {
        b
    }
}*/

default_impls_as!(Image);
