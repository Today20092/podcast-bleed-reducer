//! Offline smoke test executed by the shipped executable.
use crate::{Project, RenderBlocker, TICKS_PER_SECOND, TimeRange};
use std::sync::atomic::{AtomicBool, AtomicU8};

pub fn package_self_check() -> Result<(), RenderBlocker> {
    let root = std::env::temp_dir().join(format!(
        "cleanslate-package-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| RenderBlocker(e.to_string()))?
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).map_err(|e| RenderBlocker(e.to_string()))?;
    let result = (|| {
        let mut paths = Vec::new();
        for track in 0..2 {
            let path = root.join(format!("speaker-{track}.wav"));
            let mut format = bwavfile::WaveFmt::new_pcm_mono(16000, 32);
            format.tag = 3;
            let mut writer = bwavfile::WaveWriter::create(&path, format)
                .map_err(|e| RenderBlocker(e.to_string()))?
                .audio_frame_writer()
                .map_err(|e| RenderBlocker(e.to_string()))?;
            let samples: Vec<f32> = (0..64000)
                .map(|frame| {
                    let active = if track == 0 {
                        8000..24000
                    } else {
                        32000..48000
                    };
                    if active.contains(&frame) {
                        (frame as f32 * (0.07 + track as f32 * 0.03)).sin() * 0.1
                    } else {
                        0.0
                    }
                })
                .collect();
            writer
                .write_frames(&samples)
                .map_err(|e| RenderBlocker(e.to_string()))?;
            writer.end().map_err(|e| RenderBlocker(e.to_string()))?;
            paths.push(path);
        }
        let mut project = Project::create(root.join("smoke.cleanslate"), paths)
            .map_err(|e| RenderBlocker(e.to_string()))?;
        project
            .set_supplied_alignment(true)
            .map_err(|e| RenderBlocker(e.to_string()))?;
        project
            .add_reference_label(TimeRange::new(0, TICKS_PER_SECOND / 4)?, vec![], &[])
            .map_err(|e| RenderBlocker(e.to_string()))?;
        for (track, start) in [TICKS_PER_SECOND / 2, 2 * TICKS_PER_SECOND]
            .into_iter()
            .enumerate()
        {
            project
                .add_reference_label(
                    TimeRange::new(start, start + TICKS_PER_SECOND)?,
                    vec![project.sources[track].speaker_id],
                    &[],
                )
                .map_err(|e| RenderBlocker(e.to_string()))?;
        }
        let progress = AtomicU8::new(0);
        let mut worker = crate::EditorAnalysisWorker::default();
        let analysis = worker.analyze(
            &project,
            &root.join("cache"),
            &progress,
            &AtomicBool::new(false),
        )?;
        let effective = analysis.install(
            &mut project,
            &mut crate::EffectiveEnvelopeCache::default(),
            None,
        )?;
        let snapshot = crate::SupervisedRenderSnapshot::capture(&project, &analysis, &effective)?;
        let generation = crate::RenderGeneration::default();
        crate::prepare_supervised_render(
            &project,
            &analysis,
            &effective,
            &snapshot,
            &root.join("output"),
            &progress,
            None,
        )?
        .publish(&project, &generation, generation.current(), None)?
        .accept(&mut project)?;
        let reopened = Project::open(project.path()).map_err(|e| RenderBlocker(e.to_string()))?;
        if reopened.reference_labels().len() != 3 || reopened.analysis_recipe().is_none() {
            return Err(RenderBlocker("Package project round trip failed.".into()));
        }
        if worker
            .analyze(
                &reopened,
                &root.join("cache"),
                &progress,
                &AtomicBool::new(true),
            )
            .is_ok()
        {
            return Err(RenderBlocker("Package cancellation check failed.".into()));
        }
        Ok(())
    })();
    std::fs::remove_dir_all(&root).map_err(|e| RenderBlocker(e.to_string()))?;
    result
}
