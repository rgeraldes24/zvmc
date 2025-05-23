# EVMC: Ethereum Client-VM Connector API.
# Copyright 2016 The EVMC Authors.
# Licensed under the Apache License, Version 2.0.

cmake_minimum_required(VERSION 3.16...3.24)

if(POLICY CMP0074)
    cmake_policy(SET CMP0074 NEW)
endif()

if(TARGET zvmc)
    # The zvmc library has been already created (probably by other submodule).
    return()
endif()

list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/cmake ${CMAKE_CURRENT_SOURCE_DIR}/cmake/cable)
include(CableBuildType)
include(CableCompilerSettings)
include(CablePackage)
include(CMakeDependentOption)
include(CMakePackageConfigHelpers)

option(ZVMC_JAVA "Enable building Java Native Interface (JNI) bindings" OFF)

option(ZVMC_INSTALL "Enable ZVMC installation (e.g. make install)" ON)

option(ZVMC_TESTING "Build everything (libraries, tools, examples, internal tests)" OFF)

cmake_dependent_option(ZVMC_TOOLS "Build ZVMC tools" OFF
    "NOT ZVMC_TESTING" ON)

cmake_dependent_option(ZVMC_EXAMPLES "Build ZVMC examples" OFF
    "NOT ZVMC_TESTING" ON)

option(HUNTER_ENABLED "Enable Hunter package manager support" ${ZVMC_TOOLS})

if(HUNTER_ENABLED)
    set(HUNTER_CONFIGURATION_TYPES Release CACHE STRING "Build type of Hunter packages")
    option(HUNTER_USE_CACHE_SERVERS "Use default Hunter cache servers" NO)
    include(HunterGate)
    include(Hunter/init)
endif()

cable_set_build_type(DEFAULT Release CONFIGURATION_TYPES Debug Release)

project(zvmc)
set(PROJECT_VERSION 10.1.0)

set(CMAKE_CXX_EXTENSIONS OFF)

include(GNUInstallDirs)  # Must be included after any language is enabled.
if(NOT DEFINED CMAKE_RUNTIME_OUTPUT_DIRECTORY)
    # By default put every executable in top-level /bin dir.
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/${CMAKE_INSTALL_BINDIR})
endif()


cable_configure_compiler(NO_STACK_PROTECTION)
if(CABLE_COMPILER_GNULIKE)
    # TODO: Resolve issues or remove "result optional storage" and enable -Wcast-align.
    add_compile_options(
        -Wmissing-declarations
        $<$<COMPILE_LANGUAGE:CXX>:-Wold-style-cast>
    )
    cable_add_cxx_compiler_flag_if_supported(-Wfinal-dtor-non-final-class)
    cable_add_cxx_compiler_flag_if_supported(-Wnewline-eof)
    cable_add_cxx_compiler_flag_if_supported(-Wsuggest-destructor-override)
    cable_add_cxx_compiler_flag_if_supported(-Wunreachable-code-break)
    cable_add_cxx_compiler_flag_if_supported(-Wduplicated-cond)
    cable_add_cxx_compiler_flag_if_supported(-Wduplicate-enum)
    cable_add_cxx_compiler_flag_if_supported(-Wlogical-op)
endif()
if(CABLE_COMPILER_CLANG)
    set(CMAKE_C_FLAGS_COVERAGE "-fprofile-instr-generate -fcoverage-mapping")
    set(CMAKE_CXX_FLAGS_COVERAGE "-fprofile-instr-generate -fcoverage-mapping")
elseif(CABLE_COMPILER_GNU)
    set(CMAKE_C_FLAGS_COVERAGE "--coverage")
    set(CMAKE_CXX_FLAGS_COVERAGE "--coverage")
endif()

set(ZVMC_INCLUDE_DIR ${PROJECT_SOURCE_DIR}/include)

add_subdirectory(lib)

if(ZVMC_JAVA)
    add_subdirectory(bindings/java)
endif()

if(ZVMC_TOOLS)
    add_subdirectory(tools)
endif()

if(ZVMC_EXAMPLES)
    add_subdirectory(examples)
endif()

if(ZVMC_TESTING)
    enable_testing()
    add_subdirectory(test)
endif()

write_basic_package_version_file(zvmcConfigVersion.cmake COMPATIBILITY ExactVersion)

configure_package_config_file(
    cmake/Config.cmake.in
    zvmcConfig.cmake
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/zvmc
)

if(ZVMC_INSTALL)
    install(DIRECTORY include/zvmc DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

    install(
        EXPORT zvmcTargets
        NAMESPACE zvmc::
        DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/zvmc
    )
    install(
        FILES
        ${CMAKE_CURRENT_BINARY_DIR}/zvmcConfig.cmake
        ${CMAKE_CURRENT_BINARY_DIR}/zvmcConfigVersion.cmake
        cmake/ZVMC.cmake
        DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/zvmc
    )
endif()

set(
    CPACK_SOURCE_IGNORE_FILES
    /\\\\.git/
    /\\\\.idea/
    /build/
    /cmake-build-.*/
    /bindings/
    /Cargo.lock
    /Cargo.toml
    /target/
)
cable_add_archive_package()
