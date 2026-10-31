#include "PluginProcessor.h"
#include "PluginEditor.h"
#include <juce_audio_formats/juce_audio_formats.h>
GateProcessor::GateProcessor()
    : AudioProcessor(BusesProperties().withInput("Input", juce::AudioChannelSet::stereo(), true)
                                      .withOutput("Output", juce::AudioChannelSet::stereo(), true)),
      state(*this, nullptr, "gate", layout()) {}
GateProcessor::~GateProcessor() { if (session && destroy) destroy(session); }
juce::AudioProcessorValueTreeState::ParameterLayout GateProcessor::layout() {
    using namespace juce;
    return {
        std::make_unique<AudioParameterBool>(ParameterID{"bypass", 1}, "Bypass", true),
        std::make_unique<AudioParameterFloat>(ParameterID{"range", 1}, "Range", NormalisableRange<float>(0, 6, 0.01f), 6),
        std::make_unique<AudioParameterFloat>(ParameterID{"threshold", 1}, "Threshold", NormalisableRange<float>(0, 1, 0.001f), 0.10f),
        std::make_unique<AudioParameterFloat>(ParameterID{"hold", 1}, "Hold", NormalisableRange<float>(0, 1000, 1), 300),
        std::make_unique<AudioParameterFloat>(ParameterID{"attack", 1}, "Attack", NormalisableRange<float>(1, 1000, 1), 250),
        std::make_unique<AudioParameterFloat>(ParameterID{"release", 1}, "Release", NormalisableRange<float>(1, 200, 0.1f), 5),
        std::make_unique<AudioParameterFloat>(ParameterID{"bias", 1}, "Level Bias", NormalisableRange<float>(0, 1, 0.01f), 0)
    };
}
bool GateProcessor::isBusesLayoutSupported(const BusesLayout& buses) const {
    return buses.getMainInputChannelSet() == buses.getMainOutputChannelSet()
        && buses.getMainInputChannelSet() == juce::AudioChannelSet::stereo();
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
    }
}
void GateProcessor::prepareToPlay(double rate, int) {
    loadRust();
    if (session && destroy) destroy(session);
    session = create ? create((uint32_t) rate, 2) : nullptr;
}
void GateProcessor::loadPreview(const juce::File& file) {
    juce::AudioFormatManager formats;
    formats.registerBasicFormats();
    std::unique_ptr<juce::AudioFormatReader> reader(formats.createReaderFor(file));
    if (!reader) return;
    const juce::ScopedLock lock(previewLock);
    previewBuffer.setSize((int) reader->numChannels, (int) reader->lengthInSamples);
    reader->read(&previewBuffer, 0, (int) reader->lengthInSamples, 0, true, true);
    previewPos = 0;
    preview = true;
    playing = false;
}
void GateProcessor::processBlock(juce::AudioBuffer<float>& buffer, juce::MidiBuffer&) {
    if (*state.getRawParameterValue("bypass") > 0.5f) {
        activity = -1.0f;
        return;
    }
    if (!process || !session) {
        link = 2;
        activity = -1.0f;
        return;
    }
    link = 1;
    if (preview.load() && !playing.load()) {
        buffer.clear();
        return;
    }
    const juce::ScopedLock lock(previewLock);
    if (preview.load() && playing.load() && previewBuffer.getNumSamples() > 0) {
        int pos = previewPos.load();
        for (int i = 0; i < buffer.getNumSamples(); ++i) {
            int source = pos + i;
            if (source >= previewBuffer.getNumSamples()) source = previewBuffer.getNumSamples() - 1;
            for (int c = 0; c < buffer.getNumChannels(); ++c)
                buffer.setSample(c, i, previewBuffer.getSample(std::min(c, previewBuffer.getNumChannels() - 1), source));
        }
        previewPos = pos + buffer.getNumSamples();
        if (previewPos >= previewBuffer.getNumSamples()) { playing = false; previewPos = 0; }
    }
    if (!process || !session) return;
    float* planes[2] = { buffer.getWritePointer(0), buffer.getWritePointer(1) };
    GateSettings settings { 0, *state.getRawParameterValue("range"), *state.getRawParameterValue("threshold"),
        *state.getRawParameterValue("hold"), *state.getRawParameterValue("attack"), *state.getRawParameterValue("release"),
        *state.getRawParameterValue("bias") };
    minimumGain = process(session, planes, (uint32_t) buffer.getNumSamples(), settings);
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
