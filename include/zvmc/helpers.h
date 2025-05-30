// EVMC: Ethereum Client-VM Connector API.
// Copyright 2018 The EVMC Authors.
// Licensed under the Apache License, Version 2.0.

/**
 * ZVMC Helpers
 *
 * A collection of C helper functions for invoking a VM instance methods.
 * These are convenient for languages where invoking function pointers
 * is "ugly" or impossible (such as Go).
 *
 * @defgroup helpers ZVMC Helpers
 * @{
 */
#pragma once

#include <stdlib.h>
#include <string.h>
#include <zvmc/zvmc.h>

#ifdef __cplusplus
extern "C" {
#ifdef __GNUC__
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wold-style-cast"
#endif
#endif

/**
 * Returns true if the VM has a compatible ABI version.
 */
static inline bool zvmc_is_abi_compatible(struct zvmc_vm* vm)
{
    return vm->abi_version == ZVMC_ABI_VERSION;
}

/**
 * Returns the name of the VM.
 */
static inline const char* zvmc_vm_name(struct zvmc_vm* vm)
{
    return vm->name;
}

/**
 * Returns the version of the VM.
 */
static inline const char* zvmc_vm_version(struct zvmc_vm* vm)
{
    return vm->version;
}

/**
 * Checks if the VM has the given capability.
 *
 * @see zvmc_get_capabilities_fn
 */
static inline bool zvmc_vm_has_capability(struct zvmc_vm* vm, enum zvmc_capabilities capability)
{
    return (vm->get_capabilities(vm) & (zvmc_capabilities_flagset)capability) != 0;
}

/**
 * Destroys the VM instance.
 *
 * @see zvmc_destroy_fn
 */
static inline void zvmc_destroy(struct zvmc_vm* vm)
{
    vm->destroy(vm);
}

/**
 * Sets the option for the VM, if the feature is supported by the VM.
 *
 * @see zvmc_set_option_fn
 */
static inline enum zvmc_set_option_result zvmc_set_option(struct zvmc_vm* vm,
                                                          char const* name,
                                                          char const* value)
{
    if (vm->set_option)
        return vm->set_option(vm, name, value);
    return ZVMC_SET_OPTION_INVALID_NAME;
}

/**
 * Executes code in the VM instance.
 *
 * @see zvmc_execute_fn.
 */
static inline struct zvmc_result zvmc_execute(struct zvmc_vm* vm,
                                              const struct zvmc_host_interface* host,
                                              struct zvmc_host_context* context,
                                              enum zvmc_revision rev,
                                              const struct zvmc_message* msg,
                                              uint8_t const* code,
                                              size_t code_size)
{
    return vm->execute(vm, host, context, rev, msg, code, code_size);
}

/// The zvmc_result release function using free() for releasing the memory.
///
/// This function is used in the zvmc_make_result(),
/// but may be also used in other case if convenient.
///
/// @param result The result object.
static void zvmc_free_result_memory(const struct zvmc_result* result)
{
    free((uint8_t*)result->output_data);
}

/// Creates the result from the provided arguments.
///
/// The provided output is copied to memory allocated with malloc()
/// and the zvmc_result::release function is set to one invoking free().
///
/// In case of memory allocation failure, the result has all fields zeroed
/// and only zvmc_result::status_code is set to ::ZVMC_OUT_OF_MEMORY internal error.
///
/// @param status_code  The status code.
/// @param gas_left     The amount of gas left.
/// @param gas_refund   The amount of refunded gas.
/// @param output_data  The pointer to the output.
/// @param output_size  The output size.
static inline struct zvmc_result zvmc_make_result(enum zvmc_status_code status_code,
                                                  int64_t gas_left,
                                                  int64_t gas_refund,
                                                  const uint8_t* output_data,
                                                  size_t output_size)
{
    struct zvmc_result result;
    memset(&result, 0, sizeof(result));

    if (output_size != 0)
    {
        uint8_t* buffer = (uint8_t*)malloc(output_size);

        if (!buffer)
        {
            result.status_code = ZVMC_OUT_OF_MEMORY;
            return result;
        }

        memcpy(buffer, output_data, output_size);
        result.output_data = buffer;
        result.output_size = output_size;
        result.release = zvmc_free_result_memory;
    }

    result.status_code = status_code;
    result.gas_left = gas_left;
    result.gas_refund = gas_refund;
    return result;
}

/**
 * Releases the resources allocated to the execution result.
 *
 * @param result  The result object to be released. MUST NOT be NULL.
 *
 * @see zvmc_result::release() zvmc_release_result_fn
 */
static inline void zvmc_release_result(struct zvmc_result* result)
{
    if (result->release)
        result->release(result);
}


/**
 * Helpers for optional storage of zvmc_result.
 *
 * In some contexts (i.e. zvmc_result::create_address is unused) objects of
 * type zvmc_result contains a memory storage that MAY be used by the object
 * owner. This group defines helper types and functions for accessing
 * the optional storage.
 *
 * @defgroup result_optional_storage Result Optional Storage
 * @{
 */

/**
 * The union representing zvmc_result "optional storage".
 *
 * The zvmc_result struct contains 24 bytes of optional storage that can be
 * reused by the object creator if the object does not contain
 * zvmc_result::create_address.
 *
 * A VM implementation MAY use this memory to keep additional data
 * when returning result from zvmc_execute_fn().
 * The host application MAY use this memory to keep additional data
 * when returning result of performed calls from zvmc_call_fn().
 *
 * @see zvmc_get_optional_storage(), zvmc_get_const_optional_storage().
 */
union zvmc_result_optional_storage
{
    uint8_t bytes[24]; /**< 24 bytes of optional storage. */
    void* pointer;     /**< Optional pointer. */
};

/** Provides read-write access to zvmc_result "optional storage". */
static inline union zvmc_result_optional_storage* zvmc_get_optional_storage(
    struct zvmc_result* result)
{
    return (union zvmc_result_optional_storage*)&result->create_address;
}

/** Provides read-only access to zvmc_result "optional storage". */
static inline const union zvmc_result_optional_storage* zvmc_get_const_optional_storage(
    const struct zvmc_result* result)
{
    return (const union zvmc_result_optional_storage*)&result->create_address;
}

/** @} */

/** Returns text representation of the ::zvmc_status_code. */
static inline const char* zvmc_status_code_to_string(enum zvmc_status_code status_code)
{
    switch (status_code)
    {
    case ZVMC_SUCCESS:
        return "success";
    case ZVMC_FAILURE:
        return "failure";
    case ZVMC_REVERT:
        return "revert";
    case ZVMC_OUT_OF_GAS:
        return "out of gas";
    case ZVMC_INVALID_INSTRUCTION:
        return "invalid instruction";
    case ZVMC_UNDEFINED_INSTRUCTION:
        return "undefined instruction";
    case ZVMC_STACK_OVERFLOW:
        return "stack overflow";
    case ZVMC_STACK_UNDERFLOW:
        return "stack underflow";
    case ZVMC_BAD_JUMP_DESTINATION:
        return "bad jump destination";
    case ZVMC_INVALID_MEMORY_ACCESS:
        return "invalid memory access";
    case ZVMC_CALL_DEPTH_EXCEEDED:
        return "call depth exceeded";
    case ZVMC_STATIC_MODE_VIOLATION:
        return "static mode violation";
    case ZVMC_PRECOMPILE_FAILURE:
        return "precompile failure";
    case ZVMC_CONTRACT_VALIDATION_FAILURE:
        return "contract validation failure";
    case ZVMC_ARGUMENT_OUT_OF_RANGE:
        return "argument out of range";
    case ZVMC_WASM_UNREACHABLE_INSTRUCTION:
        return "wasm unreachable instruction";
    case ZVMC_WASM_TRAP:
        return "wasm trap";
    case ZVMC_INSUFFICIENT_BALANCE:
        return "insufficient balance";
    case ZVMC_INTERNAL_ERROR:
        return "internal error";
    case ZVMC_REJECTED:
        return "rejected";
    case ZVMC_OUT_OF_MEMORY:
        return "out of memory";
    }
    return "<unknown>";
}

/** Returns the name of the ::zvmc_revision. */
static inline const char* zvmc_revision_to_string(enum zvmc_revision rev)
{
    switch (rev)
    {
    case ZVMC_SHANGHAI:
        return "Shanghai";
    }
    return "<unknown>";
}

/** @} */

#ifdef __cplusplus
#ifdef __GNUC__
#pragma GCC diagnostic pop
#endif
}  // extern "C"
#endif
