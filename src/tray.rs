use crate::common::{self, *};

use winapi::um::shellapi;

pub const MESSAGE: u32 = 0xbaba;

#[repr(C)]
pub struct WindowsTray {
    label: String,
    icon: image::DynamicImage,
    cfg: shellapi::NOTIFYICONDATAW,
    menu: (windef::HMENU, Vec<callbacks::Action>, isize),
    on_close: Option<callbacks::OnClose>,
    this: *mut Tray,
}

pub type Tray = AMember<ACloseable<ATray<WindowsTray>>>;

impl WindowsTray {
    pub(crate) fn toggle_menu(&mut self) {
        if !self.menu.0.is_null() {
            unsafe {
                let hwnd = (&*self.this).native_id().into();
                if self.menu.2 > -2 {
                    self.menu.2 = -2;
                    winuser::SendMessageW(hwnd, winuser::WM_CANCELMODE, 0, 0);
                } else {
                    self.menu.2 = -1;
                    let mut click_point = mem::zeroed();
                    winuser::GetCursorPos(&mut click_point);
                    winuser::TrackPopupMenu(self.menu.0, winuser::TPM_LEFTALIGN | winuser::TPM_LEFTBUTTON | winuser::TPM_BOTTOMALIGN, click_point.x, click_point.y, 0, hwnd, ptr::null_mut());
                }
            }
        }
    }
    pub(crate) fn is_menu_shown(&self) -> bool {
        self.menu.2 > -2
    }
    pub(crate) fn select_menu(&mut self, id: usize) {
        self.menu.2 = id as isize;
    }
    pub(crate) fn run_menu(&mut self, this: &mut Tray) {
        if self.menu.2 > -1 {
            if let Some(a) = self.menu.1.get_mut(self.menu.2 as usize) {
                (a.as_mut())(this);
            }
        }
        self.menu.2 = -2;
    }
    fn install_image(&mut self) {
    	use plygui_api::external::image::GenericImageView;
    	
    	let i = unsafe {
    		let status_size = winuser::GetSystemMetrics(winuser::SM_CXSMICON) as u32;
    		self.icon.resize(status_size, status_size, image::imageops::FilterType::Lanczos3)
    	};
    	
    	let (w,h) = i.dimensions();
    	let mut mask = image::ImageBuffer::new(w, h);
	    for x in 0..w {
	        for y in 0..h {
	            let bright = std::u8::MAX;
	            mask.put_pixel(x, y, image::Rgba([bright, bright, bright, 0x0]));
	        }
	    }
    	unsafe {
    		if !self.cfg.hIcon.is_null() {
    			winuser::DestroyIcon(self.cfg.hIcon);
    		}
	        let mut ii: winuser::ICONINFO = mem::zeroed();
	        ii.fIcon = minwindef::TRUE;
	        common::image_to_native(&image::DynamicImage::ImageRgba8(mask), &mut ii.hbmMask);
	        common::image_to_native(&i, &mut ii.hbmColor);
	        self.cfg.hIcon = winuser::CreateIconIndirect(&mut ii);
	        if shellapi::Shell_NotifyIconW(shellapi::NIM_MODIFY, &mut self.cfg) == minwindef::FALSE {
                common::log_error();
            }
    	}

    }
}

impl HasLabelInner for WindowsTray {
    fn label(&self, _base: &MemberBase) -> Cow<str> {
        Cow::Borrowed(self.label.as_ref())
    }
    fn set_label(&mut self, _base: &mut MemberBase, label: Cow<str>) {
        self.label = label.into();
        if !self.cfg.hWnd.is_null() {
            let control_name = common::str_to_wchar(&self.label);
            unsafe {
                winuser::SetWindowTextW(self.cfg.hWnd, control_name.as_ptr());
            }
        }
    }
}

impl CloseableInner for WindowsTray {
    fn close(&mut self, skip_callbacks: bool) -> bool {
        if !skip_callbacks {
            if let Some(ref mut on_close) = self.on_close {
                if !(on_close.as_mut())(unsafe { &mut *self.this }) {
                    return false;
                }
            }
        }
        unsafe {
            if shellapi::Shell_NotifyIconW(shellapi::NIM_DELETE, &mut self.cfg) == minwindef::FALSE {
                common::log_error();
            }
        }
        true
    }
    fn on_close(&mut self, callback: Option<callbacks::OnClose>) {
        self.on_close = callback;
    }
    fn application<'a>(&'a self, base: &'a MemberBase) -> &'a dyn controls::Application {
        unsafe { utils::base_to_impl::<Tray>(base) }.inner().application_impl::<crate::application::Application>()
    }
    fn application_mut<'a>(&'a mut self, base: &'a mut MemberBase) -> &'a mut dyn controls::Application {
        unsafe { utils::base_to_impl_mut::<Tray>(base) }.inner_mut().application_impl_mut::<crate::application::Application>()
    }
}

impl HasImageInner for WindowsTray {
	fn image(&self, _base: &MemberBase) -> Cow<image::DynamicImage> {
        Cow::Borrowed(&self.icon)
    }
    #[inline]
    fn set_image(&mut self, _base: &mut MemberBase, i: Cow<image::DynamicImage>) {
    	self.icon = i.into_owned();
    	self.install_image();
    }
}

impl<O: controls::Tray> NewTrayInner<O> for WindowsTray {
    fn with_uninit_params(u: &mut mem::MaybeUninit<O>, title: &str, icon: image::DynamicImage, menu: types::Menu) -> Self {
        WindowsTray {
            label: title.into(),
            icon: icon,
            cfg: unsafe { mem::zeroed() },
            menu: (ptr::null_mut(), if menu.is_some() { Vec::new() } else { vec![] }, -2),
            on_close: None,
            this: u as *mut _ as *mut Tray,
        }
    }
}
impl TrayInner for WindowsTray {
    fn with_params<S: AsRef<str>>(app: &mut dyn controls::Application, title: S, icon: image::DynamicImage, menu: types::Menu) -> Box<dyn controls::Tray> {
        let mut b: Box<mem::MaybeUninit<Tray>> = Box::new_uninit();
        let ab = AMember::with_inner(
            ACloseable::with_inner(
                ATray::with_inner(
                    <Self as NewTrayInner<Tray>>::with_uninit_params(b.as_mut(), title.as_ref(), icon, types::Menu::None),
                ),
	            app.as_any_mut().downcast_mut::<crate::application::Application>().unwrap()
            )
        );
        let mut t = unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        };
        {
            let app = app.as_any_mut().downcast_mut::<crate::application::Application>().unwrap();
            let id = unsafe { controls::Member::id(t.as_ref()).into_raw() as u32 };
            let tt = t.inner_mut().inner_mut().inner_mut();
            let tip_size = tt.cfg.szTip.len();
            let title = OsStr::new(tt.label.as_str()).encode_wide().take(tip_size - 1).chain(Some(0).into_iter()).collect::<Vec<_>>();
    
            tt.cfg.hWnd = app.inner().native_id().into();
            tt.cfg.cbSize = mem::size_of::<shellapi::NOTIFYICONDATAW>() as u32;
            tt.cfg.uID = id;
            //t.inner_mut().inner_mut().cfg.hIcon = unsafe { winuser::GetClassLongW(app.inner().root.into(), winuser::GCL_HICON) as windef::HICON };
    
            unsafe {
                commctrl::LoadIconMetric(ptr::null_mut(), winuser::MAKEINTRESOURCEW(32512), commctrl::LIM_SMALL as i32, &mut tt.cfg.hIcon);
            }
    
            tt.cfg.uFlags = shellapi::NIF_ICON | shellapi::NIF_TIP | shellapi::NIF_MESSAGE | shellapi::NIF_SHOWTIP;
            tt.cfg.uCallbackMessage = MESSAGE;
            unsafe {
                tt.cfg.szTip[..title.len()].clone_from_slice(title.as_slice());
    	        if shellapi::Shell_NotifyIconW(shellapi::NIM_ADD, &mut tt.cfg) == minwindef::FALSE {
                    common::log_error();
                }
                *tt.cfg.u.uVersion_mut() = shellapi::NOTIFYICON_VERSION_4;
                if shellapi::Shell_NotifyIconW(shellapi::NIM_SETVERSION, &mut tt.cfg) == minwindef::FALSE {
                    common::log_error();
                }
            }
            if let Some(items) = menu {
                unsafe {
                    let menu = winuser::CreatePopupMenu();
                    common::make_menu(menu, items, &mut tt.menu.1);
                    tt.menu.0 = menu;
                }
            }
            tt.install_image();
        }
        t
    }
}

impl HasNativeIdInner for WindowsTray {
    type Id = common::Hwnd;

    fn native_id(&self) -> Self::Id {
        self.cfg.hWnd.into()
    }
}

impl MemberInner for WindowsTray {}

impl Drop for WindowsTray {
    fn drop(&mut self) {
        unsafe {
            if !self.menu.0.is_null() {
                winuser::DeleteMenu(self.menu.0, 0, 0);
                if shellapi::Shell_NotifyIconW(shellapi::NIM_DELETE, &mut self.cfg) == minwindef::FALSE {
                    common::log_error();
                }
            }
        }
    }
}
