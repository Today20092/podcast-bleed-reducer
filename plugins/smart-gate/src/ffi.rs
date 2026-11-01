//! C boundary for the JUCE editor. The gain policy and detector stay in this crate.
use crate::engine::Engine;
use crate::history::History;
use nice_plug::prelude::{Buffer, ProcessMode};

pub struct Session {
    engine: Engine,
    history: History,
    channels: usize,
    buffer: Buffer<'static>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GateSettings {
    pub bypass: i32,
    pub range_db: f32,
    pub open_threshold: f32,
    pub hold_ms: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub level_bias: f32,
    pub lookahead_ms: f32,
}

#[unsafe(no_mangle)]
pub extern "C" fn cleanslate_gate_create(rate: u32, channels: u32) -> *mut Session {
    let Ok(engine) = Engine::prepare(rate, channels.clamp(1, 2), ProcessMode::Realtime) else {
        return std::ptr::null_mut();
    };
    let channels = channels.clamp(1, 2) as usize;
    let mut buffer = Buffer::default();
    // SAFETY: The buffer has no slices until processing; reservation creates no references.
    unsafe { buffer.set_slices(0, |slices| slices.reserve_exact(channels)) };
    Box::into_raw(Box::new(Session {
        engine,
        history: History::default(),
        channels,
        buffer,
    }))
}

#[unsafe(no_mangle)]
/// Destroys a session created by `cleanslate_gate_create`.
///
/// # Safety
/// A non-null pointer must be a live session returned by create, used exactly once,
/// with no concurrent calls or outstanding references.
pub unsafe extern "C" fn cleanslate_gate_destroy(session: *mut Session) {
    if !session.is_null() {
        // SAFETY: The caller transfers exclusive ownership of the live allocation.
        drop(unsafe { Box::from_raw(session) });
    }
}

#[unsafe(no_mangle)]
/// Clears detector, envelope, and delay state. The host calls this on transport reset.
///
/// # Safety
/// A non-null pointer must refer to a live session with no concurrent calls.
pub unsafe extern "C" fn cleanslate_gate_reset(session: *mut Session) {
    // SAFETY: The caller guarantees exclusive access to a live session when non-null.
    if let Some(session) = unsafe { session.as_mut() } {
        session.engine.reset();
    }
}

#[unsafe(no_mangle)]
/// Returns the current decision score, or -1 when unavailable.
///
/// # Safety
/// A non-null pointer must refer to a live session, with no concurrent mutation.
pub unsafe extern "C" fn cleanslate_gate_activity(session: *const Session) -> f32 {
    // SAFETY: The caller guarantees a live, non-mutated session when non-null.
    unsafe { session.as_ref() }
        .and_then(|session| session.engine.current_activity())
        .unwrap_or(-1.0)
}

/// Processes channel-major audio in place. Bypass leaves the buffer untouched.
///
/// # Safety
/// `session` must be live and exclusively accessible. `planar` must point to the
/// session's number of non-null, aligned, non-overlapping writable channel buffers,
/// each containing `frames` samples, valid for the duration of this call. The
/// pointer array must not overlap the session or any channel buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cleanslate_gate_process(
    session: *mut Session,
    planar: *mut *mut f32,
    frames: u32,
    settings: GateSettings,
) -> f32 {
    if session.is_null() || planar.is_null() || frames == 0 {
        return 1.0;
    }
    // SAFETY: The caller guarantees exclusive access to a live session.
    let session = unsafe { &mut *session };
    let frames = frames as usize;
    for index in 0..session.channels {
        // SAFETY: The caller guarantees a pointer array of the session's channel count.
        if unsafe { (*planar.add(index)).is_null() } {
            return 1.0;
        }
    }
    let minimum = {
        // SAFETY: The caller guarantees valid, disjoint buffers. The vector was
        // reserved at creation, and all borrowed slices are cleared before return.
        unsafe {
            session.buffer.set_slices(frames, |output| {
                output.clear();
                for index in 0..session.channels {
                    output.push(std::slice::from_raw_parts_mut(*planar.add(index), frames));
                }
            });
        }
        session.engine.process(
            &mut session.buffer,
            true,
            settings.range_db,
            settings.open_threshold,
            settings.hold_ms,
            settings.attack_ms,
            settings.release_ms,
            settings.level_bias,
            settings.lookahead_ms,
            &session.history,
            true,
        );
        session.engine.minimum_gain
    };
    // SAFETY: Clearing drops every temporary slice before the caller regains access.
    unsafe { session.buffer.set_slices(0, |slices| slices.clear()) };
    minimum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_callback_does_not_allocate_for_varying_block_sizes() {
        let session = cleanslate_gate_create(48_000, 2);
        assert!(!session.is_null());
        let mut left = [0.0; 1024];
        let mut right = [0.0; 1024];
        let mut planes = [left.as_mut_ptr(), right.as_mut_ptr()];
        let settings = GateSettings {
            bypass: 0,
            range_db: 6.0,
            open_threshold: 0.1,
            hold_ms: 300.0,
            attack_ms: 5.0,
            release_ms: 250.0,
            level_bias: 0.0,
            lookahead_ms: 5.0,
        };
        nice_assert_no_alloc::assert_no_alloc(|| {
            // SAFETY: Both channels are distinct writable buffers, and the session is exclusively owned.
            for frames in [64, 1024, 128] {
                unsafe { cleanslate_gate_process(session, planes.as_mut_ptr(), frames, settings) };
            }
        });
        // SAFETY: The session was created above and is no longer in use.
        unsafe { cleanslate_gate_destroy(session) };
    }

    #[test]
    fn host_bypass_keeps_lookahead_and_rejects_other_rates() {
        assert_eq!(crate::engine::Engine::latency_samples(48_000, 5.0), 240);
        assert_eq!(crate::engine::Engine::latency_samples(48_000, 20.0), 960);
        assert_eq!(crate::engine::Engine::latency_samples(48_000, 0.0), 0);
        assert!(cleanslate_gate_create(88_200, 2).is_null());
        assert!(cleanslate_gate_create(192_000, 1).is_null());
        let session = cleanslate_gate_create(48_000, 1);
        assert!(!session.is_null());
        let mut audio = vec![0.25_f32; 480];
        let mut planes = [audio.as_mut_ptr()];
        let settings = GateSettings {
            bypass: 1,
            range_db: 60.0,
            open_threshold: 0.1,
            hold_ms: 0.0,
            attack_ms: 5.0,
            release_ms: 20.0,
            level_bias: 0.0,
            lookahead_ms: 5.0,
        };
        // SAFETY: One writable channel, exclusively owned, matches the session.
        unsafe { cleanslate_gate_process(session, planes.as_mut_ptr(), 480, settings) };
        assert!(audio[100].abs() < 0.001, "delay line starts empty, got {}", audio[100]);
        assert!(
            (audio[240] - 0.25).abs() < 0.02,
            "bypass still delays by 5 ms, got {}",
            audio[240]
        );
        assert!(
            (audio[400] - 0.25).abs() < 0.02,
            "bypassed audio stays at unity after the slew, got {}",
            audio[400]
        );
        // SAFETY: The session is still exclusively owned.
        unsafe { cleanslate_gate_reset(session) };
        audio.fill(0.25);
        unsafe { cleanslate_gate_process(session, planes.as_mut_ptr(), 480, settings) };
        assert!(audio[100].abs() < 0.001, "reset clears the delay, got {}", audio[100]);
        unsafe { cleanslate_gate_destroy(session) };
    }
}
