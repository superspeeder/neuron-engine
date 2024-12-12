use std::ffi::c_int;
use ash::{khr, vk};
use ash::vk::{xcb_connection_t, Display, VisualID};
use winit::raw_window_handle::RawDisplayHandle;
use crate::render::context::instance::Instance;

#[cfg(target_os = "linux")]
unsafe fn grab_visualid_xlib(display: *mut Display, screen: c_int) -> VisualID {
    (*x11::xlib::XDefaultVisual(display as *mut _, screen)).visualid as VisualID
}

#[cfg(target_os = "linux")]
unsafe fn grab_visualid_xcb(connection: *mut xcb_connection_t, screen: c_int) -> VisualID {
    let conn = xcb::Connection::from_raw_conn(connection as *mut _);
    conn.get_setup().roots().nth(screen as _).unwrap().root_visual() as VisualID
}

#[cfg(not(target_os = "linux"))]
unsafe fn grab_visualid_xlib(_display: *mut Display, _screen: c_int) -> VisualID {
    unreachable!()
}

#[cfg(not(target_os = "linux"))]
unsafe fn grab_visualid_xcb(_connection: *mut xcb_connection_t, _screen: c_int) -> VisualID {
    unreachable!()
}

pub(crate) fn can_present(display_handle: &RawDisplayHandle, family: u32, instance: &Instance, physical_device: vk::PhysicalDevice) -> bool {
    match display_handle {
        &RawDisplayHandle::Windows(_) => unsafe {
            let loader = instance.load_extension(khr::win32_surface::Instance::new);

            loader.get_physical_device_win32_presentation_support(physical_device, family)
        },
        &RawDisplayHandle::Xlib(dh) => unsafe {
            let loader = instance.load_extension(khr::xlib_surface::Instance::new);
            let display: *mut Display = dh.display.unwrap().as_ptr() as *mut _;
            loader.get_physical_device_xlib_presentation_support(physical_device, family, display, grab_visualid_xlib(display, dh.screen))
        },
        &RawDisplayHandle::Xcb(dh) => unsafe {
            let loader = instance.load_extension(khr::xcb_surface::Instance::new);
            let connection: *mut xcb_connection_t = dh.connection.unwrap().as_ptr() as *mut _;
            loader.get_physical_device_xcb_presentation_support(physical_device, family, connection.as_mut().unwrap(), grab_visualid_xcb(connection, dh.screen))
        },
        _ => false,
    }
}