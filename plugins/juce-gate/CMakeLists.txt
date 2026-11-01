cmake_minimum_required(VERSION 3.22)
project(CleanSlateSmartGateJuce VERSION 0.1.0)
set(CMAKE_CXX_STANDARD 20)
include(FetchContent)
FetchContent_Declare(JUCE GIT_REPOSITORY https://github.com/juce-framework/JUCE.git GIT_TAG 8.0.8 GIT_SHALLOW TRUE)
FetchContent_MakeAvailable(JUCE)
juce_add_plugin(CleanSlateSmartGate
    COMPANY_NAME CleanSlate
    PLUGIN_MANUFACTURER_CODE CsLt
    PLUGIN_CODE CsSg
    FORMATS VST3
    PRODUCT_NAME "CleanSlate Smart Gate"
    IS_SYNTH FALSE
    NEEDS_MIDI_INPUT FALSE
    NEEDS_MIDI_OUTPUT FALSE
    EDITOR_WANTS_KEYBOARD_FOCUS FALSE)
target_sources(CleanSlateSmartGate PRIVATE Source/PluginProcessor.cpp Source/PluginEditor.cpp)
target_compile_definitions(CleanSlateSmartGate PUBLIC JUCE_WEB_BROWSER=0 JUCE_USE_CURL=0 JUCE_VST3_CAN_REPLACE_VST2=0)
target_link_libraries(CleanSlateSmartGate PRIVATE
    juce::juce_audio_utils
    juce::juce_audio_formats
    juce::juce_audio_processors
    juce::juce_gui_extra
    PUBLIC juce::juce_recommended_config_flags juce::juce_recommended_warning_flags)

juce_add_console_app(CleanSlateGateInteractionCheck PRODUCT_NAME "CleanSlate Gate Interaction Check")
target_sources(CleanSlateGateInteractionCheck PRIVATE Tests/InteractionCheck.cpp)
target_compile_definitions(CleanSlateGateInteractionCheck PRIVATE JUCE_MODAL_LOOPS_PERMITTED=1)
target_link_libraries(CleanSlateGateInteractionCheck PRIVATE CleanSlateSmartGate juce::juce_audio_utils juce::juce_audio_formats juce::juce_audio_processors juce::juce_gui_extra)
