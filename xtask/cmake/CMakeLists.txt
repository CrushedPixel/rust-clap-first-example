# ==============================================================================
# Project Configuration
# ==============================================================================
cmake_minimum_required(VERSION 3.15) # TODO: can we lower this?

# Set project name and initialize project
set(PROJECT_NAME "ClapFirstRustPlugin" CACHE STRING "Say my name")
project(${PROJECT_NAME})

# ==============================================================================
# Platform and Compiler Settings
# ==============================================================================
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# macOS Audio Unit SDK requirements
if (APPLE)
    enable_language(OBJC)
    enable_language(OBJCXX)

    set(CMAKE_OSX_DEPLOYMENT_TARGET 11.0) # At least Big Sur is needed for MIDI 2.0 in CoreAudio
    
    # Build universal binary (Intel + Apple Silicon)
    set(CMAKE_OSX_ARCHITECTURES "x86_64;arm64")

    set(CMAKE_OBJC_VISIBILITY_PRESET hidden)
    set(CMAKE_OBJCXX_VISIBILITY_PRESET hidden)
    set(CMAKE_VISIBILITY_INLINES_HIDDEN ON)
endif ()

# Windows specific settings
if (WIN32)
    add_compile_definitions(_SILENCE_ALL_CXX17_DEPRECATION_WARNINGS)

    # Set MSVC runtime library to static linking
    if (MSVC)
        set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
        add_compile_options(/MP)  # Enable multi-processor compilation
    endif ()

    # Add required Windows libraries
    set(WINDOWS_LIBS
            ws2_32     # For network functionality
            userenv    # For user profile functions
            ntdll      # For NT API functions
    )
endif ()

# Always build static libraries unless explicitly specified otherwise
set(BUILD_SHARED_LIBS OFF CACHE BOOL "Never want shared if not specified")

# ==============================================================================
# 📦 Dependencies: CPM and CLAP Wrapper
# ==============================================================================
# Download CPM.cmake for dependency management
file(
        DOWNLOAD
        https://github.com/cpm-cmake/CPM.cmake/releases/download/v0.40.2/CPM.cmake
        ${CMAKE_CURRENT_BINARY_DIR}/cmake/CPM.cmake
        EXPECTED_HASH SHA256=C8CDC32C03816538CE22781ED72964DC864B2A34A310D3B7104812A5CA2D835D
)
include(${CMAKE_CURRENT_BINARY_DIR}/cmake/CPM.cmake)

# Configure CLAP wrapper options
set(BUILD_SHARED_LIBS OFF CACHE BOOL "Never want shared if not specified")
set(CLAP_WRAPPER_DOWNLOAD_DEPENDENCIES ON)
set(CLAP_WRAPPER_DONT_ADD_TARGETS ON CACHE BOOL "I'll targetize")

# Add CLAP wrapper package
CPMAddPackage(
        NAME clap-wrapper
        GITHUB_REPOSITORY "free-audio/clap-wrapper"
        GIT_TAG "main"
)

# ==============================================================================
# Static Library Configuration
# ==============================================================================
# Setup for pre-built static library with rust_clap_entry symbol
# These variables can be overridden via command line or parent scope
set(STATIC_LIB_FILE "" CACHE FILEPATH "Full path to the static library file")
set(STATIC_LIB_INCLUDE_DIR "" CACHE PATH "Optional path to the include directory for the static library")
set(BUNDLE_ID "org.free-audio.clap-plugin" CACHE STRING "Bundle identifier for the plugin")
set(PLUGIN_OUTPUT_DIR "${CMAKE_BINARY_DIR}/plugins" CACHE PATH "Output directory to place the built plugins in")
set(INSTALL_PLUGINS_AFTER_BUILD OFF CACHE BOOL "Whether to install resulting plugins")

# Validate required variables
if (NOT STATIC_LIB_FILE)
    message(FATAL_ERROR "STATIC_LIB_FILE must be specified")
endif ()

if (NOT EXISTS "${STATIC_LIB_FILE}")
    message(FATAL_ERROR "Static library file does not exist: ${STATIC_LIB_FILE}")
endif ()

# Create an imported target for the pre-built static library
add_library(rust_static_lib STATIC IMPORTED)
set_target_properties(rust_static_lib PROPERTIES
        IMPORTED_LOCATION "${STATIC_LIB_FILE}"
)

# ==============================================================================
# Plugin Target Configuration
# ==============================================================================
# Create a CLAP library target that re-exposes the Rust static library's CLAP entry
add_library(clap_entry STATIC "clap_entry.cpp")
target_link_libraries(clap_entry PRIVATE rust_static_lib)

# Link required platform-specific libraries
if (APPLE)
    target_link_libraries(clap_entry PUBLIC
            "-framework Foundation"
            "-framework CoreFoundation"
            "-framework AppKit"
            "-framework AudioToolbox"
            "-framework Quartz"
            "-framework CoreAudio"
            "-framework AudioUnit"
            "-framework CoreMIDI"
    )
elseif (WIN32)
    target_link_libraries(clap_entry PUBLIC ${WINDOWS_LIBS})
endif ()

# ==============================================================================
# Plugin Build Configuration
# ==============================================================================
# Use clap-wrapper's make_clapfirst_plugin to build all plugin formats,
# including the CLAP, from the provided clap_entry implementation

make_clapfirst_plugins(
        TARGET_NAME ${PROJECT_NAME}
        IMPL_TARGET clap_entry

        OUTPUT_NAME "${PROJECT_NAME}"

        ENTRY_SOURCE "clap_entry.cpp"

        BUNDLE_IDENTIFER "${BUNDLE_ID}"
        BUNDLE_VERSION ${PROJECT_VERSION}

        COPY_AFTER_BUILD ${INSTALL_PLUGINS_AFTER_BUILD}

        PLUGIN_FORMATS CLAP VST3 AUV2

        ASSET_OUTPUT_DIRECTORY ${PLUGIN_OUTPUT_DIR}
)