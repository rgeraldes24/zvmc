# EVMC: Ethereum Client-VM Connector API.
# Copyright 2018 The EVMC Authors.
# Licensed under the Apache License, Version 2.0.

include(GNUInstallDirs)

hunter_add_package(GTest)
find_package(GTest CONFIG REQUIRED)

# Disable support for std::tr1::tuple in GTest. This causes problems in Visual Studio 2015.
set_target_properties(GTest::gtest PROPERTIES INTERFACE_COMPILE_DEFINITIONS GTEST_HAS_TR1_TUPLE=0)

add_executable(zvmc-vmtester vmtester.hpp vmtester.cpp tests.cpp)
target_link_libraries(zvmc-vmtester PRIVATE zvmc::loader zvmc::mocked_host GTest::gtest)
set_source_files_properties(vmtester.cpp PROPERTIES COMPILE_DEFINITIONS PROJECT_VERSION="${PROJECT_VERSION}")
add_executable(zvmc::zvmc-vmtester ALIAS zvmc-vmtester)

if(ZVMC_INSTALL)
    install(TARGETS zvmc-vmtester EXPORT zvmcTargets RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})
endif()
