// EVMC: Ethereum Client-VM Connector API.
// Copyright 2019 The EVMC Authors.
// Licensed under the Apache License, Version 2.0.

#include "example_precompiles_vm.h"
#include <algorithm>

namespace
{
zvmc_result execute_identity(const zvmc_message* msg)
{
    auto result = zvmc_result{};

    // Check the gas cost.
    auto gas_cost = 15 + 3 * ((int64_t(msg->input_size) + 31) / 32);
    auto gas_left = msg->gas - gas_cost;
    if (gas_left < 0)
    {
        result.status_code = ZVMC_OUT_OF_GAS;
        return result;
    }

    // Execute.
    auto data = new uint8_t[msg->input_size];
    std::copy_n(msg->input_data, msg->input_size, data);

    // Return the result.
    result.status_code = ZVMC_SUCCESS;
    result.output_data = data;
    result.output_size = msg->input_size;
    result.release = [](const zvmc_result* r) { delete[] r->output_data; };
    result.gas_left = gas_left;
    return result;
}

zvmc_result execute_empty(const zvmc_message* msg)
{
    auto result = zvmc_result{};
    result.status_code = ZVMC_SUCCESS;
    result.gas_left = msg->gas;
    return result;
}

zvmc_result not_implemented()
{
    auto result = zvmc_result{};
    result.status_code = ZVMC_REJECTED;
    return result;
}

zvmc_result execute(zvmc_vm* /*vm*/,
                    const zvmc_host_interface* /*host*/,
                    zvmc_host_context* /*context*/,
                    enum zvmc_revision /*rev*/,
                    const zvmc_message* msg,
                    const uint8_t* /*code*/,
                    size_t /*code_size*/)
{
    // The EIP-1352 (https://eips.ethereum.org/EIPS/eip-1352) defines
    // the range 0 - Zffff (2 bytes) of addresses reserved for precompiled contracts.
    // Check if the code address is within the reserved range.

    constexpr auto prefix_size = sizeof(zvmc_address) - 2;
    const auto& addr = msg->code_address;
    // Check if the address prefix is all zeros.
    if (std::any_of(&addr.bytes[0], &addr.bytes[prefix_size], [](uint8_t x) { return x != 0; }))
    {
        // If not, reject the execution request.
        auto result = zvmc_result{};
        result.status_code = ZVMC_REJECTED;
        return result;
    }

    // Extract the precompiled contract id from last 2 bytes of the code address.
    const auto id = (addr.bytes[prefix_size] << 8) | addr.bytes[prefix_size + 1];
    switch (id)
    {
    case 0x0001:  // DEPOSITROOT
    case 0x0002:  // SHA256
    case 0x0004:  // Identity
        return execute_identity(msg);

    case 0x0005:  // EXPMOD
    case 0x0006:  // BNADD
    case 0x0007:  // BNMUL
    case 0x0008:  // BNPAIRING
        return not_implemented();

    default:  // As if empty code was executed.
        return execute_empty(msg);
    }
}
}  // namespace

zvmc_vm* zvmc_create_example_precompiles_vm()
{
    static struct zvmc_vm vm = {
        ZVMC_ABI_VERSION,
        "example_precompiles_vm",
        PROJECT_VERSION,
        [](zvmc_vm*) {},
        execute,
        [](zvmc_vm*) { return zvmc_capabilities_flagset{ZVMC_CAPABILITY_PRECOMPILES}; },
        nullptr,
    };
    return &vm;
}
