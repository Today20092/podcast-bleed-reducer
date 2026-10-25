//! Synthetic CLAP host: validates the actual bundle without controlling FL Studio.
use clap_sys::{
    audio_buffer::*,
    entry::*,
    events::*,
    ext::{audio_ports_config::*, gui::*, params::*, render::*, state::*},
    factory::plugin_factory::*,
    fixedpoint::CLAP_SECTIME_FACTOR,
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
    sync::atomic::{AtomicU64, Ordering::Relaxed},
};

static RESTARTS: AtomicU64 = AtomicU64::new(0);
unsafe extern "C" fn restart(_: *const clap_host) {
    RESTARTS.fetch_add(1, Relaxed);
}
unsafe extern "C" fn write_state(
    stream: *const clap_ostream,
    data: *const c_void,
    size: u64,
) -> i64 {
    unsafe {
        let bytes = &mut *((*stream).ctx.cast::<Vec<u8>>());
        bytes.extend_from_slice(std::slice::from_raw_parts(data.cast::<u8>(), size as usize));
        size as i64
    }
}
unsafe extern "C" fn read_state(stream: *const clap_istream, data: *mut c_void, size: u64) -> i64 {
    unsafe {
        let cursor = &mut *((*stream).ctx.cast::<Cursor<Vec<u8>>>());
        cursor
            .read(std::slice::from_raw_parts_mut(
                data.cast::<u8>(),
                size as usize,
            ))
            .unwrap() as i64
    }
}

unsafe fn state_round_trip(plugin: *const clap_plugin) {
    unsafe {
        let state = &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_STATE.as_ptr())
            .cast::<clap_plugin_state>();
        let params = &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_PARAMS.as_ptr())
            .cast::<clap_plugin_params>();
        let mut before = Vec::new();
        for index in 0..3 {
            let mut info = mem::zeroed();
            assert!((params.get_info.unwrap())(plugin, index, &mut info));
            let mut value = 0.0;
            assert!((params.get_value.unwrap())(plugin, info.id, &mut value));
            before.push((info.id, value));
        }
        let mut bytes = Vec::new();
        let output = clap_ostream {
            ctx: (&mut bytes as *mut Vec<u8>).cast(),
            write: Some(write_state),
        };
        assert!((state.save.unwrap())(plugin, &output));
        set_parameter(plugin, 0, 0.0);
        set_parameter(plugin, 2, 1.0);
        let mut cursor = Cursor::new(bytes);
        let input = clap_istream {
            ctx: (&mut cursor as *mut Cursor<Vec<u8>>).cast(),
            read: Some(read_state),
        };
        assert!((state.load.unwrap())(plugin, &input));
        for (id, expected) in before {
            let mut value = -1.0;
            assert!((params.get_value.unwrap())(plugin, id, &mut value));
            assert_eq!(value, expected);
        }
    }
}

unsafe fn load_legacy_state(plugin: *const clap_plugin, member: usize) {
    // Exact 0.0.1 parameter shape observed read-only in the owner's saved test FLP.
    let json = if member == 1 {
        br#"{"version":"0.0.1","params":{"probe_group":{"i32":1},"probe_member":{"i32":1}},"fields":{}}"#.as_slice()
    } else {
        br#"{"version":"0.0.1","params":{"probe_group":{"i32":1},"probe_member":{"i32":2}},"fields":{}}"#.as_slice()
    };
    unsafe {
        let state = &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_STATE.as_ptr())
            .cast::<clap_plugin_state>();
        let params = &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_PARAMS.as_ptr())
            .cast::<clap_plugin_params>();
        let mut bytes = (json.len() as u64).to_le_bytes().to_vec();
        bytes.extend_from_slice(json);
        let mut cursor = Cursor::new(bytes);
        let input = clap_istream {
            ctx: (&mut cursor as *mut Cursor<Vec<u8>>).cast(),
            read: Some(read_state),
        };
        assert!((state.load.unwrap())(plugin, &input));
        for (index, expected) in [1.0, (member - 1) as f64, 0.0].into_iter().enumerate() {
            let mut info = mem::zeroed();
            assert!((params.get_info.unwrap())(plugin, index as u32, &mut info));
            let mut actual = -1.0;
            assert!((params.get_value.unwrap())(plugin, info.id, &mut actual));
            // The adapter reconstructs stepped values from f32 normalized state (1/7 is inexact).
            assert!(
                (actual - expected).abs() < 1e-6,
                "0.0.1 project settings did not migrate: {actual} versus {expected}"
            );
            if index == 1 {
                let mut display = [0_i8; 32];
                assert!((params.value_to_text.unwrap())(
                    plugin,
                    info.id,
                    actual,
                    display.as_mut_ptr(),
                    display.len() as u32
                ));
                assert_eq!(
                    CStr::from_ptr(display.as_ptr()).to_str().unwrap(),
                    member.to_string()
                );
            }
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
unsafe extern "C" fn accept_event(
    _: *const clap_output_events,
    _: *const clap_event_header,
) -> bool {
    true
}
unsafe extern "C" fn one_event(_: *const clap_input_events) -> u32 {
    1
}
unsafe extern "C" fn get_event(list: *const clap_input_events, _: u32) -> *const clap_event_header {
    unsafe { (*list).ctx.cast() }
}

// All ABI pointers below refer to live stack values or a live loaded library. Plugins are
// stopped, deactivated and destroyed before the entry and library are released.
unsafe fn set_parameter(plugin: *const clap_plugin, index: u32, value: f64) {
    unsafe {
        let params = &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_PARAMS.as_ptr())
            .cast::<clap_plugin_params>();
        let mut info: clap_param_info = mem::zeroed();
        assert!((params.get_info.unwrap())(plugin, index, &mut info));
        let text = CString::new(value.to_string()).unwrap();
        let mut clap_value = 0.0;
        if index == 2 {
            clap_value = value;
        } else {
            assert!((params.text_to_value.unwrap())(
                plugin,
                info.id,
                text.as_ptr(),
                &mut clap_value
            ));
        }
        let mut event = clap_event_param_value {
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
            value: clap_value,
        };
        let input = clap_input_events {
            ctx: (&mut event as *mut clap_event_param_value).cast(),
            size: Some(one_event),
            get: Some(get_event),
        };
        let output = clap_output_events {
            ctx: ptr::null_mut(),
            try_push: Some(accept_event),
        };
        (params.flush.unwrap())(plugin, &input, &output);
        let mut actual = -1.0;
        assert!((params.get_value.unwrap())(plugin, info.id, &mut actual));
        assert_eq!(actual, clap_value);
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn process(
    plugin: *const clap_plugin,
    rate: f64,
    start: i64,
    frames: usize,
    transport_present: bool,
    playing: bool,
    in_place: bool,
    channels: u32,
) {
    unsafe {
        let left: Vec<f32> = (0..frames)
            .map(|n| ((n as f32 * 0.37).sin() * 0.25) - 0.01)
            .collect();
        let right: Vec<f32> = left.iter().map(|x| -*x).collect();
        let mut inputs = [left.clone(), right.clone()];
        let mut outputs = [vec![f32::NAN; frames], vec![f32::NAN; frames]];
        let mut in_channels = [inputs[0].as_mut_ptr(), inputs[1].as_mut_ptr()];
        let mut out_channels = if in_place {
            in_channels
        } else {
            [outputs[0].as_mut_ptr(), outputs[1].as_mut_ptr()]
        };
        let input = clap_audio_buffer {
            data32: in_channels.as_mut_ptr(),
            data64: ptr::null_mut(),
            channel_count: channels,
            latency: 0,
            constant_mask: 0,
        };
        let mut output = clap_audio_buffer {
            data32: out_channels.as_mut_ptr(),
            data64: ptr::null_mut(),
            channel_count: channels,
            latency: 0,
            constant_mask: 0,
        };
        let mut transport: clap_event_transport = mem::zeroed();
        transport.header = clap_event_header {
            size: mem::size_of::<clap_event_transport>() as u32,
            time: 0,
            space_id: CLAP_CORE_EVENT_SPACE_ID,
            type_: CLAP_EVENT_TRANSPORT,
            flags: 0,
        };
        transport.flags = CLAP_TRANSPORT_HAS_SECONDS_TIMELINE
            | if playing {
                CLAP_TRANSPORT_IS_PLAYING
            } else {
                0
            };
        transport.song_pos_seconds =
            (start as f64 / rate * CLAP_SECTIME_FACTOR as f64).round() as i64;
        let events_in = clap_input_events {
            ctx: ptr::null_mut(),
            size: Some(empty),
            get: Some(no_event),
        };
        let events_out = clap_output_events {
            ctx: ptr::null_mut(),
            try_push: Some(accept_event),
        };
        let block = clap_process {
            steady_time: -1,
            frames_count: frames as u32,
            transport: if transport_present {
                &transport
            } else {
                ptr::null()
            },
            audio_inputs: &input,
            audio_outputs: &mut output,
            audio_inputs_count: 1,
            audio_outputs_count: 1,
            in_events: &events_in,
            out_events: &events_out,
        };
        assert_ne!(
            ((*plugin).process.unwrap())(plugin, &block),
            CLAP_PROCESS_ERROR
        );
        let result = if in_place { &inputs } else { &outputs };
        for (expected, actual) in [left, right].iter().zip(result).take(channels as usize) {
            assert!(
                expected
                    .iter()
                    .zip(actual)
                    .all(|(a, b)| a.to_bits() == b.to_bits()),
                "Pass-through changed samples"
            );
        }
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("host_check <absolute plugin.clap path>");
    unsafe {
        let library = libloading::Library::new(&path).expect("Load plugin DLL");
        let entry = library
            .get::<*const clap_plugin_entry>(b"clap_entry\0")
            .expect("CLAP export");
        let entry = &**entry;
        let path_c = CString::new(path).unwrap();
        assert!((entry.init.unwrap())(path_c.as_ptr()));
        let factory = &*(entry.get_factory.unwrap())(CLAP_PLUGIN_FACTORY_ID.as_ptr())
            .cast::<clap_plugin_factory>();
        assert_eq!((factory.get_plugin_count.unwrap())(factory), 1);
        let descriptor = &*(factory.get_plugin_descriptor.unwrap())(factory, 0);
        assert_eq!(
            CStr::from_ptr(descriptor.name).to_str().unwrap(),
            "CleanSlate FL Probe"
        );
        let host = clap_host {
            clap_version: CLAP_VERSION,
            host_data: ptr::null_mut(),
            name: c"CleanSlate synthetic host".as_ptr(),
            vendor: c"CleanSlate".as_ptr(),
            url: c"https://github.com/Today20092/podcast-bleed-reducer".as_ptr(),
            version: c"0.0.1".as_ptr(),
            get_extension: Some(no_extension),
            request_restart: Some(restart),
            request_process: Some(request),
            request_callback: Some(request),
        };
        let plugins: Vec<_> = (0..2)
            .map(|_| {
                let plugin = (factory.create_plugin.unwrap())(factory, &host, descriptor.id);
                assert!(!plugin.is_null());
                assert!(((*plugin).init.unwrap())(plugin));
                plugin
            })
            .collect();
        for (i, &plugin) in plugins.iter().enumerate() {
            let gui = ((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_GUI.as_ptr())
                .cast::<clap_plugin_gui>();
            assert!(!gui.is_null(), "CLAP must expose a custom editor");
            let gui = &*gui;
            assert!((gui.is_api_supported.unwrap())(
                plugin,
                CLAP_WINDOW_API_WIN32.as_ptr(),
                false
            ));
            let (mut width, mut height) = (0, 0);
            assert!((gui.get_size.unwrap())(plugin, &mut width, &mut height));
            assert!(width >= 600 && height >= 500);
            load_legacy_state(plugin, i + 1);
            set_parameter(plugin, 0, 1.0);
            set_parameter(plugin, 1, (i + 1) as f64);
            state_round_trip(plugin);
        }
        let mut checks = 0;
        for (channels, rate) in [(2, 44_100.0), (2, 48_000.0), (1, 44_100.0), (1, 48_000.0)] {
            for offline in [false, true] {
                for &plugin in &plugins {
                    let configs = &*((*plugin).get_extension.unwrap())(
                        plugin,
                        CLAP_EXT_AUDIO_PORTS_CONFIG.as_ptr(),
                    )
                    .cast::<clap_plugin_audio_ports_config>();
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
                    let render =
                        &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_RENDER.as_ptr())
                            .cast::<clap_plugin_render>();
                    assert!((render.set.unwrap())(
                        plugin,
                        if offline {
                            CLAP_RENDER_OFFLINE
                        } else {
                            CLAP_RENDER_REALTIME
                        }
                    ));
                    assert!(((*plugin).activate.unwrap())(plugin, rate, 1, 1024));
                    assert!(((*plugin).start_processing.unwrap())(plugin));
                }
                let mut start = 0;
                // Continuous input must fill the lookback and validate actual shared PCM.
                for _ in 0..80 {
                    for &plugin in &plugins {
                        process(plugin, rate, start, 256, true, true, false, channels);
                        checks += 1;
                    }
                    start += 256;
                }
                for frames in [1, 64, 128, 256, 511, 512, 1024] {
                    for present in [true, false] {
                        for in_place in [true, false] {
                            for &plugin in &plugins {
                                process(
                                    plugin, rate, start, frames, present, true, in_place, channels,
                                );
                                checks += 1;
                            }
                            start += frames as i64;
                        }
                    }
                }
                for start in [0, 8192, 0, -1024] {
                    for &plugin in &plugins {
                        ((*plugin).reset.unwrap())(plugin);
                        process(plugin, rate, start, 256, true, false, false, channels);
                        process(plugin, rate, start, 256, true, true, false, channels);
                        checks += 2;
                    }
                }
                for &plugin in &plugins {
                    ((*plugin).stop_processing.unwrap())(plugin);
                    ((*plugin).deactivate.unwrap())(plugin);
                }
            }
        }
        // A CLAP mode change while active requests a host restart; activation-mode logs
        // cannot update until the host honors that request.
        let restarts_before = RESTARTS.load(Relaxed);
        for &plugin in &plugins {
            let render = &*((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_RENDER.as_ptr())
                .cast::<clap_plugin_render>();
            assert!((render.set.unwrap())(plugin, CLAP_RENDER_REALTIME));
            assert!(((*plugin).activate.unwrap())(plugin, 48000.0, 1, 1024));
            assert!(((*plugin).start_processing.unwrap())(plugin));
            set_parameter(plugin, 2, 1.0);
            assert!((render.set.unwrap())(plugin, CLAP_RENDER_OFFLINE));
            process(plugin, 48000.0, 0, 256, true, false, false, 1);
            checks += 1;
            ((*plugin).stop_processing.unwrap())(plugin);
            ((*plugin).deactivate.unwrap())(plugin);
        }
        assert!(RESTARTS.load(Relaxed) >= restarts_before + 2);
        for &plugin in &plugins {
            ((*plugin).destroy.unwrap())(plugin);
        }
        (entry.deinit.unwrap())();
        let _get_factory = library
            .get::<unsafe extern "system" fn() -> *mut c_void>(b"GetPluginFactory\0")
            .expect("VST3 export");
        let report_dir = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap())
            .join("CleanSlate/PluginProbe");
        let prefix = format!("probe-{}-", std::process::id());
        let (mut ready, mut offline, mut labelled) = (0, 0, 0);
        for file in std::fs::read_dir(report_dir).unwrap() {
            let file = file.unwrap();
            if !file.file_name().to_string_lossy().starts_with(&prefix) {
                continue;
            }
            let text = std::fs::read_to_string(file.path()).unwrap();
            assert!(
                text.lines()
                    .last()
                    .unwrap()
                    .ends_with("queue_dropped=0; rows_omitted=0; io_failed=false")
            );
            for row in text
                .lines()
                .filter(|line| line.as_bytes().first().is_some_and(u8::is_ascii_digit))
            {
                let columns: Vec<_> = row.split(',').collect();
                if columns[19] == "Ready" {
                    ready += 1;
                    assert_eq!(columns[21], "512");
                }
                if columns[16] == "Offline" {
                    offline += 1;
                }
                if columns[17] == "true" {
                    labelled += 1;
                }
                assert_eq!(
                    columns[22], "0",
                    "Synthetic normal scheduling lost audio packets"
                );
            }
        }
        assert!(ready > 0 && offline > 0 && labelled > 0);
        println!(
            "PASS: {checks} bit-exact mono/stereo blocks; {ready} validated shared PCM windows; 0.0.1 saved-state migration and round-trip; offline activation and manual export label; active render-mode change requests host restart. VST3 factory symbol found (not a VST3 processing test)."
        );
    }
}
