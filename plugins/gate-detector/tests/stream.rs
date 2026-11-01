use cleanslate_gate_detector::Detector;

#[test]
fn causal_events_are_block_invariant_and_reset_discards_history() {
    for rate in [44_100, 48_000, 96_000] {
        let mut detector = Detector::new(rate).unwrap();
        let samples = vec![0.0; rate as usize / 5];
        let mut first = Vec::new();
        detector.push(&samples, |event| first.push(event)).unwrap();
        assert_eq!(first.len(), 6);
        assert_eq!(
            first[0].available_at,
            (512_u64 * rate as u64).div_ceil(16_000)
        );
        assert!(first.iter().all(|e| (0.0..=1.0).contains(&e.probability)));
        assert!(
            first.iter().all(|e| e.probability < 0.1),
            "constructed silence should not be speech"
        );
        let contamination: Vec<_> = (0..rate / 4)
            .map(|i| (i as f32 * 0.17).sin() * 0.2)
            .collect();
        detector.push(&contamination, |_| {}).unwrap();
        detector.reset();
        let mut second = Vec::new();
        for block in samples.chunks(17) {
            detector.push(block, |event| second.push(event)).unwrap();
        }
        assert_eq!(first.len(), second.len());
        for (a, b) in first.iter().zip(&second) {
            assert_eq!(a.probability, b.probability);
            assert_eq!(a.available_at, b.available_at);
            assert_eq!(b.epoch, a.epoch + 1);
        }
        assert!(detector.push(&[f32::NAN], |_| {}).is_err());
        assert!(
            detector.push(&[0.0], |_| {}).is_err(),
            "fault is latched until reset"
        );
        detector.reset();
        assert!(detector.push(&[0.0], |_| {}).is_ok());
    }
}
