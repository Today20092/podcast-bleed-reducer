#include "PluginProcessor.h"
#include "PluginEditor.h"
#include <algorithm>
#include <cmath>
GateProcessor::GateProcessor()
    : AudioProcessor(BusesProperties().withInput("Input", juce::AudioChannelSet::stereo(), true)
                                      .withOutput("Output", juce::AudioChannelSet::stereo(), true)),
      state(*this, &undo, "gate", layout()) {
    for (auto& slot : reduction) slot.store(1.0f);
}
GateProcessor::~GateProcessor() { if (session && destroy) destroy(session); }
juce::AudioProcessorValueTreeState::ParameterLayout GateProcessor::layout() {
    using namespace juce;
    return {
        std::make_unique<AudioParameterBool>(ParameterID{"bypass", 1}, "Bypass", false),
        std::make_unique<AudioParameterFloat>(ParameterID{"range", 1}, "Depth", NormalisableRange<float>(0, 60, 0.01f), 6),
        std::make_unique<AudioParameterFloat>(ParameterID{"threshold", 1}, "Voice", NormalisableRange<float>(0, 1, 0.001f), 0.10f),
        std::make_unique<AudioParameterFloat>(ParameterID{"hold", 1}, "Hold", NormalisableRange<float>(0, 1000, 1), 300),
        std::make_unique<AudioParameterFloat>(ParameterID{"attack", 1}, "Attack", NormalisableRange<float>(1, 1000, 1), 5),
        std::make_unique<AudioParameterFloat>(ParameterID{"release", 1}, "Release", NormalisableRange<float>(1, 1000, 1), 250),
        std::make_unique<AudioParameterFloat>(ParameterID{"bias", 1}, "Level Bias", NormalisableRange<float>(0, 1, 0.01f), 0),
        std::make_unique<AudioParameterFloat>(ParameterID{"lookahead", 1}, "Lookahead", NormalisableRange<float>(0, 20, 0.1f), 5)
    };
}
bool GateProcessor::isBusesLayoutSupported(const BusesLayout& buses) const {
    const auto input = buses.getMainInputChannelSet();
    return input == buses.getMainOutputChannelSet()
        && (input == juce::AudioChannelSet::mono() || input == juce::AudioChannelSet::stereo());
}
bool GateProcessor::supportedRate(double rate) {
    const int rounded = (int) std::llround(rate);
    return rounded == 44100 || rounded == 48000 || rounded == 96000;
}
int GateProcessor::latencySamplesFor(double rate, float lookaheadMs) {
    if (!(rate > 0.0)) return 0;
    const int cap = (int) rate * 20 / 1000 + 8;
    const double ms = std::clamp((double) lookaheadMs, 0.0, 20.0);
    const int requested = (int) std::llround(rate * ms / 1000.0);
    return std::min(requested, std::max(0, cap - 1));
}
void GateProcessor::publishLatency() {
    if (!session) {
        if (getLatencySamples() != 0) setLatencySamples(0);
        return;
    }
    const int samples = latencySamplesFor(getSampleRate(), *state.getRawParameterValue("lookahead"));
    if (samples != getLatencySamples()) setLatencySamples(samples);
}
double GateProcessor::getTailLengthSeconds() const {
    const float hold = *state.getRawParameterValue("hold");
    const float release = *state.getRawParameterValue("release");
    const float lookahead = *state.getRawParameterValue("lookahead");
    return (double) (hold + release + lookahead) / 1000.0;
}
juce::AudioProcessorParameter* GateProcessor::getBypassParameter() const {
    return state.getParameter("bypass");
}
void GateProcessor::loadRust() {
    if (process) return;
    auto dll = juce::File::getSpecialLocation(juce::File::currentExecutableFile)
                   .getParentDirectory().getChildFile("cleanslate_smart_gate.dll");
    if (!dll.existsAsFile())
        dll = juce::File(__FILE__).getParentDirectory().getParentDirectory().getParentDirectory()
                  .getChildFile("smart-gate").getChildFile("target").getChildFile("release")
                  .getChildFile("cleanslate_smart_gate.dll");
    if (rust.open(dll.getFullPathName())) {
        create = (CreateFn) rust.getFunction("cleanslate_gate_create");
        destroy = (DestroyFn) rust.getFunction("cleanslate_gate_destroy");
        process = (ProcessFn) rust.getFunction("cleanslate_gate_process");
        readActivity = (ActivityFn) rust.getFunction("cleanslate_gate_activity");
        resetGate = (ResetFn) rust.getFunction("cleanslate_gate_reset");
    }
}
void GateProcessor::prepareToPlay(double rate, int) {
    loadRust();
    if (session && destroy) destroy(session);
    session = nullptr;
    sessionChannels = 0;
    const int channels = juce::jlimit(1, 2, getMainBusNumInputChannels());
    if (create && supportedRate(rate))
        session = create((uint32_t) std::llround(rate), (uint32_t) channels);
    if (session) {
        sessionChannels = channels;
        link = 0;
    } else {
        link = supportedRate(rate) ? 2 : 3;
    }
    publishLatency();
}
void GateProcessor::reset() {
    if (session && resetGate) resetGate(session);
}
void GateProcessor::processBlock(juce::AudioBuffer<float>& buffer, juce::MidiBuffer&) {
    juce::ScopedNoDenormals noDenormals;
    publishLatency();
    float lo = 0.0f, hi = 0.0f;
    if (buffer.getNumChannels() > 0 && buffer.getNumSamples() > 0) {
        lo = hi = buffer.getSample(0, 0);
        for (int i = 1; i < buffer.getNumSamples(); ++i) {
            float s = buffer.getSample(0, i);
            lo = std::min(lo, s);
            hi = std::max(hi, s);
        }
    }
    int waveSlot = columnCount.load() % waveCapacity;
    waveMin[(size_t) waveSlot] = lo;
    waveMax[(size_t) waveSlot] = hi;
    const bool bypassed = *state.getRawParameterValue("bypass") > 0.5f;
    if (!process || !session || buffer.getNumChannels() < sessionChannels || sessionChannels <= 0) {
        link = supportedRate(getSampleRate()) ? 2 : 3;
        activity = -1.0f;
        minimumGain = 1.0f;
        waveGain[(size_t) waveSlot] = 1.0f;
        waveActivity[(size_t) waveSlot] = -1.0f;
        columnCount = columnCount.load() + 1;
        return;
    }
    link = 1;
    float* planes[2] = {};
    for (int channel = 0; channel < sessionChannels; ++channel)
        planes[channel] = buffer.getWritePointer(channel);
    GateSettings settings { bypassed ? 1 : 0, *state.getRawParameterValue("range"), *state.getRawParameterValue("threshold"),
        *state.getRawParameterValue("hold"), *state.getRawParameterValue("attack"), *state.getRawParameterValue("release"),
        *state.getRawParameterValue("bias"), *state.getRawParameterValue("lookahead") };
    minimumGain = process(session, planes, (uint32_t) buffer.getNumSamples(), settings);
    activity = bypassed || !readActivity ? -1.0f : readActivity(session);
    waveActivity[(size_t) waveSlot] = activity.load();
    waveGain[(size_t) waveSlot] = minimumGain.load();
    columnCount = columnCount.load() + 1;
    int slot = reductionWrite.load() % (int) reduction.size();
    reduction[(size_t) slot] = minimumGain.load();
    reductionWrite = slot + 1;
}
void GateProcessor::getStateInformation(juce::MemoryBlock& dest) {
    if (auto xml = state.copyState().createXml()) copyXmlToBinary(*xml, dest);
}
void GateProcessor::setStateInformation(const void* data, int size) {
    if (auto xml = getXmlFromBinary(data, size)) state.replaceState(juce::ValueTree::fromXml(*xml));
}
juce::AudioProcessorEditor* GateProcessor::createEditor() { return new GateEditor(*this); }
juce::AudioProcessor* JUCE_CALLTYPE createPluginFilter() { return new GateProcessor(); }
