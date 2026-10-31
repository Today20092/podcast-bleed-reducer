#pragma once
#include <array>
#include <juce_audio_processors/juce_audio_processors.h>
struct GateSettings {
    int bypass; float rangeDb, openThreshold, holdMs, attackMs, releaseMs, levelBias;
};
using CreateFn = void* (*)(uint32_t, uint32_t);
using DestroyFn = void (*)(void*);
using ProcessFn = float (*)(void*, float**, uint32_t, GateSettings);
class GateProcessor : public juce::AudioProcessor {
public:
    GateProcessor();
    ~GateProcessor() override;
    void prepareToPlay(double, int) override;
    void releaseResources() override {}
    void processBlock(juce::AudioBuffer<float>&, juce::MidiBuffer&) override;
    juce::AudioProcessorEditor* createEditor() override;
    bool hasEditor() const override { return true; }
    const juce::String getName() const override { return "CleanSlate Smart Gate"; }
    bool acceptsMidi() const override { return false; }
    bool producesMidi() const override { return false; }
    double getTailLengthSeconds() const override { return 0; }
    int getNumPrograms() override { return 1; }
    int getCurrentProgram() override { return 0; }
    void setCurrentProgram(int) override {}
    const juce::String getProgramName(int) override { return {}; }
    void changeProgramName(int, const juce::String&) override {}
    void getStateInformation(juce::MemoryBlock&) override;
    void setStateInformation(const void*, int) override;
    bool isBusesLayoutSupported(const BusesLayout&) const override;
    juce::AudioProcessorValueTreeState state;
    std::atomic<float> minimumGain { 1.0f };
    /// Negative means no activity score. A detector failure stays negative.
    std::atomic<float> activity { -1.0f };
    /// 0 waiting, 1 armed, 2 failed. Bypass is read from the parameter.
    std::atomic<int> link { 0 };
    std::array<std::atomic<float>, 256> reduction {};
    std::atomic<int> reductionWrite { 0 };
    std::atomic<bool> preview { false }, playing { false };
    void loadPreview(const juce::File&);
    juce::CriticalSection previewLock;
    juce::AudioBuffer<float> previewBuffer;
    std::atomic<int> previewPos { 0 };
private:
    juce::AudioProcessorValueTreeState::ParameterLayout layout();
    void loadRust();
    juce::DynamicLibrary rust;
    CreateFn create = nullptr;
    DestroyFn destroy = nullptr;
    ProcessFn process = nullptr;
    void* session = nullptr;
    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(GateProcessor)
};
