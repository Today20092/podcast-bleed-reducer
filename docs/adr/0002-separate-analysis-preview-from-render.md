# Separate live audition from final render

Parameter changes update ownership overlays and read-only gain envelopes quickly inside the editor. Original, Processed, and Removed audition apply the current completed envelopes during playback without writing intermediate WAV files; while recalculation runs, playback continues with the last completed envelope and switches seamlessly when the new one is ready. Final rendering exists only to create and validate deliverable files after the session looks and sounds acceptable, so an old deliverable may be stale without disabling live audition.
