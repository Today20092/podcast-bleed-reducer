# Deferred Nemotron experiment

The owner removed Nemotron from CleanSlate on 2026-10-09 after testing the acoustic
workflow. It is absent from the GUI, inference pipeline, package and dependencies.
No model acquisition or native diarization setup is required.

Earlier experiments used NVIDIA NeMo-Speech.cpp revision
`8642eaa5cc51efbc17ad0f3e433944ba858a873f` and Nemotron-3-Diarization revision
`f667ed73aee57d40cc39428eb768b4fd87a0a29e`, q8_0 artifact SHA-256
`08456d9e22cd9a323c0364d98375f3746d6e68507ebb705cd46438c534c7a3a1`.
Their implementation, runtime-isolation decisions and license records are in Git
history. These historical identifiers do not establish current support or licensing.

A future trial would need explicit authorization, current license review and an
independent comparison of wanted-speech preservation, overlap, bleed, listening,
latency and memory against acoustic analysis. No benefit has been established that
justifies retaining this integration in the shipping application.
