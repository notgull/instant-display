// SPDX-License-Identifier: MIT OR Apache-2.0 OR Zlib

//! A simple type that allows you to get a display handle if you don't need a window.

use raw_window_handle::{self as rwh, HasRawDisplayHandle, RawDisplayHandle};

/// The whole point.
pub struct Display {
    /// This type is a private field to prevent construction.
    _private: (),

    #[cfg(all(
        unix,
        not(any(target_vendor = "apple", target_os = "android", target_os = "redox"))
    ))]
    global_display: (&'static GlobalDisplay, usize),
}

impl Display {
    /// Create a new `Display`.
    pub fn new() -> Result<Self, Error> {
        cfg_if::cfg_if! {
            if #[cfg(all(unix, not(any(target_vendor = "apple", target_os = "android", target_os = "redox"))))] {
                let (display, screen) = get_display().as_ref().map_err(Error)?;
                Ok(Self {
                    _private: (),
                    global_display: (display, *screen),
                })
            } else {
                Ok(Self {
                    _private: ()
                })
            }
        }
    }

    fn raw_handle(&self) -> RawDisplayHandle {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                RawDisplayHandle::Windows(rwh::WindowsDisplayHandle::empty())
            } else if #[cfg(target_os = "macos")] {
                RawDisplayHandle::AppKit(rwh::AppKitDisplayHandle::empty())
            } else if #[cfg(target_vendor = "apple")] {
                RawDisplayHandle::UiKit(rwh::UiKitDisplayHandle::empty())
            } else if #[cfg(target_os = "redox")] {
                RawDisplayHandle::Orbital(rwh::OrbitalDisplayHandle::empty())
            } else if #[cfg(target_os = "android")] {
                RawDisplayHandle::Android(rwh::AndroidDisplayHandle::empty())
            } else if #[cfg(target_family = "wasm")] {
                RawDisplayHandle::Web(rwh::WebDisplayHandle::empty())
            } else if #[cfg(target_os = "haiku")] {
                RawDisplayHandle::Haiku(rwh::HaikuDisplayHandle::empty())
            } else if #[cfg(unix)] {
                let mut handle = rwh::XcbDisplayHandle::empty();
                handle.connection = self.global_display.0.get_raw_xcb_connection();
                handle.screen = self.global_display.1 as _;
                RawDisplayHandle::Xcb(handle)
            } else {
                compile_error!("Unsupported platform");
            }
        }
    }
}

unsafe impl HasRawDisplayHandle for Display {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.raw_handle()
    }
}

impl rwh::HasDisplayHandle for Display {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        unsafe { Ok(rwh::DisplayHandle::borrow_raw(self.raw_handle())) }
    }
}

/// The inner error type.
#[derive(Debug)]
pub struct Error(ErrorImpl);

#[cfg(all(
    unix,
    not(any(target_vendor = "apple", target_os = "android", target_os = "redox"))
))]
mod global_display {
    pub(crate) use x11rb::xcb_ffi::XCBConnection as GlobalDisplay;

    use once_cell::sync::OnceCell;
    use x11rb::errors::ConnectError;

    pub(crate) type ErrorImpl = &'static ConnectError;
    type GlobalResult = Result<(GlobalDisplay, usize), ConnectError>;

    #[inline]
    pub(crate) fn get_display() -> &'static GlobalResult {
        static GLOBAL_DISPLAY: OnceCell<GlobalResult> = OnceCell::new();
        GLOBAL_DISPLAY.get_or_init(|| GlobalDisplay::connect(None))
    }
}

#[cfg(all(
    unix,
    not(any(target_vendor = "apple", target_os = "android", target_os = "redox"))
))]
use global_display::*;

#[cfg(not(all(
    unix,
    not(any(target_vendor = "apple", target_os = "android", target_os = "redox"))
)))]
type ErrorImpl = std::convert::Infallible;
