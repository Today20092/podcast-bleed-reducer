#include "PluginEditor.h"
#include "PluginProcessor.h"
#include <cstdlib>

namespace {
const juce::Colour bg { 0xff121418 };
const juce::Colour plotBg { 0xff1a1d22 };
const juce::Colour grid { 0xff2a2e34 };
const juce::Colour gold { 0xfff2b030 };
const juce::Colour inputInk { 0xff5c656f };
const juce::Colour outputInk { 0xffd5dde6 };
const juce::Colour reductionInk { 0xffff4d4d };
const float kFullScaleDb = 60.0f;

float reductionDb(float gain) {
    if (!(gain > 1.0e-8f)) return kFullScaleDb;
    return juce::jlimit(0.0f, kFullScaleDb, -juce::Decibels::gainToDecibels(gain, -kFullScaleDb));
}

juce::String reductionText(float db) {
    if (db >= kFullScaleDb - 0.05f) return "-inf dB";
    if (db < 0.05f) return "0.0 dB";
    return "-" + juce::String(db, 1) + " dB";
}
}

void GateLook::drawRotarySlider(juce::Graphics& g, int x, int y, int width, int height, float position,
    float start, float end, juce::Slider& slider) {
    auto bounds = juce::Rectangle<float>((float) x, (float) y, (float) width, (float) height).reduced(8, 8);
    auto radius = juce::jmin(bounds.getWidth(), bounds.getHeight()) / 2.0f;
    auto centre = bounds.getCentre();
    auto angle = start + position * (end - start);
    const bool hot = slider.isMouseOverOrDragging();
    for (int i = 0; i <= 10; ++i) {
        const float tickAngle = start + (end - start) * (float) i / 10.0f;
        const auto inner = centre.getPointOnCircumference(radius + 2, tickAngle);
        const auto outer = centre.getPointOnCircumference(radius + 5, tickAngle);
        g.setColour(juce::Colour(0xff65707c));
        g.drawLine({ inner, outer }, 1.0f);
    }
    juce::ColourGradient cap(juce::Colour(0xff3a3e48), centre.x, centre.y - radius,
        juce::Colour(0xff16181c), centre.x, centre.y + radius, false);
    g.setGradientFill(cap);
    g.fillEllipse(centre.x - radius, centre.y - radius, radius * 2, radius * 2);
    g.setColour(juce::Colour(0xff0c0d10));
    g.drawEllipse(centre.x - radius, centre.y - radius, radius * 2, radius * 2, 1.2f);
    g.setColour(hot ? juce::Colour(0xff6a5424) : juce::Colour(0xff2c3036));
    juce::Path track;
    track.addCentredArc(centre.x, centre.y, radius - 3, radius - 3, 0, start, end, true);
    g.strokePath(track, juce::PathStrokeType(2.4f, juce::PathStrokeType::curved, juce::PathStrokeType::rounded));
    g.setColour(hot ? juce::Colour(0xffffe08a) : gold);
    juce::Path value;
    value.addCentredArc(centre.x, centre.y, radius - 3, radius - 3, 0, start, angle, true);
    g.strokePath(value, juce::PathStrokeType(2.8f, juce::PathStrokeType::curved, juce::PathStrokeType::rounded));
    auto tip = centre.getPointOnCircumference(radius - 7, angle);
    auto base = centre.getPointOnCircumference(radius * 0.43f, angle);
    g.setColour(juce::Colours::white.withAlpha(hot ? 1.0f : 0.85f));
    g.drawLine({ base, tip }, 2.5f);
}

void GateLook::drawButtonBackground(juce::Graphics& g, juce::Button& button, const juce::Colour&, bool over, bool down) {
    auto r = button.getLocalBounds().toFloat().reduced(1);
    g.setColour(down ? juce::Colour(0xff2c3238) : (over ? juce::Colour(0xff262b31) : juce::Colour(0xff1c2025)));
    g.fillRoundedRectangle(r, 4);
    if (!button.isEnabled()) g.setColour(juce::Colour(0xff3a4046));
    else if (button.getToggleState()) g.setColour(gold);
    else g.setColour(juce::Colour(0xff4a515a));
    g.drawRoundedRectangle(r, 4, 1);
}

juce::Label* GateLook::createSliderTextBox(juce::Slider& slider) {
    auto* label = juce::LookAndFeel_V4::createSliderTextBox(slider);
    label->setFont(juce::Font(12.0f));
    return label;
}

GateKnob::GateKnob(const juce::String& name, double reset) : resetValue(reset) {
    setSliderStyle(RotaryHorizontalVerticalDrag);
    setTextBoxStyle(TextBoxBelow, false, 72, 18);
    setWantsKeyboardFocus(true);
    setRotaryParameters(juce::MathConstants<float>::pi * 1.25f, juce::MathConstants<float>::pi * 2.75f, true);
    setVelocityBasedMode(false);
    setMouseDragSensitivity(220);
    setName(name);
    setDoubleClickReturnValue(false, reset);
    setPopupDisplayEnabled(false, false, nullptr);
}
void GateKnob::configureValueEntry() {
    valueFromTextFunction = [this](const juce::String& input) {
        auto text = input.trim();
        if (getName() == "Depth" && (text.equalsIgnoreCase("-inf") || text.equalsIgnoreCase("-inf dB")))
            return getMaximum();
        auto utf8 = text.toStdString();
        char* end = nullptr;
        const double number = std::strtod(utf8.c_str(), &end);
        if (end == utf8.c_str() || !std::isfinite(number)) return getValue();
        const auto suffix = juce::String::fromUTF8(end).trim();
        double value = number;
        if (suffix == "%") value = getMinimum() + (getMaximum() - getMinimum()) * number / 100.0;
        else if (getName() == "Level Bias" && suffix.isEmpty()) value = number / 100.0;
        else if (suffix.isNotEmpty() && !suffix.equalsIgnoreCase(getTextValueSuffix().trim())) return getValue();
        return value >= getMinimum() && value <= getMaximum() ? value : getValue();
    };
}
void GateKnob::mouseDown(const juce::MouseEvent& e) {
    if (e.mods.isCtrlDown() || e.mods.isCommandDown()) {
        ScopedDragNotification gesture(*this);
        setValue(resetValue, juce::sendNotificationSync);
        return;
    }
    setMouseDragSensitivity(e.mods.isShiftDown() ? 900 : 220);
    Slider::mouseDown(e);
}
void GateKnob::mouseDoubleClick(const juce::MouseEvent&) { showTextBox(); }
void GateKnob::mouseDrag(const juce::MouseEvent& e) {
    setMouseDragSensitivity(e.mods.isShiftDown() ? 900 : 220);
    Slider::mouseDrag(e);
    const double legal = juce::jlimit(getMinimum(), getMaximum(), getValue());
    if (legal != getValue()) setValue(legal, juce::sendNotificationSync);
}
void GateKnob::mouseWheelMove(const juce::MouseEvent& e, const juce::MouseWheelDetails& wheel) {
    if (!e.mods.isShiftDown() || e.mods.isAnyMouseButtonDown()) {
        fineWheelActive = false;
        Slider::mouseWheelMove(e, wheel);
        return;
    }
    if (!fineWheelActive || fineWheelValue != getValue())
        fineWheelPosition = valueToProportionOfLength(getValue());
    const float delta = (std::abs(wheel.deltaX) > std::abs(wheel.deltaY) ? -wheel.deltaX : wheel.deltaY)
        * (wheel.isReversed ? -1.0f : 1.0f);
    fineWheelPosition = juce::jlimit(0.0, 1.0, fineWheelPosition + (double) delta * 0.015);
    {
        ScopedDragNotification gesture(*this);
        setValue(proportionOfLengthToValue(fineWheelPosition), juce::sendNotificationSync);
    }
    fineWheelValue = getValue();
    fineWheelActive = true;
    const double legal = juce::jlimit(getMinimum(), getMaximum(), getValue());
    if (legal != getValue()) setValue(legal, juce::sendNotificationSync);
}

GateEditor::GateEditor(GateProcessor& processor)
    : AudioProcessorEditor(processor), gate(processor),
      range("Depth", 6), threshold("Voice", 0.10), hold("Hold", 300),
      attack("Attack", 5), release("Release", 250), lookahead("Lookahead", 5), bias("Level Bias", 0) {
    setLookAndFeel(&look);
    setResizable(true, true);
    setResizeLimits(760, 480, 1800, 1100);
    setSize(980, 640);
    for (auto* knob : { &range, &threshold, &hold, &attack, &release, &lookahead, &bias }) {
        knob->setLookAndFeel(&look);
        addAndMakeVisible(knob);
    }
    range.setRange(0, 60, 0.01);
    threshold.setRange(0, 1, 0.001);
    hold.setRange(0, 1000, 1); hold.setTextValueSuffix(" ms");
    attack.setRange(1, 1000, 1); attack.setTextValueSuffix(" ms");
    release.setRange(1, 1000, 1); release.setTextValueSuffix(" ms");
    lookahead.setRange(0, 20, 0.1); lookahead.setTextValueSuffix(" ms");
    bias.setRange(0, 1, 0.01);
    aRange = std::make_unique<Attachment>(gate.state, "range", range);
    aThreshold = std::make_unique<Attachment>(gate.state, "threshold", threshold);
    aHold = std::make_unique<Attachment>(gate.state, "hold", hold);
    aAttack = std::make_unique<Attachment>(gate.state, "attack", attack);
    aRelease = std::make_unique<Attachment>(gate.state, "release", release);
    aLookahead = std::make_unique<Attachment>(gate.state, "lookahead", lookahead);
    aBias = std::make_unique<Attachment>(gate.state, "bias", bias);
    for (auto* knob : { &range, &threshold, &hold, &attack, &release, &lookahead, &bias }) knob->configureValueEntry();
    range.setTextValueSuffix(" dB");
    range.textFromValueFunction = [](double value) { return value >= 60.0 ? juce::String("-inf") : juce::String(value, 1); };
    bias.setTextValueSuffix("%");
    bias.textFromValueFunction = [](double value) { return juce::String(juce::roundToInt(value * 100.0)); };
    auto name = [](juce::Label& label, const juce::String& text) {
        label.setText(text, juce::dontSendNotification);
        label.setJustificationType(juce::Justification::centred);
        label.setColour(juce::Label::textColourId, juce::Colour(0xffd5dee8));
        label.setFont(juce::Font(12.0f));
        label.setMinimumHorizontalScale(1.0f);
    };
    name(rangeName, "Depth"); name(thresholdName, "Voice"); name(holdName, "Hold");
    name(attackName, "Attack"); name(releaseName, "Release"); name(lookaheadName, "Lookahead"); name(biasName, "Level Bias");
    for (auto* label : { &rangeName, &thresholdName, &holdName, &attackName, &releaseName, &lookaheadName, &biasName })
        addAndMakeVisible(label);
    bypass.setClickingTogglesState(true);
    for (auto* button : { &undoBtn, &redoBtn, &abBtn, &copyBtn, &displayBtn, &bypass }) {
        button->addListener(this);
        button->setLookAndFeel(&look);
        addAndMakeVisible(button);
    }
    aBypass = std::make_unique<juce::AudioProcessorValueTreeState::ButtonAttachment>(gate.state, "bypass", bypass);
    for (auto* knob : { &hold, &attack, &release, &lookahead, &bias }) knob->setVisible(false);
    for (auto* label : { &holdName, &attackName, &releaseName, &lookaheadName, &biasName }) label->setVisible(false);
    slotA = gate.state.copyState();
    slotB = slotA.createCopy();
    startTimerHz(60);
}
GateEditor::~GateEditor() {
    setLookAndFeel(nullptr);
    for (auto* knob : { &range, &threshold, &hold, &attack, &release, &lookahead, &bias }) knob->setLookAndFeel(nullptr);
}

void GateEditor::storeActive() {
    auto tree = gate.state.copyState();
    if (showingA) slotA = tree; else slotB = tree;
}

void GateEditor::timerCallback() {
    undoBtn.setEnabled(gate.undo.canUndo());
    redoBtn.setEnabled(gate.undo.canRedo());
    const bool same = gate.state.copyState().isEquivalentTo(showingA ? slotB : slotA);
    copyBtn.setEnabled(!same);
    abBtn.setButtonText(showingA ? "A" : "B");
    copyBtn.setButtonText(showingA ? "A to B" : "B to A");
    displayBtn.setButtonText(controlsOpen ? "Less" : "More");
    peakGr = reductionDb(gate.minimumGain.load());
    auto ease = [](float& shown, float target) { shown += (target - shown) * 0.35f; };
    ease(shownThreshold, gate.state.getRawParameterValue("threshold")->load());
    ease(shownRange, gate.state.getRawParameterValue("range")->load());
    ease(shownHold, gate.state.getRawParameterValue("hold")->load());
    ease(shownAttack, gate.state.getRawParameterValue("attack")->load());
    ease(shownRelease, gate.state.getRawParameterValue("release")->load());
    ease(shownBias, gate.state.getRawParameterValue("bias")->load());
    const int width = juce::jmax(8, (int) speechBounds.getWidth());
    if (pen.empty())
        pen.assign((size_t) width, { -1.0f, 1.0f });
    else if ((int) pen.size() > width)
        pen.erase(pen.begin(), pen.begin() + ((int) pen.size() - width));
    else if ((int) pen.size() < width)
        pen.insert(pen.begin(), (size_t) (width - (int) pen.size()), { -1.0f, 1.0f });
    const int total = gate.columnCount.load();
    if (total == columnsSeen) {
        repaint();
        return;
    }
    std::rotate(pen.begin(), pen.begin() + 1, pen.end());
    Column latest { -1.0f, 1.0f };
    const int first = juce::jmax(columnsSeen, total - GateProcessor::waveCapacity);
    for (int i = first; i < total; ++i) {
        const auto slot = (size_t) (i % GateProcessor::waveCapacity);
        latest.activity = juce::jmax(latest.activity, gate.waveActivity[slot].load());
        latest.gain = juce::jmin(latest.gain, gate.waveGain[slot].load());
    }
    pen.back() = latest;
    columnsSeen = total;
    repaint();
}

void GateEditor::buttonClicked(juce::Button* button) {
    if (button == &displayBtn) {
        controlsOpen = !controlsOpen;
        displayBtn.setToggleState(controlsOpen, juce::dontSendNotification);
        for (auto* knob : { &hold, &attack, &release, &lookahead, &bias }) knob->setVisible(controlsOpen);
        for (auto* label : { &holdName, &attackName, &releaseName, &lookaheadName, &biasName }) label->setVisible(controlsOpen);
        resized();
        repaint();
        return;
    }
    if (button == &undoBtn) gate.undo.undo();
    else if (button == &redoBtn) gate.undo.redo();
    else if (button == &abBtn) {
        storeActive();
        showingA = !showingA;
        const auto target = showingA ? slotA : slotB;
        gate.undo.beginNewTransaction("Switch A/B");
        for (const auto& child : target) {
            const auto id = child.getProperty("id").toString();
            auto current = gate.state.state.getChildWithProperty("id", id);
            if (!current.isValid() || !child.hasProperty("value")) continue;
            auto* parameter = gate.state.getParameter(id);
            if (parameter != nullptr) parameter->beginChangeGesture();
            current.setProperty("value", child.getProperty("value"), &gate.undo);
            if (parameter != nullptr) parameter->endChangeGesture();
        }
    } else if (button == &copyBtn) {
        storeActive();
        if (showingA) slotB = slotA.createCopy(); else slotA = slotB.createCopy();
    }
}

void GateEditor::layoutPlotRegions() {
    auto history = plotBounds.toFloat().reduced(20.0f, 8.0f);
    history.removeFromTop(44.0f);
    auto speech = history.removeFromTop(history.getHeight() * 0.52f);
    speech.removeFromBottom(6.0f);
    history.removeFromTop(18.0f);
    speechBounds = speech;
    reductionBounds = history;
}

void GateEditor::setVoiceFromY(float y) {
    if (speechBounds.isEmpty()) return;
    const float score = juce::jlimit(0.0f, 1.0f, juce::jmap(y, speechBounds.getBottom(), speechBounds.getY(), 0.0f, 1.0f));
    threshold.setValue(score, juce::sendNotificationSync);
}

void GateEditor::mouseDown(const juce::MouseEvent& e) {
    if (speechBounds.contains(e.position)) {
        if (auto* parameter = gate.state.getParameter("threshold")) parameter->beginChangeGesture();
        dragVoice = true;
        lastVoiceY = e.position.y;
        setVoiceFromY(e.position.y);
        return;
    }
}

void GateEditor::mouseDrag(const juce::MouseEvent& e) {
    if (dragVoice) {
        if (speechBounds.getHeight() > 0.0f) {
            const float scale = e.mods.isShiftDown() ? 10.0f : 1.0f;
            const auto value = threshold.getValue() + (lastVoiceY - e.position.y) / (speechBounds.getHeight() * scale);
            threshold.setValue(value, juce::sendNotificationSync);
        }
        lastVoiceY = e.position.y;
    }
}

void GateEditor::mouseUp(const juce::MouseEvent&) {
    if (!dragVoice) return;
    dragVoice = false;
    if (auto* parameter = gate.state.getParameter("threshold")) parameter->endChangeGesture();
}

bool GateEditor::keyPressed(const juce::KeyPress& key) {
    const auto mods = key.getModifiers();
    if (key == juce::KeyPress::escapeKey && controlsOpen) {
        buttonClicked(&displayBtn);
        return true;
    }
    if (mods.isCommandDown() || mods.isCtrlDown()) {
        if (key.getKeyCode() == 'z' || key.getKeyCode() == 'Z') {
            if (mods.isShiftDown()) gate.undo.redo(); else gate.undo.undo();
            return true;
        }
        if (key.getKeyCode() == 'y' || key.getKeyCode() == 'Y') {
            gate.undo.redo();
            return true;
        }
    }
    return AudioProcessorEditor::keyPressed(key);
}

void GateEditor::paint(juce::Graphics& g) {
    g.fillAll(bg);
    g.setColour(juce::Colour(0xff8b939c));
    g.setFont(11);
    if (controlsOpen) {
        const int headingY = attack.getY() - 16;
        g.drawText("Timing", attack.getX(), headingY, lookahead.getRight() - attack.getX(), 14, juce::Justification::centred);
    }
    auto top = getLocalBounds().removeFromTop(36);
    g.setColour(juce::Colour(0xff0e1013));
    g.fillRect(top);
    g.setColour(gold);
    g.setFont(juce::Font(16, juce::Font::bold));
    g.drawText("Smart Gate", 16, 8, 88, 24, juce::Justification::left);
    g.setColour(juce::Colour(0xff8b939c));
    g.setFont(12.0f);
    g.drawText("0.1.0", 104, 12, 36, 16, juce::Justification::left);

    auto plot = plotBounds.toFloat();
    g.setColour(plotBg);
    g.fillRoundedRectangle(plot, 8);
    const bool bypassed = *gate.state.getRawParameterValue("bypass") > 0.5f;
    const auto speech = speechBounds;
    const auto reductionArea = reductionBounds;
    g.setColour(grid);
    g.drawHorizontalLine((int) speech.getBottom(), speech.getX(), speech.getRight());
    g.drawHorizontalLine((int) reductionArea.getY(), reductionArea.getX(), reductionArea.getRight());
    g.setFont(11.0f);
    g.setColour(juce::Colour(0xff8b939c));
    const float legendY = plot.getY() + 26.0f;
    g.drawText("Speech", speech.getX(), legendY, 48, 14, juce::Justification::left);
    auto swatch = juce::Rectangle<float>(speech.getX() + 52.0f, legendY + 3.0f, 8.0f, 8.0f);
    g.setColour(juce::Colour(0xff76d4ae));
    g.fillRect(swatch);
    g.setColour(juce::Colour(0xff8b939c));
    g.drawText("Open", juce::Rectangle<float>(swatch.getRight() + 4.0f, legendY, 36.0f, 14.0f), juce::Justification::left);
    auto dash = swatch.translated(52.0f, 0.0f);
    g.setColour(juce::Colour(0xff72818e));
    g.fillRect(dash.withWidth(3.0f));
    g.fillRect(dash.translated(5.0f, 0.0f).withWidth(3.0f));
    g.setColour(juce::Colour(0xff8b939c));
    g.drawText("Ducked", juce::Rectangle<float>(dash.getRight() + 6.0f, legendY, 56.0f, 14.0f), juce::Justification::left);
    g.drawText("0", speech.getX() - 18, speech.getBottom() - 12, 16, 12, juce::Justification::right);
    g.drawText("1", speech.getX() - 18, speech.getY() - 1, 16, 12, juce::Justification::right);
    g.drawText("Reduction", reductionArea.getX(), reductionArea.getY() - 16, 80, 14, juce::Justification::left);
    g.drawText("0 to -60 dB", reductionArea.getRight() - 88, reductionArea.getY() - 16, 88, 14, juce::Justification::right);
    const float thresholdY = juce::jmap((float) threshold.getValue(), 0.0f, 1.0f, speech.getBottom(), speech.getY());
    const float depthY = reductionArea.getY() + reductionArea.getHeight() * ((float) range.getValue() / kFullScaleDb);
    const int columns = (int) pen.size();
    const float columnWidth = speech.getWidth() / (float) juce::jmax(1, columns);
    for (int i = 0; i < columns; ++i) {
        const float x = speech.getX() + columnWidth * (float) i;
        const auto& point = pen[(size_t) i];
        if (point.activity >= 0.0f) {
            const float y = juce::jmap(juce::jlimit(0.0f, 1.0f, point.activity), 0.0f, 1.0f, speech.getBottom(), speech.getY());
            const bool above = point.activity >= (float) threshold.getValue();
            const bool ducking = reductionDb(point.gain) > 0.5f;
            const bool openColumn = above || !ducking;
            if (openColumn || (i & 1) == 0) {
                g.setColour(openColumn ? juce::Colour(0xff76d4ae) : juce::Colour(0xff72818e));
                g.fillRect(x, y, juce::jmax(1.0f, columnWidth), speech.getBottom() - y);
            }
        }
        const float amount = reductionDb(point.gain) / kFullScaleDb;
        g.setColour(reductionInk.withAlpha(0.85f));
        g.fillRect(x, reductionArea.getY(), juce::jmax(1.0f, columnWidth), reductionArea.getHeight() * amount);
    }
    g.setColour(gold.withAlpha(0.95f));
    g.drawHorizontalLine((int) thresholdY, speech.getX(), speech.getRight());
    if (dragVoice || threshold.isMouseButtonDown())
        g.drawText(juce::String(threshold.getValue(), 2), speech.getRight() - 44, thresholdY - 14, 44, 14, juce::Justification::right);
    g.drawHorizontalLine((int) depthY, reductionArea.getX(), reductionArea.getRight());
    if (range.isMouseButtonDown())
        g.drawText(range.getTextFromValue(range.getValue()), reductionArea.getRight() - 72, depthY - 14, 72, 14, juce::Justification::right);
    if (!bypassed && (pen.empty() || pen.back().activity < 0.0f)) {
        g.setColour(juce::Colour(0xffc6cdd5));
        const int linkNow = gate.link.load();
        const char* waiting = linkNow == 3 ? "Use 44.1, 48, or 96 kHz"
            : linkNow == 2 ? "Detector unavailable" : "Waiting for speech";
        g.drawText(waiting, speech.toNearestInt(), juce::Justification::centred);
    }
    const int link = gate.link.load();
    const float cut = pen.empty() ? 0.0f : reductionDb(pen.back().gain);
    const float depth = (float) range.getValue();
    juce::String status;
    if (bypassed) status = "Bypassed";
    else if (link == 1 && !pen.empty() && pen.back().activity >= 0.0f) {
        if (cut < 0.5f || depth < 0.5f) status = "Open";
        else if (cut >= depth - 0.6f) status = "Closed";
        else status = "Moving";
        status += "   " + juce::String(pen.back().activity, 2);
    }
    if (bypassed) {
        g.setColour(juce::Colours::black.withAlpha(0.45f));
        g.fillRoundedRectangle(plot, 8);
        g.setColour(reductionInk);
        g.fillRect(bypass.getX(), bypass.getY(), bypass.getWidth(), 2);
    }
    if (status.isNotEmpty()) {
        g.setColour(juce::Colour(0xffd5dee8));
        g.setFont(13);
        g.drawText(status, plot.getX() + 16, plot.getY() + 6, 160, 16, juce::Justification::left);
    }

    auto meter = meterBounds.toFloat();
    auto readout = meter.removeFromTop(28.0f);
    g.setColour(juce::Colour(0xffd5dee8));
    g.setFont(11);
    g.drawText(reductionText(peakGr), readout, juce::Justification::centred);
    g.setColour(juce::Colour(0xff101216));
    g.fillRoundedRectangle(meter, 4);
    const float amount = juce::jlimit(0.0f, 1.0f, peakGr / kFullScaleDb);
    g.setColour(reductionInk);
    auto filled = meter.withTrimmedTop(meter.getHeight() * (1.0f - amount));
    g.fillRect(filled.reduced(3, 3));
    g.setColour(juce::Colour(0xff8b939c));
    g.drawText("GR", meterBounds.translated(0, meterBounds.getHeight() + 2).withHeight(14), juce::Justification::centred);

    GateKnob* hot = nullptr;
    for (auto* knob : { &range, &threshold, &hold, &attack, &release, &lookahead, &bias })
        if (knob->isMouseOverOrDragging()) hot = knob;
    if (hot != nullptr) {
        juce::String hint = "How far a closed gate ducks";
        if (hot == &threshold) hint = "Speech score required to open";
        else if (hot == &hold) hint = "Stays open after the voice";
        else if (hot == &attack) hint = "How fast the gate opens for speech";
        else if (hot == &release) hint = "How fast the gate closes after hold";
        else if (hot == &lookahead) hint = "Keeps the start of each word";
        else if (hot == &bias) hint = "Speech at 0%, loudness at 100%";
        auto card = juce::Rectangle<int>(0, 0, 280, 24);
        card.setCentre(hot->getBounds().getCentreX(), hot->getY() - 16);
        card.setX(juce::jlimit(8, getWidth() - card.getWidth() - 8, card.getX()));
        card.setY(juce::jmax(40, card.getY()));
        g.setColour(juce::Colour(0xf0181b20));
        g.fillRoundedRectangle(card.toFloat(), 6);
        g.setColour(juce::Colour(0xff3a4048));
        g.drawRoundedRectangle(card.toFloat(), 6, 1);
        g.setColour(juce::Colour(0xffa8b0b8));
        g.setFont(11);
        g.drawText(hint, card.reduced(8, 0), juce::Justification::centred);
    }
    g.setColour(juce::Colour(0xff8b939c));
    g.setFont(11.0f);
    g.drawText("Shift for fine changes. Ctrl-click resets. Double-click to type.", shortcutBounds, juce::Justification::centred);
}

void GateEditor::resized() {
    auto area = getLocalBounds();
    auto top = area.removeFromTop(40).reduced(144, 8);
    undoBtn.setBounds(top.removeFromLeft(64));
    top.removeFromLeft(8);
    redoBtn.setBounds(top.removeFromLeft(64));
    top.removeFromLeft(8);
    abBtn.setBounds(top.removeFromLeft(40));
    top.removeFromLeft(8);
    copyBtn.setBounds(top.removeFromLeft(80));
    top.removeFromLeft(8);
    displayBtn.setBounds(top.removeFromLeft(72));
    bypass.setBounds(getWidth() - 96, 8, 80, 24);
    const int gap = 16;
    const int valueH = 18;
    const int unit = juce::jlimit(96, 144, (getWidth() - 48 - gap * 2) / 7);
    const int bottomHeight = 16 + 12 + (unit + valueH) + 20;
    auto bottom = area.removeFromBottom(bottomHeight).reduced(16, 0);
    shortcutBounds = bottom.removeFromTop(16);
    bottom.removeFromTop(12);
    auto names = bottom.removeFromBottom(20);
    const int row = unit * 7 + gap * 2;
    const int lead = juce::jmax(0, bottom.getWidth() - row) / 2;
    bottom.removeFromLeft(lead);
    names.removeFromLeft(lead);
    auto place = [&](GateKnob& knob, juce::Label& label) {
        knob.setTextBoxStyle(juce::Slider::TextBoxBelow, false, juce::jmax(72, unit - 16), valueH);
        knob.setBounds(bottom.removeFromLeft(unit));
        label.setBounds(names.removeFromLeft(unit));
    };
    place(threshold, thresholdName);
    place(range, rangeName);
    bottom.removeFromLeft(gap); names.removeFromLeft(gap);
    place(attack, attackName);
    place(hold, holdName);
    place(release, releaseName);
    place(lookahead, lookaheadName);
    place(bias, biasName);
    meterBounds = area.removeFromRight(96).reduced(8, 16);
    plotBounds = area.reduced(16, 8);
    layoutPlotRegions();
}
