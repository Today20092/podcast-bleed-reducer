#pragma once
#include <juce_audio_processors/juce_audio_processors.h>
class GateProcessor;
class GateLook : public juce::LookAndFeel_V4 {
public:
    void drawRotarySlider(juce::Graphics&, int x, int y, int width, int height, float position,
        float start, float end, juce::Slider&) override;
};
class GateKnob : public juce::Slider {
public:
    GateKnob(const juce::String& name, double resetValue);
    void mouseDown(const juce::MouseEvent&) override;
    void mouseDoubleClick(const juce::MouseEvent&) override;
    void mouseDrag(const juce::MouseEvent&) override;
    void mouseWheelMove(const juce::MouseEvent&, const juce::MouseWheelDetails&) override;
private:
    double resetValue;
    bool fine = false;
};
class GateEditor : public juce::AudioProcessorEditor, private juce::Button::Listener, private juce::Timer {
public:
    explicit GateEditor(GateProcessor&);
    ~GateEditor() override;
    void paint(juce::Graphics&) override;
    void resized() override;
private:
    void buttonClicked(juce::Button*) override;
    void timerCallback() override;
    GateProcessor& gate;
    GateLook look;
    GateKnob range, threshold, hold, attack, release, bias;
    juce::Label hover;
    juce::TextButton play { "Play" }, load { "Load recording" };
    juce::ToggleButton bypass { "Bypass" };
    using Attachment = juce::AudioProcessorValueTreeState::SliderAttachment;
    std::unique_ptr<Attachment> aRange, aThreshold, aHold, aAttack, aRelease, aBias;
    std::unique_ptr<juce::AudioProcessorValueTreeState::ButtonAttachment> aBypass;
};
