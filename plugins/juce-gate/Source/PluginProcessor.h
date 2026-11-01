#pragma once
#include <array>
#include <juce_audio_processors/juce_audio_processors.h>
struct GateSettings {
    int bypass; float rangeDb, openThreshold, holdMs, attackMs, releaseMs, levelBias, lookaheadMs;
};
using CreateFn = void* (*)(uint32_t, uint32_t);
using DestroyFn = void (*)(void*);
using ProcessFn = float (*)(void*, float**, uint32_t, GateSettings);
using ActivityFn = float (*)(const void*);
using ResetFn = void (*)(void*);
class GateProcessor : public juce::AudioProcessor {
public:
    GateProcessor();
    ~GateProcessor() override;
    void prepareToPlay(double, int) override;
    void releaseResources() override {}
    void reset() override;
    void processBlock(juce::AudioBuffer<float>&, juce::MidiBuffer&) override;
    juce::AudioProcessorEditor* createEditor() override;
    bool hasEditor() const override { return true; }
    const juce::String getName() const override { return "CleanSlate Smart Gate"; }
    bool acceptsMidi() const override { return false; }
    bool producesMidi() const override { return false; }
    double getTailLengthSeconds() const override;
    juce::AudioProcessorParameter* getBypassParameter() const override;
    int getNumPrograms() override { return 1; }
    int getCurrentProgram() override { return 0; }
    void setCurrentProgram(int) override {}
    const juce::String getProgramName(int) override { return {}; }
    void changeProgramName(int, const juce::String&) override {}
    void getStateInformation(juce::MemoryBlock&) override;
    void setStateInformation(const void*, int) override;
    bool isBusesLayoutSupported(const BusesLayout&) const override;
    juce::UndoManager undo;
    juce::AudioProcessorValueTreeState state;
    std::atomic<float> minimumGain { 1.0f };
    /// Negative means no activity score. A detector failure stays negative.
    std::atomic<float> activity { -1.0f };
    /// 0 waiting, 1 armed, 2 detector unavailable, 3 sample rate not supported.
    std::atomic<int> link { 0 };
    std::array<std::atomic<float>, 256> reduction {};
    std::atomic<int> reductionWrite { 0 };
    static constexpr int waveCapacity = 2048;
    std::array<std::atomic<float>, waveCapacity> waveMin {};
    std::array<std::atomic<float>, waveCapacity> waveMax {};
    std::array<std::atomic<float>, waveCapacity> waveGain {};
    std::array<std::atomic<float>, waveCapacity> waveActivity {};
    std::atomic<int> columnCount { 0 };
private:
    juce::AudioProcessorValueTreeState::ParameterLayout layout();
    void loadRust();
    void publishLatency();
    static int latencySamplesFor(double rate, float lookaheadMs);
    static bool supportedRate(double rate);
    juce::DynamicLibrary rust;
    CreateFn create = nullptr;
    DestroyFn destroy = nullptr;
    ProcessFn process = nullptr;
    ActivityFn readActivity = nullptr;
    ResetFn resetGate = nullptr;
    void* session = nullptr;
    int sessionChannels = 0;
    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(GateProcessor)
};
