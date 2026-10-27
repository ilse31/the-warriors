cmake_minimum_required(VERSION 3.21)
project(WarriorsRuntime LANGUAGES C CXX)

set(PS2X_BUILD_RECOMP OFF CACHE BOOL "" FORCE)
set(PS2X_BUILD_ANALYZER OFF CACHE BOOL "" FORCE)
set(PS2X_BUILD_TEST OFF CACHE BOOL "" FORCE)
set(PS2X_BUILD_STUDIO OFF CACHE BOOL "" FORCE)
set(PS2X_ENABLE_DEBUG_UI OFF CACHE BOOL "" FORCE)
set(PS2X_ENABLE_AGRESSIVE_LOGS OFF CACHE BOOL "" FORCE)
set(PS2X_DEFAULT_BOOT_ELF "data/SLUS_212.15" CACHE STRING "" FORCE)

get_filename_component(WARRIORS_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/.." ABSOLUTE)
set(WARRIORS_GENERATED "${WARRIORS_ROOT}/analysis/ghidra/output" CACHE PATH "Generated game C++ directory")
if(NOT EXISTS "${WARRIORS_GENERATED}/register_functions.cpp")
    message(FATAL_ERROR "Run the Ghidra-map recompilation before building the runtime")
endif()

add_subdirectory("${WARRIORS_ROOT}/tools/PS2Recomp" ps2recomp)
get_target_property(WARRIORS_RUNNER_SOURCES ps2EntryRunner SOURCES)
list(REMOVE_ITEM WARRIORS_RUNNER_SOURCES "${WARRIORS_ROOT}/tools/PS2Recomp/ps2xRuntime/src/runner/register_functions.cpp")
list(REMOVE_ITEM WARRIORS_RUNNER_SOURCES "${WARRIORS_ROOT}/tools/PS2Recomp/ps2xRuntime/src/main.cpp")
list(APPEND WARRIORS_RUNNER_SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/main.cpp")
set_property(TARGET ps2EntryRunner PROPERTY SOURCES "${WARRIORS_RUNNER_SOURCES}")
file(GLOB WARRIORS_SOURCES CONFIGURE_DEPENDS "${WARRIORS_GENERATED}/*.cpp")
target_sources(ps2EntryRunner PRIVATE ${WARRIORS_SOURCES})
target_include_directories(ps2EntryRunner PRIVATE "${WARRIORS_GENERATED}")
target_precompile_headers(ps2EntryRunner PRIVATE "${WARRIORS_GENERATED}/ps2_recompiled_functions.h")
if(MSVC)
    foreach(target ps2_runtime ps2EntryRunner)
        get_target_property(options ${target} COMPILE_OPTIONS)
        string(REPLACE "/Qspectre-" "" options "${options}")
        string(REPLACE "/GL" "" options "${options}")
        string(REPLACE "/GS-" "" options "${options}")
        set_property(TARGET ${target} PROPERTY COMPILE_OPTIONS "${options}")
        get_target_property(link_options ${target} LINK_OPTIONS)
        string(REPLACE "/LTCG" "" link_options "${link_options}")
        set_property(TARGET ${target} PROPERTY LINK_OPTIONS "${link_options}")
    endforeach()
endif()
set_source_files_properties("${WARRIORS_GENERATED}/register_functions.cpp" TARGET_DIRECTORY ps2EntryRunner PROPERTIES SKIP_UNITY_BUILD_INCLUSION ON)
if(APPLE AND CMAKE_SYSTEM_PROCESSOR MATCHES "arm64|aarch64")
    # GlobalISel's intra-block localizer stalls on the large generated table initializer.
    set_property(SOURCE "${WARRIORS_GENERATED}/register_functions.cpp" TARGET_DIRECTORY ps2EntryRunner APPEND PROPERTY COMPILE_OPTIONS -fno-experimental-isel)
endif()
set_target_properties(ps2EntryRunner PROPERTIES
    OUTPUT_NAME "TheWarriors"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
    UNITY_BUILD_BATCH_SIZE 256
    INTERPROCEDURAL_OPTIMIZATION FALSE
    INTERPROCEDURAL_OPTIMIZATION_RELEASE FALSE
)
# shortcut: generated code is unoptimized for boot diagnostics; optimize after cold boot works.
if(MSVC)
    target_compile_options(ps2EntryRunner PRIVATE /Od)
else()
    target_compile_options(ps2EntryRunner PRIVATE -O0)
endif()
