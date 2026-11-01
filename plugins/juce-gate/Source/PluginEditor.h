#pragma once
#include <juce_audio_processors/juce_audio_processors.h>
class GateProcessor;
class GateLook : public juce::LookAndFeel_V4 {
public:
    void drawRotarySlider(juce::Graphics&, int x, int y, int width, int height, float position,
        float start, float end, juce::Slider&) override;
    void drawButtonBackground(juce::Graphics&, juce::Button&, const juce::Colour&, bool, bool) override;
    juce::Label* createSliderTextBox(juce::Slider&) override;
};
class GateKnob : public juce::Slider {
public:
    GateKnob(const juce::String& name, double resetValue);
    void mouseDown(const juce::MouseEvent&) override;
    void mouseDoubleClick(const juce::MouseEvent&) override;
    void mouseDrag(const juce::MouseEvent&) override;
    void mouseWheelMove(const juce::MouseEvent&, const juce::MouseWheelDetails&) override;
    void configureValueEntry();
private:
    double resetValue;
    double fineWheelPosition = 0.0;
    double fineWheelValue = 0.0;
    bool fineWheelActive = false;
};
class GateEditor : public juce::AudioProcessorEditor, private juce::Button::Listener, private juce::Timer {
public:
    explicit GateEditor(GateProcessor&);
    ~GateEditor() override;
    void paint(juce::Graphics&) override;
    void resized() override;
    void mouseDown(const juce::MouseEvent&) override;
    void mouseDrag(const juce::MouseEvent&) override;
    void mouseUp(const juce::MouseEvent&) override;
    bool keyPressed(const juce::KeyPress&) override;
private:
    void buttonClicked(juce::Button*) override;
    void timerCallback() override;
    void storeActive();
    void layoutPlotRegions();
    void setVoiceFromY(float y);
    GateProcessor& gate;
    GateLook look;
    GateKnob range, threshold, hold, attack, release, lookahead, bias;
    juce::Label rangeName, thresholdName, holdName, attackName, releaseName, lookaheadName, biasName;
    juce::TextButton undoBtn { "Undo" }, redoBtn { "Redo" }, abBtn { "A" }, copyBtn { "Copy" };
    juce::TextButton displayBtn { "More" }, bypass { "Bypass" };
    enum class DisplayMode { slowDown = 1, fast = 2, slow = 3, infinite = 4 };
    DisplayMode displayMode = DisplayMode::fast;
    struct Column { float activity, gain; };
    std::vector<Column> pen;
    int penIndex = 0;
    int columnsSeen = 0;
    int idleTicks = 0;
    int slowDownHold = 0;
    using Attachment = juce::AudioProcessorValueTreeState::SliderAttachment;
    std::unique_ptr<Attachment> aRange, aThreshold, aHold, aAttack, aRelease, aLookahead, aBias;
    std::unique_ptr<juce::AudioProcessorValueTreeState::ButtonAttachment> aBypass;
    juce::ValueTree slotA, slotB;
    bool showingA = true;
    bool controlsOpen = false;
    float peakGr = 0;
    float shownThreshold = 0.10f, shownRange = 6, shownHold = 300, shownAttack = 5, shownRelease = 250, shownBias = 0;
    juce::Rectangle<int> meterBounds, plotBounds, shortcutBounds;
    juce::Rectangle<float> speechBounds, reductionBounds;
    bool dragVoice = false;
    float lastVoiceY = 0.0f;
};
