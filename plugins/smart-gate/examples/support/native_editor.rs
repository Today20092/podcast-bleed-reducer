//! A host-owned Windows parent and render-request button. This never changes consent.
// Layouts/signatures checked against Microsoft's Windows SDK 10.0.22621.0
// WinUser.h (WNDCLASSW, MSG, RECT) and libloaderapi.h. No DPI override is made.
use std::{ffi::c_void, mem, ptr};

type Handle = *mut c_void;
const RUN: u32 = 0x8001;
const CLOSE: u32 = 0x8002;
const STYLE: u32 = 0x00cf0000;

#[repr(C)]
struct WindowClass {
    style: u32,
    procedure: unsafe extern "system" fn(Handle, u32, usize, isize) -> isize,
    class_extra: i32,
    window_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu: *const u16,
    name: *const u16,
}
#[repr(C)]
#[derive(Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct Message {
    window: Handle,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    point: [i32; 2],
}
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(mem::size_of::<WindowClass>() == 72);
    assert!(mem::size_of::<Message>() == 48);
    assert!(mem::size_of::<Rect>() == 16);
};
#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WindowClass) -> u16;
    fn UnregisterClassW(name: *const u16, instance: Handle) -> i32;
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Handle,
        menu: Handle,
        instance: Handle,
        data: Handle,
    ) -> Handle;
    fn DefWindowProcW(window: Handle, message: u32, wparam: usize, lparam: isize) -> isize;
    fn PostMessageW(window: Handle, message: u32, wparam: usize, lparam: isize) -> i32;
    fn DestroyWindow(window: Handle) -> i32;
    fn ShowWindow(window: Handle, command: i32) -> i32;
    fn PeekMessageW(
        message: *mut Message,
        window: Handle,
        first: u32,
        last: u32,
        remove: u32,
    ) -> i32;
    fn TranslateMessage(message: *const Message) -> i32;
    fn DispatchMessageW(message: *const Message) -> isize;
    fn AdjustWindowRectEx(rect: *mut Rect, style: u32, menu: i32, ex: u32) -> i32;
    fn SetWindowPos(
        window: Handle,
        after: Handle,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> i32;
    fn GetDlgItem(parent: Handle, id: i32) -> Handle;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

unsafe extern "system" fn procedure(
    window: Handle,
    message: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    // SAFETY: Windows supplies a live HWND; forwarding/posting does not retain borrowed data.
    unsafe {
        if message == 0x10 {
            PostMessageW(window, CLOSE, 0, 0);
            return 0;
        }
        if message == 0x111 && wparam & 0xffff == 1001 {
            PostMessageW(window, RUN, 0, 0);
            return 0;
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}

pub enum Action {
    Run,
    Close,
}
pub struct Window {
    handle: Handle,
    instance: Handle,
    class: Vec<u16>,
}

impl Window {
    pub fn new(render_button: bool) -> Result<Self, String> {
        // SAFETY: all UTF-16 strings and class storage outlive their corresponding API calls.
        unsafe {
            let instance = GetModuleHandleW(ptr::null());
            let class = wide(&format!("CleanSlateGateNativeCheck{}", std::process::id()));
            let definition = WindowClass {
                style: 0,
                procedure,
                class_extra: 0,
                window_extra: 0,
                instance,
                icon: ptr::null_mut(),
                cursor: ptr::null_mut(),
                background: 6_usize as Handle,
                menu: ptr::null(),
                name: class.as_ptr(),
            };
            if RegisterClassW(&definition) == 0 {
                return Err("Cannot register native check window".into());
            }
            let handle = CreateWindowExW(
                // Keep this bounded engineering check visible over other host windows.
                if render_button { 0x00000008 } else { 0 },
                class.as_ptr(),
                wide(if render_button {
                    "CleanSlate Gate — engineering VST3 consent check"
                } else {
                    "CleanSlate Smart Gate — GUI preview"
                })
                .as_ptr(),
                STYLE,
                100,
                100,
                660,
                540,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut(),
            );
            if handle.is_null() {
                UnregisterClassW(class.as_ptr(), instance);
                return Err("Cannot create native check window".into());
            }
            let button = CreateWindowExW(
                0,
                wide(if render_button { "BUTTON" } else { "STATIC" }).as_ptr(),
                wide(if render_button {
                    "Run armed ABI check"
                } else {
                    "GUI preview · no audio device"
                })
                .as_ptr(),
                if render_button {
                    0x50010000
                } else {
                    0x50000000
                },
                16,
                448,
                250,
                28,
                handle,
                1001_usize as Handle,
                instance,
                ptr::null_mut(),
            );
            let window = Self {
                handle,
                instance,
                class,
            };
            if button.is_null() || !resize(handle as usize, 1000, 620) {
                return Err("Cannot prepare native render button".into());
            }
            ShowWindow(handle, 5);
            Ok(window)
        }
    }
    pub fn handle(&self) -> Handle {
        self.handle
    }
    pub fn pump(&self) -> Option<Action> {
        // SAFETY: message storage is initialized and used only by its owning GUI thread.
        unsafe {
            let mut message: Message = mem::zeroed();
            while PeekMessageW(&mut message, ptr::null_mut(), 0, 0, 1) != 0 {
                if message.message == CLOSE || message.message == 0x12 {
                    return Some(Action::Close);
                }
                if message.message == RUN {
                    return Some(Action::Run);
                }
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        None
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        // SAFETY: the editor must be detached before this host-owned parent is destroyed.
        unsafe {
            DestroyWindow(self.handle);
            UnregisterClassW(self.class.as_ptr(), self.instance);
        }
    }
}

pub fn resize(parent: usize, width: i32, height: i32) -> bool {
    if !(1..=4096).contains(&width) || !(1..=4096).contains(&height) {
        return false;
    }
    // SAFETY: caller retains the live host-owned parent for the complete editor lifetime.
    unsafe {
        let mut rect = Rect {
            right: width,
            bottom: height + 48,
            ..Rect::default()
        };
        AdjustWindowRectEx(&mut rect, STYLE, 0, 0) != 0
            && SetWindowPos(
                parent as Handle,
                ptr::null_mut(),
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                0x16,
            ) != 0
            && SetWindowPos(
                GetDlgItem(parent as Handle, 1001),
                ptr::null_mut(),
                16,
                height + 8,
                250,
                28,
                0x14,
            ) != 0
    }
}
