//! Bounded in-memory host-owned VST3 state stream.
use std::{
    ffi::c_void,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};
use vst3::Steinberg::Vst::IComponentTrait;
use vst3::{Class, ComWrapper, Steinberg::*};
const LIMIT: usize = 1024 * 1024;
struct Stream(Arc<Mutex<Cursor<Vec<u8>>>>);
impl Class for Stream {
    type Interfaces = (IBStream,);
}
impl IBStreamTrait for Stream {
    unsafe fn read(&self, buffer: *mut c_void, count: i32, read: *mut i32) -> tresult {
        if count < 0 || count as usize > LIMIT || (count > 0 && buffer.is_null()) {
            return kInvalidArgument;
        }
        if count == 0 {
            if !read.is_null() {
                unsafe {
                    *read = 0;
                }
            }
            return kResultOk;
        }
        let Ok(mut cursor) = self.0.lock() else {
            return kResultFalse;
        };
        let Ok(length) =
            cursor.read(unsafe { std::slice::from_raw_parts_mut(buffer.cast(), count as usize) })
        else {
            return kResultFalse;
        };
        if !read.is_null() {
            unsafe {
                *read = length as i32;
            }
        }
        if length == count as usize {
            kResultOk
        } else {
            kResultFalse
        }
    }
    unsafe fn write(&self, buffer: *mut c_void, count: i32, written: *mut i32) -> tresult {
        if count < 0 || count as usize > LIMIT || (count > 0 && buffer.is_null()) {
            return kInvalidArgument;
        }
        if count == 0 {
            if !written.is_null() {
                unsafe {
                    *written = 0;
                }
            }
            return kResultOk;
        }
        let Ok(mut cursor) = self.0.lock() else {
            return kResultFalse;
        };
        if cursor.position() + count as u64 > LIMIT as u64 {
            return kInvalidArgument;
        }
        if cursor
            .write_all(unsafe { std::slice::from_raw_parts(buffer.cast(), count as usize) })
            .is_err()
        {
            return kResultFalse;
        }
        if !written.is_null() {
            unsafe {
                *written = count;
            }
        }
        kResultOk
    }
    unsafe fn seek(&self, position: i64, mode: i32, result: *mut i64) -> tresult {
        let seek = match mode {
            0 if position >= 0 => SeekFrom::Start(position as u64),
            1 => SeekFrom::Current(position),
            2 => SeekFrom::End(position),
            _ => return kInvalidArgument,
        };
        let Ok(mut cursor) = self.0.lock() else {
            return kResultFalse;
        };
        let previous = cursor.position();
        let Ok(position) = cursor.seek(seek) else {
            return kInvalidArgument;
        };
        if position > LIMIT as u64 {
            cursor.set_position(previous);
            return kInvalidArgument;
        }
        if !result.is_null() {
            unsafe {
                *result = position as i64;
            }
        }
        kResultOk
    }
    unsafe fn tell(&self, result: *mut i64) -> tresult {
        if result.is_null() {
            return kInvalidArgument;
        }
        let Ok(cursor) = self.0.lock() else {
            return kResultFalse;
        };
        unsafe {
            *result = cursor.position() as i64;
        }
        kResultOk
    }
}
pub unsafe fn save(component: &vst3::ComPtr<Vst::IComponent>) -> Vec<u8> {
    let cursor = Arc::new(Mutex::new(Cursor::new(Vec::new())));
    let stream = ComWrapper::new(Stream(cursor.clone()))
        .to_com_ptr::<IBStream>()
        .unwrap();
    unsafe {
        assert_eq!(component.getState(stream.as_ptr()), kResultOk);
    }
    let bytes = cursor.lock().unwrap().get_ref().clone();
    assert!(!bytes.is_empty());
    bytes
}
pub unsafe fn load(component: &vst3::ComPtr<Vst::IComponent>, bytes: Vec<u8>) {
    assert!(bytes.len() <= LIMIT);
    let stream = ComWrapper::new(Stream(Arc::new(Mutex::new(Cursor::new(bytes)))))
        .to_com_ptr::<IBStream>()
        .unwrap();
    unsafe {
        assert_eq!(component.setState(stream.as_ptr()), kResultOk);
    }
}
