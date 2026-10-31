//! C boundary for the JUCE editor. The gain policy and detector stay in this crate.
use crate::engine::Engine;
use crate::history::History;
use nice_plug::prelude::{Buffer, ProcessMode};

pub struct Session {
    engine: Engine,
    history: History,
    channels: Vec<Vec<f32>>,
}

#[repr(C)]
pub struct GateSettings {
    pub bypass: i32,
    pub range_db: f32,
    pub open_threshold: f32,
    pub hold_ms: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub level_bias: f32,
}

#[unsafe(no_mangle)]
pub extern "C" fn cleanslate_gate_create(rate: u32, channels: u32) -> *mut Session {
    let Ok(engine) = Engine::prepare(rate, channels.clamp(1, 2), ProcessMode::Realtime) else {
        return std::ptr::null_mut();
    };
    let channels = channels.clamp(1, 2) as usize;
    Box::into_raw(Box::new(Session {
        engine,
        history: History::default(),
        channels: vec![Vec::new(); channels],
    }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cleanslate_gate_destroy(session: *mut Session) {
    if !session.is_null() {
        drop(Box::from_raw(session));
    }
}

/// `planar` is channel-major. Bypass leaves the caller buffer untouched.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cleanslate_gate_process(
    session: *mut Session,
    planar: *mut *mut f32,
    frames: u32,
    settings: GateSettings,
) -> f32 {
    if session.is_null() || planar.is_null() || frames == 0 || settings.bypass != 0 {
        return 1.0;
    }
    let session = &mut *session;
    let frames = frames as usize;
    let mut owned = std::mem::take(&mut session.channels);
    for (index, channel) in owned.iter_mut().enumerate() {
        let source = std::slice::from_raw_parts(*planar.add(index), frames);
        channel.clear();
        channel.extend_from_slice(source);
    }
    let minimum = {
        let mut slices: Vec<&mut [f32]> = owned.iter_mut().map(|channel| channel.as_mut_slice()).collect();
        let mut buffer = Buffer::default();
        unsafe {
            buffer.set_slices(frames, |output| {
                output.clear();
                output.append(&mut slices);
            });
        }
        session.engine.process(
            &mut buffer,
            true,
            settings.range_db,
            settings.open_threshold,
            settings.hold_ms,
            settings.attack_ms,
            settings.release_ms,
            settings.level_bias,
            &session.history,
        );
        session.engine.minimum_gain
    };
    for (index, channel) in owned.iter().enumerate() {
        let dest = std::slice::from_raw_parts_mut(*planar.add(index), frames);
        dest.copy_from_slice(channel);
    }
    session.channels = owned;
    minimum
}
