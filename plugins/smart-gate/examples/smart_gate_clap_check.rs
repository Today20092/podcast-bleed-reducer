//! Device-free Gate/probe coexistence, state and exact pass-through through the actual CLAP ABI.
#[path = "support/candidate_loader.rs"]
mod candidate_loader;
use clap_sys::{
    audio_buffer::*,
    entry::*,
    events::*,
    ext::{audio_ports_config::*, gui::*, latency::*, params::*, state::*},
    factory::plugin_factory::*,
    host::*,
    plugin::*,
    process::*,
    stream::*,
    version::*,
};
use std::{
    ffi::{CStr, CString, c_void},
    io::{Cursor, Read},
    mem, ptr,
};

struct Bundle {
    _library: libloading::Library,
    entry: *const clap_plugin_entry,
    factory: *const clap_plugin_factory,
}
impl Bundle {
    unsafe fn load(path: &str, expected: &str, isolated: bool) -> Self {
        unsafe {
            let library = candidate_loader::load(path, isolated);
            let entry = *library
                .get::<*const clap_plugin_entry>(b"clap_entry\0")
                .unwrap();
            assert!(((*entry).init.unwrap())(
                CString::new(path).unwrap().as_ptr()
            ));
            let factory = ((*entry).get_factory.unwrap())(CLAP_PLUGIN_FACTORY_ID.as_ptr())
                .cast::<clap_plugin_factory>();
            assert!(!factory.is_null());
            assert_eq!(((*factory).get_plugin_count.unwrap())(factory), 1);
            let descriptor = ((*factory).get_plugin_descriptor.unwrap())(factory, 0);
            assert_eq!(CStr::from_ptr((*descriptor).id).to_str().unwrap(), expected);
            Self {
                _library: library,
                entry,
                factory,
            }
        }
    }
    unsafe fn create(&self, host: &clap_host) -> *const clap_plugin {
        unsafe {
            let descriptor = ((*self.factory).get_plugin_descriptor.unwrap())(self.factory, 0);
            let plugin =
                ((*self.factory).create_plugin.unwrap())(self.factory, host, (*descriptor).id);
            assert!(!plugin.is_null());
            assert!(((*plugin).init.unwrap())(plugin));
            plugin
        }
    }
}
impl Drop for Bundle {
    fn drop(&mut self) {
        unsafe {
            ((*self.entry).deinit.unwrap())();
        }
    }
}

unsafe extern "C" fn no_extension(
    _: *const clap_host,
    _: *const std::ffi::c_char,
) -> *const c_void {
    ptr::null()
}
unsafe extern "C" fn request(_: *const clap_host) {}
unsafe extern "C" fn empty(_: *const clap_input_events) -> u32 {
    0
}
unsafe extern "C" fn no_event(_: *const clap_input_events, _: u32) -> *const clap_event_header {
    ptr::null()
}
unsafe extern "C" fn accept(_: *const clap_output_events, _: *const clap_event_header) -> bool {
    true
}
unsafe extern "C" fn one(_: *const clap_input_events) -> u32 {
    1
}
unsafe extern "C" fn event(list: *const clap_input_events, _: u32) -> *const clap_event_header {
    unsafe { (*list).ctx.cast() }
}
unsafe extern "C" fn write_state(
    stream: *const clap_ostream,
    data: *const c_void,
    size: u64,
) -> i64 {
    unsafe {
        let bytes = &mut *((*stream).ctx.cast::<Vec<u8>>());
        bytes.extend_from_slice(std::slice::from_raw_parts(data.cast(), size as usize));
        size as i64
    }
}
unsafe extern "C" fn read_state(stream: *const clap_istream, data: *mut c_void, size: u64) -> i64 {
    unsafe {
        (&mut *((*stream).ctx.cast::<Cursor<Vec<u8>>>()))
            .read(std::slice::from_raw_parts_mut(data.cast(), size as usize))
            .unwrap() as i64
    }
}

unsafe fn bypass(plugin: *const clap_plugin, value: bool) {
    unsafe {
        let params = &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_PARAMS.as_ptr())
            .cast::<clap_plugin_params>();
        assert_eq!((params.count.unwrap())(plugin), 2);
        let mut info = mem::zeroed();
        assert!((params.get_info.unwrap())(plugin, 0, &mut info));
        assert_eq!(
            CStr::from_ptr(info.name.as_ptr()).to_str().unwrap(),
            "Bypass"
        );
        let mut value = clap_event_param_value {
            header: clap_event_header {
                size: mem::size_of::<clap_event_param_value>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                type_: CLAP_EVENT_PARAM_VALUE,
                flags: 0,
            },
            param_id: info.id,
            cookie: info.cookie,
            note_id: -1,
            port_index: -1,
            channel: -1,
            key: -1,
            value: if value { 1.0 } else { 0.0 },
        };
        let input = clap_input_events {
            ctx: (&mut value as *mut clap_event_param_value).cast(),
            size: Some(one),
            get: Some(event),
        };
        let output = clap_output_events {
            ctx: ptr::null_mut(),
            try_push: Some(accept),
        };
        (params.flush.unwrap())(plugin, &input, &output);
        let mut actual = -1.0;
        assert!((params.get_value.unwrap())(plugin, info.id, &mut actual));
        assert_eq!(actual, value.value);
    }
}

unsafe fn state_round_trip(plugin: *const clap_plugin, gate: bool) {
    unsafe {
        let state = &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_STATE.as_ptr())
            .cast::<clap_plugin_state>();
        let mut bytes = Vec::new();
        let stream = clap_ostream {
            ctx: (&mut bytes as *mut Vec<u8>).cast(),
            write: Some(write_state),
        };
        if gate {
            bypass(plugin, true);
        }
        assert!((state.save.unwrap())(plugin, &stream));
        assert!(!bytes.is_empty());
        if gate {
            bypass(plugin, false);
        }
        let mut cursor = Cursor::new(bytes.clone());
        let input = clap_istream {
            ctx: (&mut cursor as *mut Cursor<Vec<u8>>).cast(),
            read: Some(read_state),
        };
        assert!((state.load.unwrap())(plugin, &input));
        let mut restored = Vec::new();
        let output = clap_ostream {
            ctx: (&mut restored as *mut Vec<u8>).cast(),
            write: Some(write_state),
        };
        assert!((state.save.unwrap())(plugin, &output));
        assert_eq!(
            bytes, restored,
            "State round trip changed this product's state"
        );
    }
}

unsafe fn process(plugin: *const clap_plugin, channels: u32, frames: usize, in_place: bool) {
    unsafe {
        let mut input: Vec<Vec<f32>> = (0..channels)
            .map(|channel| {
                (0..frames)
                    .map(|i| (i as f32 * 0.31 + channel as f32).sin() * 0.25)
                    .collect()
            })
            .collect();
        let expected: Vec<Vec<u32>> = input
            .iter()
            .map(|c| c.iter().map(|x| x.to_bits()).collect())
            .collect();
        let mut output = vec![vec![f32::NAN; frames]; channels as usize];
        let mut input_ptrs: Vec<_> = input.iter_mut().map(|c| c.as_mut_ptr()).collect();
        let mut output_ptrs: Vec<_> = if in_place {
            input_ptrs.clone()
        } else {
            output.iter_mut().map(|c| c.as_mut_ptr()).collect()
        };
        let input_bus = clap_audio_buffer {
            data32: input_ptrs.as_mut_ptr(),
            data64: ptr::null_mut(),
            channel_count: channels,
            latency: 0,
            constant_mask: 0,
        };
        let mut output_bus = clap_audio_buffer {
            data32: output_ptrs.as_mut_ptr(),
            data64: ptr::null_mut(),
            channel_count: channels,
            latency: 0,
            constant_mask: 0,
        };
        let events = clap_input_events {
            ctx: ptr::null_mut(),
            size: Some(empty),
            get: Some(no_event),
        };
        let out_events = clap_output_events {
            ctx: ptr::null_mut(),
            try_push: Some(accept),
        };
        let block = clap_process {
            steady_time: -1,
            frames_count: frames as u32,
            transport: ptr::null(),
            audio_inputs: &input_bus,
            audio_outputs: &mut output_bus,
            audio_inputs_count: 1,
            audio_outputs_count: 1,
            in_events: &events,
            out_events: &out_events,
        };
        assert_ne!(
            ((*plugin).process.unwrap())(plugin, &block),
            CLAP_PROCESS_ERROR
        );
        let actual = if in_place { input } else { output };
        let actual: Vec<Vec<u32>> = actual
            .iter()
            .map(|c| c.iter().map(|x| x.to_bits()).collect())
            .collect();
        assert_eq!(actual, expected);
    }
}

fn main() {
    let mut paths: Vec<_> = std::env::args().skip(1).collect();
    let isolated = paths.first().is_some_and(|arg| arg == "--candidate-loader");
    if isolated {
        paths.remove(0);
        assert_eq!(paths.len(), 1, "--candidate-loader <Gate binary>");
        candidate_loader::configure(&paths[0]);
    }
    assert_eq!(
        paths.len(),
        if isolated { 1 } else { 2 },
        "smart_gate_clap_check <Gate binary> <preserved probe binary>"
    );
    // Libraries, host callbacks and audio storage stay live until each instance is stopped/destroyed.
    unsafe {
        let gate = Bundle::load(&paths[0], "org.cleanslate.smart-gate", isolated);
        let probe = (!isolated).then(|| Bundle::load(&paths[1], "org.cleanslate.fl-probe", false));
        let host = clap_host {
            clap_version: CLAP_VERSION,
            host_data: ptr::null_mut(),
            name: c"CleanSlate Gate check".as_ptr(),
            vendor: c"CleanSlate".as_ptr(),
            url: c"https://github.com/Today20092/podcast-bleed-reducer".as_ptr(),
            version: c"0.0.1".as_ptr(),
            get_extension: Some(no_extension),
            request_restart: Some(request),
            request_process: Some(request),
            request_callback: Some(request),
        };
        if let Some(probe) = &probe {
            let old = probe.create(&host);
            state_round_trip(old, false);
            ((*old).destroy.unwrap())(old);
        }
        let plugin = gate.create(&host);
        state_round_trip(plugin, true);
        let gui = &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_GUI.as_ptr())
            .cast::<clap_plugin_gui>();
        let (mut width, mut height) = (0, 0);
        assert!((gui.get_size.unwrap())(plugin, &mut width, &mut height));
        assert_eq!((width, height), (640, 440));
        let configs =
            &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_AUDIO_PORTS_CONFIG.as_ptr())
                .cast::<clap_plugin_audio_ports_config>();
        for channels in [1, 2] {
            let mut selected = false;
            for index in 0..(configs.count.unwrap())(plugin) {
                let mut config = mem::zeroed();
                assert!((configs.get.unwrap())(plugin, index, &mut config));
                if config.main_input_channel_count == channels {
                    assert!((configs.select.unwrap())(plugin, config.id));
                    selected = true;
                    break;
                }
            }
            assert!(selected);
            for rate in [44100.0, 48000.0, 96000.0] {
                for enabled_bypass in [false, true] {
                    bypass(plugin, enabled_bypass);
                    assert!(((*plugin).activate.unwrap())(plugin, rate, 1, 1024));
                    let latency =
                        &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_LATENCY.as_ptr())
                            .cast::<clap_plugin_latency>();
                    assert_eq!((latency.get.unwrap())(plugin), 0);
                    assert!(((*plugin).start_processing.unwrap())(plugin));
                    for frames in [1, 17, 256, 1024] {
                        for in_place in [false, true] {
                            process(plugin, channels, frames, in_place);
                        }
                    }
                    ((*plugin).stop_processing.unwrap())(plugin);
                    ((*plugin).deactivate.unwrap())(plugin);
                }
            }
        }
        ((*plugin).destroy.unwrap())(plugin);
        if isolated {
            candidate_loader::verify(&paths[0]);
        }
    }
    if isolated {
        println!(
            "PASS: isolated candidate-local DirectML load and CLAP activation; no arming or inference qualification."
        );
        return;
    }
    println!(
        "PASS: Gate/probe CLAP IDs coexist; independent state round trips; Gate bypass, mono/stereo, 44.1/48/96 kHz, in-place/separate buffers, exact unity and zero latency; editor contract only."
    );
}
