#include "PluginEditor.h"
#include "PluginProcessor.h"
void GateLook::drawRotarySlider(juce::Graphics& g, int x, int y, int width, int height, float position,
    float start, float end, juce::Slider& slider) {
    auto bounds = juce::Rectangle<float>(x, y, width, height).reduced(8);
    auto radius = juce::jmin(bounds.getWidth(), bounds.getHeight()) / 2.0f;
    auto centre = bounds.getCentre();
    auto angle = start + position * (end - start);
    g.setColour(juce::Colour(0xff2a2e33));
    g.fillEllipse(centre.x - radius, centre.y - radius, radius * 2, radius * 2);
    g.setColour(slider.isMouseOverOrDragging() ? juce::Colour(0xffffd36a) : juce::Colour(0xff3c4148));
    juce::Path track;
    track.addCentredArc(centre.x, centre.y, radius - 4, radius - 4, 0, start, end, true);
    g.strokePath(track, juce::PathStrokeType(3.0f));
    g.setColour(juce::Colour(0xfff2b030));
    juce::Path value;
    value.addCentredArc(centre.x, centre.y, radius - 4, radius - 4, 0, start, angle, true);
    g.strokePath(value, juce::PathStrokeType(3.2f));
    g.fillEllipse(centre.x - 3, centre.y - radius + 10, 6, 6);
    g.setColour(juce::Colour(0xffd7dde4));
    g.setFont(13);
    g.drawText(slider.getTextFromValue(slider.getValue()), bounds, juce::Justification::centred);
}
GateKnob::GateKnob(const juce::String& name, double reset) : resetValue(reset) {
    setSliderStyle(RotaryHorizontalVerticalDrag);
    setTextBoxStyle(NoTextBox, false, 0, 0);
    setRotaryParameters(juce::MathConstants<float>::pi * 0.75f, juce::MathConstants<float>::pi * 2.25f, true);
    setVelocityBasedMode(true);
    setVelocityModeParameters(0.8, 1, 0.08, true);
    setName(name);
    setDoubleClickReturnValue(false, reset);
}
void GateKnob::mouseDown(const juce::MouseEvent& e) {
    if (e.mods.isCtrlDown() || e.mods.isCommandDown()) { setValue(resetValue, juce::sendNotificationSync); return; }
    fine = e.mods.isShiftDown();
    setMouseDragSensitivity(fine ? 800 : 180);
    Slider::mouseDown(e);
}
void GateKnob::mouseDoubleClick(const juce::MouseEvent&) { showTextBox(); }
void GateKnob::mouseDrag(const juce::MouseEvent& e) {
    fine = e.mods.isShiftDown();
    setMouseDragSensitivity(fine ? 800 : 180);
    Slider::mouseDrag(e);
}
void GateKnob::mouseWheelMove(const juce::MouseEvent& e, const juce::MouseWheelDetails& wheel) {
    setMouseDragSensitivity(e.mods.isShiftDown() ? 800 : 180);
    Slider::mouseWheelMove(e, wheel);
}
GateEditor::GateEditor(GateProcessor& processor)
    : AudioProcessorEditor(processor), gate(processor),
      range("Range", 6), threshold("Threshold", 0.10), hold("Hold", 300),
      attack("Attack", 250), release("Release", 5), bias("Level Bias", 0) {
    setSize(980, 620);
    setLookAndFeel(&look);
    for (auto* knob : { &range, &threshold, &hold, &attack, &release, &bias }) addAndMakeVisible(knob);
    range.setRange(0, 6, 0.01); range.setValue(6); range.setTextValueSuffix(" dB");
    threshold.setRange(0, 1, 0.001);
    hold.setRange(0, 1000, 1); hold.setTextValueSuffix(" ms");
    attack.setRange(1, 1000, 1); attack.setTextValueSuffix(" ms");
    release.setRange(1, 200, 0.1); release.setTextValueSuffix(" ms");
    bias.setRange(0, 1, 0.01);
    aRange = std::make_unique<Attachment>(gate.state, "range", range);
    aThreshold = std::make_unique<Attachment>(gate.state, "threshold", threshold);
    aHold = std::make_unique<Attachment>(gate.state, "hold", hold);
    aAttack = std::make_unique<Attachment>(gate.state, "attack", attack);
    aRelease = std::make_unique<Attachment>(gate.state, "release", release);
    aBias = std::make_unique<Attachment>(gate.state, "bias", bias);
    addAndMakeVisible(bypass);
    aBypass = std::make_unique<juce::AudioProcessorValueTreeState::ButtonAttachment>(gate.state, "bypass", bypass);
    play.addListener(this); load.addListener(this);
    addAndMakeVisible(play); addAndMakeVisible(load);
    hover.setJustificationType(juce::Justification::centred);
    addAndMakeVisible(hover);
    startTimerHz(30);
}
GateEditor::~GateEditor() { setLookAndFeel(nullptr); }
void GateEditor::timerCallback() {
    GateKnob* hot = nullptr;
    for (auto* knob : { &range, &threshold, &hold, &attack, &release, &bias })
        if (knob->isMouseOverOrDragging()) hot = knob;
    hover.setText(hot ? hot->getName() + "  " + hot->getTextFromValue(hot->getValue()) : juce::String(), juce::dontSendNotification);
    repaint();
}
void GateEditor::buttonClicked(juce::Button* button) {
    if (button == &load) {
        auto chooser = std::make_shared<juce::FileChooser>("Recording", juce::File{}, "*.wav");
        chooser->launchAsync(juce::FileBrowserComponent::openMode | juce::FileBrowserComponent::canSelectFiles,
            [this, chooser](const juce::FileChooser& chosen) {
                if (chosen.getResult().existsAsFile()) gate.loadPreview(chosen.getResult());
            });
    } else {
        gate.playing = !gate.playing.load();
        play.setButtonText(gate.playing ? "Pause" : "Play");
    }
}
void GateEditor::paint(juce::Graphics& g) {
    g.fillAll(juce::Colour(0xff171a1e));
    auto plot = getLocalBounds().reduced(16).withTrimmedBottom(190).toFloat();
    g.setColour(juce::Colour(0xff20242a));
    g.fillRoundedRectangle(plot, 8);
    const juce::ScopedLock lock(gate.previewLock);
    if (gate.previewBuffer.getNumSamples() > 0) {
        juce::Path input;
        int samples = gate.previewBuffer.getNumSamples();
        int step = std::max(1, samples / (int) plot.getWidth());
        for (int x = 0; x < (int) plot.getWidth(); ++x) {
            float sample = gate.previewBuffer.getSample(0, std::min(samples - 1, x * step));
            float y = plot.getCentreY() - sample * plot.getHeight() * 0.35f;
            if (x == 0) input.startNewSubPath(plot.getX() + x, y); else input.lineTo(plot.getX() + x, y);
        }
        g.setColour(juce::Colour(0xff6e7680));
        g.strokePath(input, juce::PathStrokeType(1.3f));
    }
    juce::Path reduction;
    int count = (int) gate.reduction.size();
    int write = gate.reductionWrite.load();
    for (int i = 0; i < count; ++i) {
        float gain = gate.reduction[(size_t) ((write + i) % count)].load();
        float db = juce::Decibels::gainToDecibels(gain, -6.0f);
        float y = juce::jmap(db, 0.0f, -6.0f, plot.getY() + 12, plot.getBottom() - 12);
        float x = plot.getX() + plot.getWidth() * i / (float) count;
        if (i == 0) reduction.startNewSubPath(x, y); else reduction.lineTo(x, y);
    }
    g.setColour(juce::Colour(0xffff5a5a));
    g.strokePath(reduction, juce::PathStrokeType(2.0f));
    auto caption = plot.removeFromTop(24).reduced(12, 0);
    const bool bypassed = *gate.state.getRawParameterValue("bypass") > 0.5f;
    const int link = gate.link.load();
    juce::String status = bypassed ? "Bypassed" : (link == 2 ? "Failed" : (link == 1 ? "Armed" : "Waiting"));
    g.setColour(juce::Colour(0xff8b939c));
    g.drawText(status, caption.removeFromLeft(120), juce::Justification::left);
    const float activity = gate.activity.load();
    // A missing or failed detector has no score. Do not plot that as zero activity.
    if (!bypassed && link != 2 && activity >= 0.0f) {
        float y = juce::jmap(activity, 0.0f, 1.0f, plot.getBottom() - 12, plot.getY() + 28);
        g.setColour(juce::Colour(0xffffd36a));
        g.drawLine(plot.getX() + 12, y, plot.getRight() - 12, y, 1.5f);
        g.drawText("Activity", caption.removeFromLeft(90), juce::Justification::left);
    } else {
        g.drawText("Activity —", caption.removeFromLeft(110), juce::Justification::left);
    }
    g.drawText("Reduction", caption.removeFromLeft(90), juce::Justification::left);
}
void GateEditor::resized() {
    auto bottom = getLocalBounds().reduced(16).removeFromBottom(180);
    hover.setBounds(bottom.removeFromTop(22));
    auto buttons = bottom.removeFromBottom(32);
    bypass.setBounds(buttons.removeFromRight(110));
    play.setBounds(buttons.removeFromRight(80));
    load.setBounds(buttons.removeFromRight(140));
    int width = bottom.getWidth() / 6;
    range.setBounds(bottom.removeFromLeft(width));
    threshold.setBounds(bottom.removeFromLeft(width));
    hold.setBounds(bottom.removeFromLeft(width));
    attack.setBounds(bottom.removeFromLeft(width));
    release.setBounds(bottom.removeFromLeft(width));
    bias.setBounds(bottom);
}
