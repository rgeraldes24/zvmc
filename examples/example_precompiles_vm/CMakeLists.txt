# EVMC: Ethereum Client-VM Connector API.
# Copyright 2019 The EVMC Authors.
# Licensed under the Apache License, Version 2.0.

add_library(example-precompiles-vm SHARED example_precompiles_vm.cpp example_precompiles_vm.h)
add_library(zvmc::example-precompiles-vm ALIAS example-precompiles-vm)
target_compile_features(example-precompiles-vm PRIVATE cxx_std_11)
target_link_libraries(example-precompiles-vm PRIVATE zvmc::zvmc)

add_library(example-precompiles-vm-static STATIC example_precompiles_vm.cpp example_precompiles_vm.h)
add_library(zvmc::example-precompiles-vm-static ALIAS example-precompiles-vm-static)
target_compile_features(example-precompiles-vm-static PRIVATE cxx_std_11)
target_link_libraries(example-precompiles-vm-static PRIVATE zvmc::zvmc)

set_source_files_properties(example_precompiles_vm.cpp PROPERTIES
    COMPILE_DEFINITIONS PROJECT_VERSION="${PROJECT_VERSION}")

if(ZVMC_INSTALL)
    install(TARGETS example-precompiles-vm
        ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    )
endif()
