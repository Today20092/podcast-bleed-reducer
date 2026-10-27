//! Small device-free VST3 host; no editor, audio device or DAW is opened.
use std::{
    cell::{Cell, RefCell},
    mem, ptr,
};
use vst3::{
    ComPtr, Interface,
    Steinberg::{Vst::*, *},
};
#[cfg(cleanslate_acceptance_driver)]
#[path = "support/podcast_acceptance.rs"]
mod podcast_acceptance;
#[cfg(cleanslate_acceptance_driver)]
#[path = "support/vst3_state.rs"]
mod vst3_state;

struct Instance {
    component: ComPtr<IComponent>,
    processor: ComPtr<IAudioProcessor>,
    controller: ComPtr<IEditController>,
    latency: Cell<usize>,
    reference: RefCell<Vec<[f32; 2]>>,
    cursor: Cell<usize>,
    expected_position: Cell<Option<i64>>,
}

impl Instance {
    unsafe fn new(factory: &ComPtr<IPluginFactory>, cid: &TUID, member: usize) -> Self {
        unsafe {
            let mut raw = ptr::null_mut();
            assert_eq!(
                factory.createInstance(
                    cid.as_ptr().cast(),
                    IComponent::IID.as_ptr().cast(),
                    &mut raw
                ),
                kResultOk
            );
            let component = ComPtr::<IComponent>::from_raw(raw.cast()).unwrap();
            assert_eq!(component.initialize(ptr::null_mut()), kResultOk);
            let processor = component.cast::<IAudioProcessor>().unwrap();
            let controller = component.cast::<IEditController>().unwrap();
            let this = Self {
                component,
                processor,
                controller,
                latency: Cell::new(0),
                reference: RefCell::new(Vec::new()),
                cursor: Cell::new(0),
                expected_position: Cell::new(None),
            };
            let mut found = 0;
            for index in 0..this.controller.getParameterCount() {
                let mut info: ParameterInfo = mem::zeroed();
                assert_eq!(
                    this.controller.getParameterInfo(index, &mut info),
                    kResultOk
                );
                let count = info
                    .title
                    .iter()
                    .position(|&x| x == 0)
                    .unwrap_or(info.title.len());
                let title = String::from_utf16_lossy(&info.title[..count]);
                let value = match title.as_str() {
                    "Session group (0 = off)" => "1".to_string(),
                    "Microphone member" => member.to_string(),
                    "Label this test as Export (manual)" => "Off".to_string(),
                    _ => continue,
                };
                let mut text: String128 = [0; 128];
                for (target, unit) in text.iter_mut().zip(value.encode_utf16()) {
                    *target = unit;
                }
                let mut normalized = -1.0;
                assert_eq!(
                    this.controller.getParamValueByString(
                        info.id,
                        text.as_mut_ptr(),
                        &mut normalized
                    ),
                    kResultOk
                );
                assert_eq!(
                    this.controller.setParamNormalized(info.id, normalized),
                    kResultOk
                );
                assert_eq!(this.controller.getParamNormalized(info.id), normalized);
                found += 1;
            }
            assert_eq!(found, 3, "Required VST3 controls are not exposed");
            // Construct/release the editor contract without attaching a native window.
            let view =
                ComPtr::<IPlugView>::from_raw(this.controller.createView(c"editor".as_ptr()))
                    .expect("The listening build must expose an editor");
            let mut rect: ViewRect = mem::zeroed();
            assert_eq!(view.getSize(&mut rect), kResultOk);
            assert!(rect.right > rect.left && rect.bottom > rect.top);
            drop(view);
            this
        }
    }

    unsafe fn activate(&self, channels: i32, rate: f64, mode: i32) {
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
            assert_eq!(
                self.component
                    .activateBus(MediaTypes_::kAudio, BusDirections_::kInput, 0, 1),
                kResultOk
            );
            assert_eq!(
                self.component
                    .activateBus(MediaTypes_::kAudio, BusDirections_::kOutput, 0, 1),
                kResultOk
            );
            let mut setup = ProcessSetup {
                processMode: mode,
                symbolicSampleSize: SymbolicSampleSizes_::kSample32,
                maxSamplesPerBlock: 1024,
                sampleRate: rate,
            };
            assert_eq!(self.processor.setupProcessing(&mut setup), kResultOk);
            assert_eq!(self.component.setActive(1), kResultOk);
            assert_eq!(self.processor.setProcessing(1), kResultOk);
            // Session delay is fixed even when its processing switch is off.
            let latency = (rate * 416.0 / 1000.0) as usize + 1024;
            self.latency.set(latency);
            *self.reference.borrow_mut() = vec![[0.0; 2]; latency];
            self.cursor.set(0);
            self.expected_position.set(None);
            assert_eq!(self.processor.getLatencySamples(), latency as u32);
        }
    }

    unsafe fn enable_uncalibrated(&self) {
        unsafe {
            let mut found = false;
            for index in 0..self.controller.getParameterCount() {
                let mut info: ParameterInfo = mem::zeroed();
                assert_eq!(
                    self.controller.getParameterInfo(index, &mut info),
                    kResultOk
                );
                let count = info
                    .title
                    .iter()
                    .position(|&x| x == 0)
                    .unwrap_or(info.title.len());
                if String::from_utf16_lossy(&info.title[..count]) == "Enable processing" {
                    assert_eq!(self.controller.setParamNormalized(info.id, 1.0), kResultOk);
                    found = true;
                }
            }
            assert!(found);
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn process(
        &self,
        start: i64,
        frames: usize,
        channels: i32,
        rate: f64,
        mode: i32,
        in_place: bool,
        timing: bool,
    ) {
        unsafe {
            self.process_audio(
                start, frames, channels, rate, mode, in_place, timing, None, true,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn process_audio(
        &self,
        start: i64,
        frames: usize,
        channels: i32,
        rate: f64,
        mode: i32,
        in_place: bool,
        timing: bool,
        source: Option<&[f32]>,
        check: bool,
    ) -> Vec<f32> {
        unsafe {
            let expected: Vec<_> = source.map(<[f32]>::to_vec).unwrap_or_else(|| {
                (0..frames)
                    .map(|i| (i as f32 * 0.31).sin() * 0.125)
                    .collect()
            });
            let mut input = [expected.clone(), expected.iter().map(|x| -*x).collect()];
            let mut output = [vec![f32::NAN; frames], vec![f32::NAN; frames]];
            let mut input_pointers = [input[0].as_mut_ptr(), input[1].as_mut_ptr()];
            let mut output_pointers = if in_place {
                input_pointers
            } else {
                [output[0].as_mut_ptr(), output[1].as_mut_ptr()]
            };
            let mut input_bus = AudioBusBuffers {
                numChannels: channels,
                silenceFlags: 0,
                __field0: AudioBusBuffers__type0 {
                    channelBuffers32: input_pointers.as_mut_ptr(),
                },
            };
            let mut output_bus = AudioBusBuffers {
                numChannels: channels,
                silenceFlags: 0,
                __field0: AudioBusBuffers__type0 {
                    channelBuffers32: output_pointers.as_mut_ptr(),
                },
            };
            let mut context: ProcessContext = mem::zeroed();
            // Independent FIFO oracle: variable block lengths never change the delayed source.
            let reference = {
                let mut queue = self.reference.borrow_mut();
                let position = timing.then_some(start);
                if position.is_some()
                    && self.expected_position.get().is_some()
                    && position != self.expected_position.get()
                {
                    queue.fill([0.0; 2]);
                    self.cursor.set(0);
                }
                let result = expected
                    .iter()
                    .map(|&sample| {
                        if queue.is_empty() {
                            return [sample, -sample];
                        }
                        let index = self.cursor.get();
                        let old = queue[index];
                        queue[index] = [sample, -sample];
                        self.cursor.set((index + 1) % queue.len());
                        old
                    })
                    .collect::<Vec<_>>();
                self.expected_position
                    .set(position.map(|p| p + frames as i64));
                result
            };
            context.state = ProcessContext_::StatesAndFlags_::kPlaying as u32;
            context.sampleRate = rate;
            context.projectTimeSamples = start;
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
                processContext: if timing {
                    &mut context
                } else {
                    ptr::null_mut()
                },
            };
            assert_eq!(self.processor.process(&mut data), kResultOk);
            if self.latency.get() > 0 {
                assert_eq!(self.processor.getTailSamples(), self.latency.get() as u32);
            }
            let actual = if in_place { &input } else { &output };
            if check {
                for (channel, samples) in actual.iter().enumerate().take(channels as usize) {
                    let mismatch = samples.iter().enumerate().find_map(|(offset, actual)| {
                        let expected = reference[offset][channel];
                        (actual.to_bits() != expected.to_bits())
                            .then_some((offset, *actual, expected))
                    });
                    assert!(
                        mismatch.is_none(),
                        "VST3 pass-through changed sample bits: start {start}, frames {frames}, channel {channel}, rate {rate}, mode {mode}, delay {}, mismatch {mismatch:?}",
                        self.latency.get()
                    );
                }
            }
            actual[0].clone()
        }
    }

    unsafe fn deactivate(&self) {
        unsafe {
            assert_eq!(self.processor.setProcessing(0), kResultOk);
            assert_eq!(self.component.setActive(0), kResultOk);
        }
    }
}

#[cfg(cleanslate_acceptance_driver)]
unsafe fn session_acceptance(
    library: &libloading::Library,
    factory: &ComPtr<IPluginFactory>,
    cid: &TUID,
) {
    unsafe {
        let driver = *library
            .get::<podcast_acceptance::Driver>(b"cleanslate_podcast_acceptance\0")
            .expect(
                "Acceptance mode requires the debug-only driver; normal candidates must reject it",
            );
        let instances = [
            Instance::new(factory, cid, 1),
            Instance::new(factory, cid, 2),
        ];
        for instance in &instances {
            instance.enable_uncalibrated();
        }
        let active = Cell::new(false);
        let process_mode = Cell::new(ProcessModes_::kRealtime);
        podcast_acceptance::run(
            driver,
            |offline, listen| {
                process_mode.set(if offline {
                    ProcessModes_::kOffline
                } else {
                    ProcessModes_::kRealtime
                });
                for instance in &instances {
                    if active.get() {
                        instance.deactivate();
                    }
                    for index in 0..instance.controller.getParameterCount() {
                        let mut info: ParameterInfo = mem::zeroed();
                        assert_eq!(
                            instance.controller.getParameterInfo(index, &mut info),
                            kResultOk
                        );
                        let end = info
                            .title
                            .iter()
                            .position(|&value| value == 0)
                            .unwrap_or(info.title.len());
                        if String::from_utf16_lossy(&info.title[..end]) == "Listen" {
                            assert_eq!(
                                instance
                                    .controller
                                    .setParamNormalized(info.id, f64::from(listen) / 2.0),
                                kResultOk
                            );
                        }
                    }
                    instance.activate(1, f64::from(podcast_acceptance::RATE), process_mode.get());
                }
                active.set(true);
            },
            || {},
            |member, start, samples| {
                instances[member].process_audio(
                    start,
                    samples.len(),
                    1,
                    f64::from(podcast_acceptance::RATE),
                    process_mode.get(),
                    false,
                    true,
                    Some(samples),
                    false,
                )
            },
            (f64::from(podcast_acceptance::RATE) * 0.416) as usize + 1024,
        );
        for instance in &instances {
            instance.deactivate();
        }
        let states = instances
            .iter()
            .map(|instance| vst3_state::save(&instance.component))
            .collect::<Vec<_>>();
        drop(instances);
        let restored = [
            Instance::new(factory, cid, 1),
            Instance::new(factory, cid, 2),
        ];
        for (instance, bytes) in restored.iter().zip(states) {
            vst3_state::load(&instance.component, bytes);
            instance.activate(
                1,
                f64::from(podcast_acceptance::RATE),
                ProcessModes_::kRealtime,
            );
        }
        for block in 0..40 {
            for instance in &restored {
                instance.process(
                    block * 1024,
                    1024,
                    1,
                    f64::from(podcast_acceptance::RATE),
                    ProcessModes_::kRealtime,
                    false,
                    true,
                );
            }
        }
        assert_eq!(
            driver(0, 0, 0, 0),
            0,
            "restored VST3 instances must bind and flush captured PCM"
        );
        assert_eq!(
            podcast_acceptance::query(driver, 5),
            4,
            "actual VST3 state retains shared examples"
        );
        assert_eq!(
            podcast_acceptance::query(driver, 3),
            0,
            "restored sources remain original until explicit analysis"
        );
        for instance in &restored {
            instance.deactivate();
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let _ = self.component.terminate();
        }
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("vst3_host_check <absolute bundle binary path>");
    // All buffers/COM references remain live for their ABI calls. Instances and factory are
    // released before ExitDll and library unloading. No background plugin thread outlives them.
    unsafe {
        let library = libloading::Library::new(path).expect("Load VST3 DLL");
        #[cfg(all(cleanslate_podcast, not(cleanslate_acceptance_driver)))]
        assert!(
            library
                .get::<unsafe extern "C" fn(u32, i64, i64, u32) -> i32>(
                    b"cleanslate_podcast_acceptance\0"
                )
                .is_err(),
            "Normal artifacts must omit the acceptance driver"
        );
        if let Ok(init) = library.get::<unsafe extern "system" fn() -> bool>(b"InitDll\0") {
            assert!(init());
        }
        let get_factory = library
            .get::<unsafe extern "system" fn() -> *mut IPluginFactory>(b"GetPluginFactory\0")
            .unwrap();
        let factory = ComPtr::from_raw(get_factory()).unwrap();
        assert!(factory.countClasses() > 0);
        let mut info: PClassInfo = mem::zeroed();
        assert_eq!(factory.getClassInfo(0, &mut info), kResultOk);
        #[cfg(cleanslate_podcast)]
        {
            assert_eq!(
                std::ffi::CStr::from_ptr(info.name.as_ptr())
                    .to_str()
                    .unwrap(),
                "CleanSlate Podcast"
            );
            let mut expected = *b"CleanSlatePodc01";
            if cfg!(target_os = "windows") {
                expected[..4].reverse();
                expected[4..6].reverse();
                expected[6..8].reverse();
            }
            assert_eq!(info.cid.map(|byte| byte as u8), expected);
        }
        let instances = [
            Instance::new(&factory, &info.cid, 1),
            Instance::new(&factory, &info.cid, 2),
        ];
        let mut checks = 0;
        for channels in [1, 2] {
            for rate in [44100.0, 48000.0] {
                for mode in [ProcessModes_::kRealtime, ProcessModes_::kOffline] {
                    for instance in &instances {
                        instance.activate(channels, rate, mode);
                    }
                    let mut start = 0;
                    for _ in 0..80 {
                        for instance in &instances {
                            instance.process(start, 256, channels, rate, mode, false, true);
                            checks += 1;
                        }
                        start += 256;
                    }
                    for frames in [1, 64, 215, 512, 1024] {
                        for in_place in [false, true] {
                            for timing in [false, true] {
                                for instance in &instances {
                                    instance.process(
                                        start, frames, channels, rate, mode, in_place, timing,
                                    );
                                    checks += 1;
                                }
                                start += frames as i64;
                            }
                        }
                    }
                    for start in [0, 8192, -1024] {
                        for instance in &instances {
                            instance.process(start, 256, channels, rate, mode, false, true);
                            checks += 1;
                        }
                    }
                    for instance in &instances {
                        instance.deactivate();
                    }
                }
            }
        }
        // An enabled instance with no calibration must preserve its delayed original exactly.
        for instance in &instances {
            instance.enable_uncalibrated();
        }
        for channels in [1, 2] {
            for rate in [44100.0, 48000.0] {
                for mode in [ProcessModes_::kRealtime, ProcessModes_::kOffline] {
                    for instance in &instances {
                        instance.activate(channels, rate, mode);
                    }
                    for block in 0..150 {
                        for instance in &instances {
                            instance.process(
                                block * 256,
                                256,
                                channels,
                                rate,
                                mode,
                                block % 2 == 0,
                                true,
                            );
                            checks += 1;
                        }
                    }
                    for instance in &instances {
                        instance.deactivate();
                    }
                }
            }
        }
        drop(instances);
        #[cfg(cleanslate_acceptance_driver)]
        session_acceptance(&library, &factory, &info.cid);
        drop(factory);
        let report_dir = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap()).join(
            if cfg!(cleanslate_podcast) {
                "CleanSlate/Podcast/Reports"
            } else {
                "CleanSlate/PluginProbe"
            },
        );
        let prefix = format!("probe-{}-", std::process::id());
        let mut ready = 0;
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
            for line in text
                .lines()
                .filter(|line| line.as_bytes().first().is_some_and(u8::is_ascii_digit))
            {
                let columns: Vec<_> = line.split(',').collect();
                assert_eq!(columns[15], "VST3");
                assert_eq!(columns[22], "0");
                if columns[19] == "Ready" {
                    ready += 1;
                    assert_eq!(columns[21], "512");
                }
            }
        }
        assert!(ready > 0, "VST3 PCM sharing was never ready");
        if let Ok(exit) = library.get::<unsafe extern "system" fn() -> bool>(b"ExitDll\0") {
            assert!(exit());
        }
        println!(
            "PASS: VST3 two actual instances; legacy controls discovered/set; mono/stereo, 44.1/48 kHz, realtime/offline; {checks} bit-exact blocks; {ready} shared PCM windows; in-place/separate buffers, missing transport and seeks. Editor constructed/released without opening a window."
        );
    }
}
