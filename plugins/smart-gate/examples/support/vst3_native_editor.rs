//! Optional actual-editor/ABI exercise. No private plugin state or consent hook.
use super::{
    Instance,
    native_editor::{self, Action, Window},
    process_audio,
};
use std::{
    mem, ptr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use vst3::{
    Class, ComPtr, ComRef, ComWrapper,
    Steinberg::{Vst::*, *},
};

struct Frame(usize);
impl Class for Frame {
    type Interfaces = (IPlugFrame,);
}
impl IPlugFrameTrait for Frame {
    unsafe fn resizeView(&self, view: *mut IPlugView, size: *mut ViewRect) -> tresult {
        if size.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: the plugin supplies the view/rectangle for this synchronous callback.
        unsafe {
            let Some(view) = ComRef::from_raw(view) else {
                return kInvalidArgument;
            };
            if native_editor::resize(
                self.0,
                (*size).right - (*size).left,
                (*size).bottom - (*size).top,
            ) {
                view.onSize(size)
            } else {
                kResultFalse
            }
        }
    }
}

const PENDING: u64 = 1 << 63;
struct Handler {
    ids: [ParamID; 2],
    values: [Arc<AtomicU64>; 2],
}
impl Class for Handler {
    type Interfaces = (IComponentHandler,);
}
impl IComponentHandlerTrait for Handler {
    unsafe fn beginEdit(&self, id: ParamID) -> tresult {
        if self.ids.contains(&id) {
            kResultOk
        } else {
            kInvalidArgument
        }
    }
    unsafe fn performEdit(&self, id: ParamID, value: f64) -> tresult {
        let Some(index) = self.ids.iter().position(|candidate| *candidate == id) else {
            return kInvalidArgument;
        };
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return kInvalidArgument;
        }
        self.values[index].store(value.to_bits() | PENDING, Ordering::Release);
        kResultOk
    }
    unsafe fn endEdit(&self, id: ParamID) -> tresult {
        if self.ids.contains(&id) {
            kResultOk
        } else {
            kInvalidArgument
        }
    }
    unsafe fn restartComponent(&self, _: i32) -> tresult {
        kNotImplemented
    }
}
struct Point {
    id: ParamID,
    value: f64,
}
impl Class for Point {
    type Interfaces = (IParamValueQueue,);
}
impl IParamValueQueueTrait for Point {
    unsafe fn getParameterId(&self) -> ParamID {
        self.id
    }
    unsafe fn getPointCount(&self) -> i32 {
        1
    }
    unsafe fn getPoint(&self, index: i32, offset: *mut i32, value: *mut f64) -> tresult {
        if index != 0 || offset.is_null() || value.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: validated caller-owned output pointers are written synchronously.
        unsafe {
            *offset = 0;
            *value = self.value;
        }
        kResultOk
    }
    unsafe fn addPoint(&self, _: i32, _: f64, _: *mut i32) -> tresult {
        kNotImplemented
    }
}
struct Changes(Vec<ComPtr<IParamValueQueue>>);
impl Class for Changes {
    type Interfaces = (IParameterChanges,);
}
impl IParameterChangesTrait for Changes {
    unsafe fn getParameterCount(&self) -> i32 {
        self.0.len() as i32
    }
    unsafe fn getParameterData(&self, index: i32) -> *mut IParamValueQueue {
        self.0
            .get(index as usize)
            .map_or(ptr::null_mut(), ComPtr::as_ptr)
    }
    unsafe fn addParameterData(&self, _: *const ParamID, _: *mut i32) -> *mut IParamValueQueue {
        ptr::null_mut()
    }
}

#[derive(Debug)]
struct Report {
    frames: usize,
    min_gain: f32,
    max_gain: f32,
    last_gain: f32,
    latency: u32,
    tail: u32,
}
fn render(
    processor: &ComPtr<IAudioProcessor>,
    ids: [ParamID; 2],
    pending: &[Arc<AtomicU64>; 2],
    armed: bool,
) -> Result<Report, String> {
    const REFERENCE: f32 = 1.0 / 8192.0;
    let mut report = Report {
        frames: 0,
        min_gain: 1.0,
        max_gain: 0.0,
        last_gain: 1.0,
        latency: 0,
        tail: 0,
    };
    for start in (0..36_000).step_by(256) {
        let frames = (36_000 - start).min(256);
        let markers: Vec<_> = (start..start + frames)
            .map(|i| {
                if i % 257 == 0 {
                    REFERENCE / 2.0
                } else if i % 251 == 0 {
                    -REFERENCE / 4.0
                } else {
                    0.0
                }
            })
            .collect();
        let mut input = vec![vec![REFERENCE; frames], markers.clone()];
        let mut output = vec![vec![f32::NAN; frames]; 2];
        let points: Vec<_> = ids
            .iter()
            .zip(pending)
            .filter_map(|(&id, value)| {
                let edit = value.swap(0, Ordering::AcqRel);
                (edit & PENDING != 0).then(|| {
                    ComWrapper::new(Point {
                        id,
                        value: f64::from_bits(edit & !PENDING),
                    })
                    .to_com_ptr::<IParamValueQueue>()
                    .unwrap()
                })
            })
            .collect();
        let changes = if !points.is_empty() {
            Some(
                ComWrapper::new(Changes(points))
                    .to_com_ptr::<IParameterChanges>()
                    .unwrap(),
            )
        } else {
            None
        };
        // SAFETY: the processor and all process/parameter buffers remain live through the call.
        let result = unsafe {
            process_audio(
                processor,
                &mut input,
                &mut output,
                ProcessModes_::kOffline,
                false,
                changes.as_ref().map_or(ptr::null_mut(), ComPtr::as_ptr),
            )
        };
        if result != kResultOk {
            return Err(format!("VST3 process returned {result}"));
        }
        for (frame, marker) in markers.iter().enumerate() {
            let gain = output[0][frame] / REFERENCE;
            if !gain.is_finite()
                || !(0.50..=1.0).contains(&gain)
                || output[1][frame].to_bits() != (marker * gain).to_bits()
            {
                return Err("Nonfinite/out-of-range gain or shifted stereo marker".into());
            }
            if !armed
                && (output[0][frame].to_bits() != REFERENCE.to_bits()
                    || output[1][frame].to_bits() != marker.to_bits())
            {
                return Err("Unarmed baseline changed input".into());
            }
            report.min_gain = report.min_gain.min(gain);
            report.max_gain = report.max_gain.max(gain);
            report.last_gain = gain;
        }
        report.frames += frames;
    }
    // SAFETY: the active processor stays owned by the worker throughout this render.
    unsafe {
        report.latency = processor.getLatencySamples();
        report.tail = processor.getTailSamples();
    }
    if report.latency != 0 || report.tail != 0 {
        return Err(format!("Unexpected latency/tail: {report:?}"));
    }
    if armed && report.last_gain >= 0.6 {
        return Err(format!(
            "No qualified attenuation; arm the real editor and disable bypass: {report:?}"
        ));
    }
    Ok(report)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    ConsentCheck,
    GuiPreview,
}

pub fn run(instance: &Instance, mode: Mode) -> Result<(), String> {
    let preview = mode == Mode::GuiPreview;
    let pending = [Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0))];
    // SAFETY: ParameterInfo is the SDK's plain integer/fixed-array output record.
    let mut info: ParameterInfo = unsafe { mem::zeroed() };
    // SAFETY: ParameterInfo is a plain SDK output record, filled below.
    let mut range_info: ParameterInfo = unsafe { mem::zeroed() };
    // SAFETY: these main-thread ABI calls configure this retained instance before processing starts.
    unsafe {
        if instance.controller.getParameterCount() != 2
            || instance.controller.getParameterInfo(0, &mut info) != kResultOk
            || instance.controller.getParameterInfo(1, &mut range_info) != kResultOk
        {
            return Err("Expected the existing bypass parameter".into());
        }
        if preview {
            if instance.controller.getParamNormalized(info.id) != 1.0 {
                return Err("GUI preview requires the plugin's default bypass on".into());
            }
        } else {
            instance.bypass(false);
        }
    }
    let handler = ComWrapper::new(Handler {
        ids: [info.id, range_info.id],
        values: pending.clone(),
    })
    .to_com_ptr::<IComponentHandler>()
    .unwrap();
    let (commands, requests) = mpsc::channel::<bool>();
    let (reports, results) = mpsc::channel();
    let mut worker = None;
    let mut window = None;
    let mut view: Option<ComPtr<IPlugView>> = None;
    let mut frame: Option<ComPtr<IPlugFrame>> = None;
    let mut attached = false;
    let outcome: Result<(), String> = (|| {
        // SAFETY: setup uses valid stereo buses and a retained component handler.
        unsafe {
            if instance.controller.setComponentHandler(handler.as_ptr()) != kResultOk {
                return Err("Cannot install GUI parameter handler".into());
            }
            let mut arrangement = SpeakerArr::kStereo;
            if instance
                .processor
                .setBusArrangements(&mut arrangement, 1, &mut arrangement, 1)
                != kResultOk
            {
                return Err("Cannot configure stereo".into());
            }
            for direction in [BusDirections_::kInput, BusDirections_::kOutput] {
                if instance
                    .component
                    .activateBus(MediaTypes_::kAudio, direction, 0, 1)
                    != kResultOk
                {
                    return Err("Cannot activate audio bus".into());
                }
            }
            let mut setup = ProcessSetup {
                processMode: ProcessModes_::kOffline,
                symbolicSampleSize: SymbolicSampleSizes_::kSample32,
                maxSamplesPerBlock: 256,
                sampleRate: 48_000.0,
            };
            if instance.processor.setupProcessing(&mut setup) != kResultOk
                || instance.component.setActive(1) != kResultOk
            {
                return Err("Cannot prepare Gate".into());
            }
        }
        if !preview {
            let processor = instance.processor.clone();
            let pending = pending.clone();
            worker = Some(std::thread::spawn(move || {
                // SAFETY: this sole processing thread owns all process/start/stop calls.
                unsafe {
                    if processor.setProcessing(1) != kResultOk {
                        let _ = reports.send(Err("Cannot start processing".into()));
                        return;
                    }
                }
                let _ = reports.send(render(
                    &processor,
                    [info.id, range_info.id],
                    &pending,
                    false,
                ));
                while let Ok(run) = requests.recv() {
                    if !run {
                        break;
                    }
                    let _ =
                        reports.send(render(&processor, [info.id, range_info.id], &pending, true));
                }
                // SAFETY: no process call remains in flight on this sole processing thread.
                unsafe {
                    assert_eq!(processor.setProcessing(0), kResultOk);
                }
            }));
            let baseline = results
                .recv_timeout(Duration::from_secs(10))
                .map_err(|error| error.to_string())??;
            println!("NATIVE_UNARMED_BASELINE: {baseline:?}");
        }
        window = Some(Window::new(!preview)?);
        let parent = window.as_ref().unwrap();
        // SAFETY: the real editor, host frame and parent survive until explicit detach below.
        unsafe {
            view = ComPtr::from_raw(instance.controller.createView(c"editor".as_ptr()));
            let editor = view.as_ref().ok_or("Cannot create Gate editor")?;
            frame = ComWrapper::new(Frame(parent.handle() as usize)).to_com_ptr();
            if editor.isPlatformTypeSupported(c"HWND".as_ptr()) != kResultOk
                || editor.setFrame(frame.as_ref().unwrap().as_ptr()) != kResultOk
                || editor.attached(parent.handle(), c"HWND".as_ptr()) != kResultOk
            {
                return Err("Cannot attach real Gate editor".into());
            }
            let mut size = ViewRect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if editor.getSize(&mut size) != kResultOk
                || !native_editor::resize(
                    parent.handle() as usize,
                    size.right - size.left,
                    size.bottom - size.top,
                )
                || editor.onSize(&mut size) != kResultOk
            {
                return Err("Cannot size the host to the real editor".into());
            }
        }
        attached = true;
        if preview {
            println!(
                "GUI_PREVIEW_READY: default bypass on, unarmed; no audio processing/device. Close this window when finished."
            );
        } else {
            println!(
                "NATIVE_EDITOR_READY: processing started; click the real Arm button, then Run armed ABI check. No audio device is opened."
            );
        }
        let deadline = Instant::now() + Duration::from_secs(120);
        let mut busy = false;
        let mut passed = false;
        while preview || Instant::now() < deadline {
            match parent.pump() {
                Some(Action::Close) => break,
                Some(Action::Run) if !preview && !busy => {
                    commands.send(true).map_err(|error| error.to_string())?;
                    busy = true;
                }
                _ => {}
            }
            if !preview {
                match results.try_recv() {
                    Ok(Ok(report)) => {
                        println!("NATIVE_ARMED_OUTPUT: {report:?}");
                        passed = true;
                        busy = false;
                    }
                    Ok(Err(error)) => {
                        eprintln!("NATIVE_CHECK_UNQUALIFIED: {error}");
                        busy = false;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        return Err("Processing worker ended unexpectedly".into());
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                }
            }
            std::thread::sleep(Duration::from_millis(8));
        }
        if preview || passed {
            Ok(())
        } else {
            Err("No qualified real-editor armed render completed before close/timeout".into())
        }
    })();
    let _ = commands.send(false);
    let mut teardown_ok = true;
    if let Some(worker) = worker
        && worker.join().is_err()
    {
        eprintln!("Native processing worker panicked");
        teardown_ok = false;
    }
    // SAFETY: processing has stopped; detach/release UI before destroying its parent and component.
    unsafe {
        if let Some(editor) = &view {
            if attached && editor.removed() != kResultOk {
                teardown_ok = false;
            }
            if editor.setFrame(ptr::null_mut()) != kResultOk {
                teardown_ok = false;
            }
        }
        drop(view);
        drop(frame);
        drop(window);
        if instance.controller.setComponentHandler(ptr::null_mut()) != kResultOk {
            teardown_ok = false;
        }
        if instance.component.setActive(0) != kResultOk {
            teardown_ok = false;
        }
    }
    outcome?;
    if !teardown_ok {
        return Err("Native editor/audio teardown failed".into());
    }
    Ok(())
}
