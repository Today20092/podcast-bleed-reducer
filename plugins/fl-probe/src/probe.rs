use std::sync::atomic::{AtomicI64, AtomicU64, Ordering::SeqCst};

const GROUPS: usize = 8;
const MEMBERS: usize = 2;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
pub static GROUPS_STATE: Registry = Registry::new();

struct Slot {
    owner: AtomicU64,
    sequence: AtomicU64,
    start: AtomicI64,
    frames: AtomicU64,
    rate: AtomicU64,
    epoch: AtomicU64,
}

impl Slot {
    const fn new() -> Self {
        Self {
            owner: AtomicU64::new(0),
            sequence: AtomicU64::new(0),
            start: AtomicI64::new(0),
            frames: AtomicU64::new(0),
            rate: AtomicU64::new(0),
            epoch: AtomicU64::new(0),
        }
    }

    fn publish(&self, start: i64, frames: u64, rate: u64, epoch: u64) {
        self.sequence.fetch_add(1, SeqCst);
        self.start.store(start, SeqCst);
        self.frames.store(frames, SeqCst);
        self.rate.store(rate, SeqCst);
        self.epoch.store(epoch, SeqCst);
        self.sequence.fetch_add(1, SeqCst);
    }

    fn read(&self) -> Option<(i64, u64, u64, u64)> {
        let before = self.sequence.load(SeqCst);
        let owner = self.owner.load(SeqCst);
        if before == 0 || !before.is_multiple_of(2) || owner == 0 {
            return None;
        }
        let result = (
            self.start.load(SeqCst),
            self.frames.load(SeqCst),
            self.rate.load(SeqCst),
            self.epoch.load(SeqCst),
        );
        (before == self.sequence.load(SeqCst) && owner == self.owner.load(SeqCst)).then_some(result)
    }
}

pub struct Registry {
    slots: [Slot; GROUPS * MEMBERS],
    epochs: [AtomicU64; GROUPS],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Binding {
    pub index: usize,
    pub epoch: u64,
    pub owner: u64,
    pub peer: u64,
}

impl Registry {
    pub(crate) const fn new() -> Self {
        Self {
            slots: [const { Slot::new() }; GROUPS * MEMBERS],
            epochs: [const { AtomicU64::new(1) }; GROUPS],
        }
    }

    pub fn is_current(&self, binding: Binding) -> bool {
        self.epochs[binding.index / MEMBERS].load(SeqCst) == binding.epoch
            && self.slots[binding.index].owner.load(SeqCst) == binding.owner
            && self.slots[binding.index ^ 1].owner.load(SeqCst) == binding.peer
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Disabled,
    DuplicateMember,
    MissingTime,
    Stopped,
    MissingPeer,
    EpochMismatch,
    RateMismatch,
    SameBlock,
    DifferentBlock,
}

#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub group: i32,
    pub member: i32,
    pub start: Option<i64>,
    pub frames: u64,
    pub rate: u32,
    pub playing: bool,
    pub epoch: u64,
    pub discontinuity: bool,
    pub peer_start: Option<i64>,
    pub peer_frames: u64,
    pub status: Status,
}

pub struct Probe {
    pub id: u64,
    slot: Option<usize>,
    expected: Option<i64>,
    playing: bool,
}

impl Probe {
    pub fn new() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, SeqCst),
            slot: None,
            expected: None,
            playing: false,
        }
    }

    pub fn reset(&mut self, registry: &Registry) {
        if let Some(slot) = self.slot.take() {
            registry.epochs[slot / MEMBERS].fetch_add(1, SeqCst);
            registry.slots[slot].frames.store(0, SeqCst);
            let _ = registry.slots[slot]
                .owner
                .compare_exchange(self.id, 0, SeqCst, SeqCst);
        }
        self.expected = None;
        self.playing = false;
    }

    pub fn binding(&self, registry: &Registry) -> Option<Binding> {
        self.slot.map(|index| Binding {
            index,
            epoch: registry.epochs[index / MEMBERS].load(SeqCst),
            owner: self.id,
            peer: registry.slots[index ^ 1].owner.load(SeqCst),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn observe(
        &mut self,
        registry: &Registry,
        group: i32,
        member: i32,
        start: Option<i64>,
        frames: u64,
        rate: u32,
        playing: bool,
    ) -> Observation {
        let wanted =
            if (1..=GROUPS as i32).contains(&group) && (1..=MEMBERS as i32).contains(&member) {
                Some((group as usize - 1) * MEMBERS + member as usize - 1)
            } else {
                None
            };
        if wanted != self.slot {
            self.reset(registry);
        }
        let discontinuity = self.playing != playing
            || (playing && self.expected.is_some() && self.expected != start);
        let mut result = Observation {
            group,
            member,
            start,
            frames,
            rate,
            playing,
            epoch: 0,
            discontinuity,
            peer_start: None,
            peer_frames: 0,
            status: Status::Disabled,
        };
        let Some(index) = wanted else {
            return result;
        };
        let slot = &registry.slots[index];
        if self.slot.is_none() {
            if slot
                .owner
                .compare_exchange(0, self.id, SeqCst, SeqCst)
                .is_err()
            {
                result.status = Status::DuplicateMember;
                return result;
            }
            self.slot = Some(index);
            // Invalidate the previous occupant's published observation before exposing this slot.
            slot.publish(0, 0, 0, 0);
        }
        let epoch = &registry.epochs[index / MEMBERS];
        if discontinuity {
            epoch.fetch_add(1, SeqCst);
        }
        result.epoch = epoch.load(SeqCst);
        self.playing = playing;
        self.expected = if playing {
            start.and_then(|s| s.checked_add(frames.try_into().ok()?))
        } else {
            None
        };
        let Some(position) = start else {
            slot.publish(0, 0, 0, 0);
            result.status = Status::MissingTime;
            return result;
        };
        slot.publish(
            position,
            if playing { frames } else { 0 },
            u64::from(rate),
            result.epoch,
        );
        if !playing {
            result.status = Status::Stopped;
            return result;
        }
        let peer = &registry.slots[index ^ 1];
        let Some((peer_start, peer_frames, peer_rate, peer_epoch)) =
            peer.read().filter(|p| p.1 > 0)
        else {
            result.status = Status::MissingPeer;
            return result;
        };
        result.peer_start = Some(peer_start);
        result.peer_frames = peer_frames;
        result.status = if peer_epoch != result.epoch {
            Status::EpochMismatch
        } else if peer_rate != u64::from(rate) {
            Status::RateMismatch
        } else if position == peer_start && frames == peer_frames {
            Status::SameBlock
        } else {
            Status::DifferentBlock
        };
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_delivery_missing_peer_duplicate_and_release() {
        let registry = Registry::new();
        let (mut a, mut b, mut duplicate) = (Probe::new(), Probe::new(), Probe::new());
        assert_eq!(
            a.observe(&registry, 1, 1, Some(0), 256, 48000, true).status,
            Status::MissingPeer
        );
        b.observe(&registry, 1, 2, Some(0), 256, 48000, true);
        let ahead = a.observe(&registry, 1, 1, Some(256), 256, 48000, true);
        assert_eq!(ahead.status, Status::DifferentBlock);
        assert_eq!(ahead.peer_start, Some(0));
        assert_eq!(
            b.observe(&registry, 1, 2, Some(256), 256, 48000, true)
                .status,
            Status::SameBlock
        );
        assert_eq!(
            duplicate
                .observe(&registry, 1, 2, Some(256), 256, 48000, true)
                .status,
            Status::DuplicateMember
        );
        b.reset(&registry);
        assert_eq!(
            a.observe(&registry, 1, 1, Some(512), 256, 48000, true)
                .status,
            Status::MissingPeer
        );
        assert_ne!(
            duplicate
                .observe(&registry, 1, 2, Some(512), 256, 48000, true)
                .status,
            Status::DuplicateMember
        );
    }

    #[test]
    fn seek_missing_time_rate_and_groups_are_not_synchronized() {
        let registry = Registry::new();
        let (mut a, mut b) = (Probe::new(), Probe::new());
        a.observe(&registry, 1, 1, Some(0), 256, 48000, true);
        b.observe(&registry, 1, 2, Some(0), 256, 48000, true);
        assert!(
            a.observe(&registry, 1, 1, Some(9999), 256, 48000, true)
                .discontinuity
        );
        assert_eq!(
            a.observe(&registry, 1, 1, None, 256, 48000, true).status,
            Status::MissingTime
        );
        assert_eq!(
            b.observe(&registry, 1, 2, Some(256), 256, 48000, true)
                .status,
            Status::MissingPeer
        );
        a.observe(&registry, 1, 1, Some(0), 256, 44100, true);
        b.observe(&registry, 1, 2, Some(0), 256, 48000, true);
        assert_eq!(
            a.observe(&registry, 1, 1, Some(256), 256, 44100, true)
                .status,
            Status::RateMismatch
        );
        assert_eq!(
            b.observe(&registry, 2, 2, Some(256), 256, 48000, true)
                .status,
            Status::MissingPeer
        );
        assert_eq!(
            a.observe(&registry, 0, 1, Some(256), 256, 48000, true)
                .status,
            Status::Disabled
        );
    }
}
