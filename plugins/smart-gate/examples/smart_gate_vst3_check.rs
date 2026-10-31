//! Device-free Gate/probe factory, state, bypass and audio checks using the actual VST3 ABI.
#[path = "support/candidate_loader.rs"]
mod candidate_loader;
use std::{
    ffi::c_void,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    mem, ptr,
    sync::{Arc, Mutex},
};
use vst3::{
    Class, ComPtr, ComWrapper, Interface,
    Steinberg::{Vst::*, *},
};

struct StateStream(Arc<Mutex<Cursor<Vec<u8>>>>);
impl Class for StateStream {
    type Interfaces = (IBStream,);
}
impl IBStreamTrait for StateStream {
    unsafe fn read(&self, buffer: *mut c_void, count: i32, read: *mut i32) -> tresult {
        unsafe {
            if !read.is_null() {
                *read = 0;
            }
            if count < 0 || (count > 0 && buffer.is_null()) {
                return kInvalidArgument;
            }
            if count == 0 {
                return kResultOk;
            }
            let bytes = std::slice::from_raw_parts_mut(buffer.cast(), count as usize);
            match self.0.lock().unwrap().read(bytes) {
                Ok(count) => {
                    if !read.is_null() {
                        *read = count as i32;
                    }
                    kResultOk
                }
                Err(_) => kResultFalse,
            }
        }
    }
    unsafe fn write(&self, buffer: *mut c_void, count: i32, written: *mut i32) -> tresult {
        unsafe {
            if !written.is_null() {
                *written = 0;
            }
            if count < 0 || (count > 0 && buffer.is_null()) {
                return kInvalidArgument;
            }
            if count == 0 {
                return kResultOk;
            }
            let bytes = std::slice::from_raw_parts(buffer.cast(), count as usize);
            match self.0.lock().unwrap().write_all(bytes) {
                Ok(()) => {
                    if !written.is_null() {
                        *written = count;
                    }
                    kResultOk
                }
                Err(_) => kResultFalse,
            }
        }
    }
    unsafe fn seek(&self, offset: i64, mode: i32, position: *mut i64) -> tresult {
        let from = match mode {
            0 if offset >= 0 => SeekFrom::Start(offset as u64),
            1 => SeekFrom::Current(offset),
            2 => SeekFrom::End(offset),
            _ => return kInvalidArgument,
        };
        match self.0.lock().unwrap().seek(from) {
            Ok(value) => {
                if !position.is_null() {
                    unsafe {
                        *position = value as i64;
                    }
                }
                kResultOk
            }
            Err(_) => kInvalidArgument,
        }
    }
    unsafe fn tell(&self, position: *mut i64) -> tresult {
        if position.is_null() {
            return kInvalidArgument;
        }
        unsafe {
            *position = self.0.lock().unwrap().position() as i64;
        }
        kResultOk
    }
}

fn platform_id(mut bytes: [u8; 16]) -> TUID {
    if cfg!(target_os = "windows") {
        bytes[..4].reverse();
        bytes[4..6].reverse();
        bytes[6..8].reverse();
    }
    bytes.map(|byte| byte as _)
}

struct Bundle {
    _library: libloading::Library,
    factory: Option<ComPtr<IPluginFactory>>,
    exit: Option<unsafe extern "system" fn() -> bool>,
    id: TUID,
}
impl Bundle {
    unsafe fn load(path: &str, expected: [u8; 16], isolated: bool) -> Self {
        unsafe {
            let library = candidate_loader::load(path, isolated);
            if let Ok(init) = library.get::<unsafe extern "system" fn() -> bool>(b"InitDll\0") {
                assert!(init());
            }
            let factory_fn = library
                .get::<unsafe extern "system" fn() -> *mut IPluginFactory>(b"GetPluginFactory\0")
                .unwrap();
            let factory = ComPtr::from_raw(factory_fn()).unwrap();
            assert_eq!(factory.countClasses(), 1);
            let mut info = mem::zeroed();
            assert_eq!(factory.getClassInfo(0, &mut info), kResultOk);
            assert_eq!(info.cid, platform_id(expected));
            let exit = library
                .get::<unsafe extern "system" fn() -> bool>(b"ExitDll\0")
                .ok()
                .map(|symbol| *symbol);
            Self {
                _library: library,
                factory: Some(factory),
                exit,
                id: info.cid,
            }
        }
    }
    unsafe fn create(&self) -> Instance {
        unsafe {
            let mut raw = ptr::null_mut();
            assert_eq!(
                self.factory.as_ref().unwrap().createInstance(
                    self.id.as_ptr().cast(),
                    IComponent::IID.as_ptr().cast(),
                    &mut raw
                ),
                kResultOk
            );
            let component = ComPtr::<IComponent>::from_raw(raw.cast()).unwrap();
            assert_eq!(component.initialize(ptr::null_mut()), kResultOk);
            let processor = component.cast::<IAudioProcessor>().unwrap();
            let controller = component.cast::<IEditController>().unwrap();
            Instance {
                component,
                processor,
                controller,
            }
        }
    }
}
impl Drop for Bundle {
    fn drop(&mut self) {
        drop(self.factory.take());
        if let Some(exit) = self.exit {
            unsafe {
                assert!(exit());
            }
        }
    }
}
struct Instance {
    component: ComPtr<IComponent>,
    processor: ComPtr<IAudioProcessor>,
    controller: ComPtr<IEditController>,
}
impl Instance {
    unsafe fn bypass(&self, enabled: bool) {
        unsafe {
            assert_eq!(self.controller.getParameterCount(), 1);
            let mut info: ParameterInfo = mem::zeroed();
            assert_eq!(self.controller.getParameterInfo(0, &mut info), kResultOk);
            let end = info.title.iter().position(|&unit| unit == 0).unwrap();
            assert_eq!(String::from_utf16_lossy(&info.title[..end]), "Bypass");
            let normalized = if enabled { 1.0 } else { 0.0 };
            assert_eq!(
                self.controller.setParamNormalized(info.id, normalized),
                kResultOk
            );
            assert_eq!(self.controller.getParamNormalized(info.id), normalized);
        }
    }
    unsafe fn state_round_trip(&self, gate: bool) {
        unsafe {
            if gate {
                self.bypass(true);
            }
            let bytes = Arc::new(Mutex::new(Cursor::new(Vec::new())));
            let object = ComWrapper::new(StateStream(bytes.clone()));
            let stream = object.to_com_ptr::<IBStream>().unwrap();
            assert_eq!(self.component.getState(stream.as_ptr()), kResultOk);
            let saved = bytes.lock().unwrap().get_ref().clone();
            assert!(!saved.is_empty());
            if gate {
                self.bypass(false);
            }
            bytes.lock().unwrap().set_position(0);
            assert_eq!(self.component.setState(stream.as_ptr()), kResultOk);
            let restored = Arc::new(Mutex::new(Cursor::new(Vec::new())));
            let restored_object = ComWrapper::new(StateStream(restored.clone()));
            let restored_stream = restored_object.to_com_ptr::<IBStream>().unwrap();
            assert_eq!(self.component.getState(restored_stream.as_ptr()), kResultOk);
            assert_eq!(*restored.lock().unwrap().get_ref(), saved);
        }
    }
    unsafe fn process(&self, channels: i32, rate: f64, mode: i32) {
        unsafe {
            let mut arrangement = if channels == 1 {
                SpeakerArr::kMono
            } else {
                SpeakerArr::kStereo
            };
            assert_eq!(
                self.processor
                    .setBusArrangements(&mut arrangement, 1, &mut arrangement, 1),
                kResultOk
            );
            for direction in [BusDirections_::kInput, BusDirections_::kOutput] {
                assert_eq!(
                    self.component
                        .activateBus(MediaTypes_::kAudio, direction, 0, 1),
                    kResultOk
                );
            }
            let mut setup = ProcessSetup {
                processMode: mode,
                symbolicSampleSize: SymbolicSampleSizes_::kSample32,
                maxSamplesPerBlock: 1024,
                sampleRate: rate,
            };
            assert_eq!(self.processor.setupProcessing(&mut setup), kResultOk);
            assert_eq!(self.component.setActive(1), kResultOk);
            assert_eq!(self.processor.getLatencySamples(), 0);
            assert_eq!(self.processor.setProcessing(1), kResultOk);
            for frames in [1, 17, 256, 1024] {
                for in_place in [false, true] {
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
                    let mut ins: Vec<_> = input.iter_mut().map(|c| c.as_mut_ptr()).collect();
                    let mut outs: Vec<_> = if in_place {
                        ins.clone()
                    } else {
                        output.iter_mut().map(|c| c.as_mut_ptr()).collect()
                    };
                    let mut input_bus = AudioBusBuffers {
                        numChannels: channels,
                        silenceFlags: 0,
                        __field0: AudioBusBuffers__type0 {
                            channelBuffers32: ins.as_mut_ptr(),
                        },
                    };
                    let mut output_bus = AudioBusBuffers {
                        numChannels: channels,
                        silenceFlags: 0,
                        __field0: AudioBusBuffers__type0 {
                            channelBuffers32: outs.as_mut_ptr(),
                        },
                    };
                    let mut data = ProcessData {
                        processMode: mode,
                        symbolicSampleSize: SymbolicSampleSizes_::kSample32,
                        numSamples: frames as i32,
                        numInputs: 1,
                        numOutputs: 1,
                        inputs: &mut input_bus,
                        outputs: &mut output_bus,
                        inputParameterChanges: ptr::null_mut(),
                        outputParameterChanges: ptr::null_mut(),
                        inputEvents: ptr::null_mut(),
                        outputEvents: ptr::null_mut(),
                        processContext: ptr::null_mut(),
                    };
                    assert_eq!(self.processor.process(&mut data), kResultOk);
                    let actual = if in_place { input } else { output };
                    let actual: Vec<Vec<u32>> = actual
                        .iter()
                        .map(|c| c.iter().map(|x| x.to_bits()).collect())
                        .collect();
                    assert_eq!(actual, expected);
                }
            }
            assert_eq!(self.processor.getTailSamples(), 0);
            assert_eq!(self.processor.setProcessing(0), kResultOk);
            assert_eq!(self.component.setActive(0), kResultOk);
        }
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(self.component.terminate(), kResultOk);
        }
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
        "smart_gate_vst3_check <Gate binary> <preserved probe binary>"
    );
    // COM objects are released before factory/ExitDll and library unloading; streams and audio storage outlive every ABI call.
    unsafe {
        let gate = Bundle::load(&paths[0], *b"CleanSlateGate01", isolated);
        let probe = (!isolated).then(|| Bundle::load(&paths[1], *b"CleanSlateProbe1", false));
        if let Some(probe) = &probe {
            assert_ne!(gate.id, probe.id);
            let old = probe.create();
            old.state_round_trip(false);
        }
        let instance = gate.create();
        instance.state_round_trip(true);
        {
            let view =
                ComPtr::<IPlugView>::from_raw(instance.controller.createView(c"editor".as_ptr()))
                    .unwrap();
            let mut rect = mem::zeroed();
            assert_eq!(view.getSize(&mut rect), kResultOk);
            assert!(rect.right - rect.left >= 640 && rect.bottom - rect.top >= 440);
        }
        for channels in [1, 2] {
            for rate in [44100.0, 48000.0, 96000.0] {
                for mode in [ProcessModes_::kRealtime, ProcessModes_::kOffline] {
                    for bypass in [false, true] {
                        instance.bypass(bypass);
                        instance.process(channels, rate, mode);
                    }
                }
            }
        }
        if isolated {
            candidate_loader::verify(&paths[0]);
        }
    }
    if isolated {
        println!(
            "PASS: isolated candidate-local DirectML load and VST3 activation; no arming or inference qualification."
        );
        return;
    }
    println!(
        "PASS: Gate/probe VST3 IDs coexist; independent state round trips; Gate bypass, mono/stereo, 44.1/48/96 kHz, realtime/offline, exact unity and zero latency; editor contract only."
    );
}
