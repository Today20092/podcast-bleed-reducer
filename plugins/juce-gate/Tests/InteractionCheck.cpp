#include "../Source/PluginEditor.h"
#include "../Source/PluginProcessor.h"
#include <cmath>
#include <iostream>

int main() {
    juce::ScopedJuceInitialiser_GUI gui;
    GateKnob depth("Depth", 6.0);
    depth.setRange(0.0, 60.0, 0.01);
    depth.setTextValueSuffix(" dB");
    depth.setValue(6.0);
    depth.configureValueEntry();
    GateKnob bias("Level Bias", 0.0);
    bias.setRange(0.0, 1.0, 0.01);
    bias.configureValueEntry();
    const bool ok = depth.getValueFromText("-inf dB") == 60.0
        && depth.getValueFromText("12 dB") == 12.0
        && depth.getValueFromText("invalid") == 6.0
        && depth.getValueFromText("1000 dB") == 6.0
        && std::abs(depth.getValueFromText("50%") - 30.0) < 0.001
        && std::abs(bias.getValueFromText("50%") - 0.5) < 0.001
        && depth.getTextBoxPosition() != juce::Slider::NoTextBox;
    GateProcessor processor;
    GateEditor editor(processor);
    juce::TextButton* ab = nullptr;
    juce::TextButton* copy = nullptr;
    for (auto* child : editor.getChildren()) {
        if (auto* button = dynamic_cast<juce::TextButton*>(child)) {
            if (button->getButtonText() == "A") ab = button;
            if (button->getButtonText() == "Copy") copy = button;
        }
    }
    auto* range = processor.state.getParameter("range");
    bool abOk = ab != nullptr && copy != nullptr && range != nullptr;
    if (abOk) {
        range->setValueNotifyingHost(range->convertTo0to1(12.0f));
        juce::MessageManager::getInstance()->runDispatchLoopUntil(100);
        abOk = copy->isEnabled();
        ab->triggerClick();
        juce::MessageManager::getInstance()->runDispatchLoopUntil(100);
        abOk = abOk && std::abs(processor.state.getRawParameterValue("range")->load() - 6.0f) < 0.01f;
        ab->triggerClick();
        juce::MessageManager::getInstance()->runDispatchLoopUntil(100);
        abOk = abOk && std::abs(processor.state.getRawParameterValue("range")->load() - 12.0f) < 0.01f;
        abOk = abOk && processor.undo.undo();
        abOk = abOk && std::abs(processor.state.getRawParameterValue("range")->load() - 6.0f) < 0.01f;
    }
    if (!ok || !abOk) std::cerr << "Gate interaction check failed: entry=" << ok << " A/B=" << abOk << '\n';
    return ok && abOk ? 0 : 1;
}
