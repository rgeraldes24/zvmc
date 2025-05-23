// EVMC: Ethereum Client-VM Connector API.
// Copyright 2019 The EVMC Authors.
// Licensed under the Apache License, Version 2.0.

//! zvmc-declare is an attribute-style procedural macro to be used for automatic generation of FFI
//! code for the ZVMC API with minimal boilerplate.
//!
//! zvmc-declare can be used by applying its attribute to any struct which implements the `ZvmcVm`
//! trait, from the zvmc-vm crate.
//!
//! The macro takes three arguments: a valid UTF-8 stylized VM name, a comma-separated list of
//! capabilities, and a version string.
//!
//! # Example
//! ```
//! #[zvmc_declare::zvmc_declare_vm("This is an example VM name", "zwasm, zvm", "1.2.3-custom")]
//! pub struct ExampleVM;
//!
//! impl zvmc_vm::ZvmcVm for ExampleVM {
//!     fn init() -> Self {
//!             ExampleVM {}
//!     }
//!
//!     fn execute(&self, revision: zvmc_vm::ffi::zvmc_revision, code: &[u8], message: &zvmc_vm::ExecutionMessage, context: Option<&mut zvmc_vm::ExecutionContext>) -> zvmc_vm::ExecutionResult {
//!             zvmc_vm::ExecutionResult::success(1337, 0, None)
//!     }
//! }
//! ```

// Set a higher recursion limit because parsing certain token trees might fail with the default of 64.
#![recursion_limit = "256"]

extern crate proc_macro;

use heck::ShoutySnakeCase;
use heck::SnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::AttributeArgs;
use syn::Ident;
use syn::ItemStruct;
use syn::Lit;
use syn::LitInt;
use syn::LitStr;
use syn::NestedMeta;

struct VMNameSet {
    type_name: String,
    name_allcaps: String,
    name_lowercase: String,
}

struct VMMetaData {
    capabilities: u32,
    // Not included in VMNameSet because it is parsed from the meta-item arguments.
    name_stylized: String,
    custom_version: String,
}

#[allow(dead_code)]
impl VMNameSet {
    fn new(ident: String) -> Self {
        let caps = ident.to_shouty_snake_case();
        let lowercase = ident
            .to_snake_case()
            .chars()
            .filter(|c| *c != '_')
            .collect();
        VMNameSet {
            type_name: ident,
            name_allcaps: caps,
            name_lowercase: lowercase,
        }
    }

    /// Return a reference to the struct name, as a string.
    fn get_type_name(&self) -> &String {
        &self.type_name
    }

    /// Return a reference to the name in shouty snake case.
    fn get_name_caps(&self) -> &String {
        &self.name_allcaps
    }

    /// Return a reference to the name in lowercase, with all underscores removed. (Used for
    /// symbols like zvmc_create_vmname)
    fn get_name_lowercase(&self) -> &String {
        &self.name_lowercase
    }

    /// Get the struct's name as an explicit identifier to be interpolated with quote.
    fn get_type_as_ident(&self) -> Ident {
        Ident::new(&self.type_name, self.type_name.span())
    }

    /// Get the lowercase name appended with arbitrary text as an explicit ident.
    fn get_lowercase_as_ident_append(&self, suffix: &str) -> Ident {
        let concat = format!("{}{}", &self.name_lowercase, suffix);
        Ident::new(&concat, self.name_lowercase.span())
    }

    /// Get the lowercase name prepended with arbitrary text as an explicit ident.
    fn get_lowercase_as_ident_prepend(&self, prefix: &str) -> Ident {
        let concat = format!("{}{}", prefix, &self.name_lowercase);
        Ident::new(&concat, self.name_lowercase.span())
    }

    /// Get the lowercase name appended with arbitrary text as an explicit ident.
    fn get_caps_as_ident_append(&self, suffix: &str) -> Ident {
        let concat = format!("{}{}", &self.name_allcaps, suffix);
        Ident::new(&concat, self.name_allcaps.span())
    }
}

impl VMMetaData {
    fn new(args: AttributeArgs) -> Self {
        assert_eq!(args.len(), 3, "Incorrect number of arguments supplied");

        let vm_name_meta = &args[0];
        let vm_capabilities_meta = &args[1];
        let vm_version_meta = &args[2];

        let vm_name_string = match vm_name_meta {
            NestedMeta::Lit(lit) => {
                if let Lit::Str(s) = lit {
                    // Add a null terminator here to ensure that it is handled correctly when
                    // converted to a C String.
                    let mut ret = s.value();
                    ret.push('\0');
                    ret
                } else {
                    panic!("Literal argument type mismatch")
                }
            }
            _ => panic!("Argument 1 must be a string literal"),
        };

        let vm_capabilities_string = match vm_capabilities_meta {
            NestedMeta::Lit(lit) => {
                if let Lit::Str(s) = lit {
                    s.value()
                } else {
                    panic!("Literal argument type mismatch")
                }
            }
            _ => panic!("Argument 2 must be a string literal"),
        };

        // Parse the individual capabilities out of the list and prepare a capabilities flagset.
        // Prune spaces and underscores here to make a clean comma-separated list.
        let capabilities_list_pruned: String = vm_capabilities_string
            .chars()
            .filter(|c| *c != '_' && *c != ' ')
            .collect();
        let capabilities_flags = {
            let mut ret: u32 = 0;
            for capability in capabilities_list_pruned.split(',') {
                match capability {
                    "zvm" => ret |= 1,
                    "zwasm" => ret |= 1 << 1,
                    "precompiles" => ret |= 1 << 2,
                    _ => panic!("Invalid capability specified."),
                }
            }
            ret
        };

        let vm_version_string: String = if let NestedMeta::Lit(lit) = vm_version_meta {
            match lit {
                // Add a null terminator here to ensure that it is handled correctly when
                // converted to a C String.
                Lit::Str(s) => {
                    let mut ret = s.value();
                    ret.push('\0');
                    ret
                }
                _ => panic!("Literal argument type mismatch"),
            }
        } else {
            panic!("Argument 3 must be a string literal")
        };

        // Make sure that the only null byte is the terminator we inserted in each string.
        assert_eq!(vm_name_string.matches('\0').count(), 1);
        assert_eq!(vm_version_string.matches('\0').count(), 1);

        VMMetaData {
            capabilities: capabilities_flags,
            name_stylized: vm_name_string,
            custom_version: vm_version_string,
        }
    }

    fn get_capabilities(&self) -> u32 {
        self.capabilities
    }

    fn get_name_stylized_nulterm(&self) -> &String {
        &self.name_stylized
    }

    fn get_custom_version_nulterm(&self) -> &String {
        &self.custom_version
    }
}

#[proc_macro_attribute]
pub fn zvmc_declare_vm(args: TokenStream, item: TokenStream) -> TokenStream {
    // First, try to parse the input token stream into an AST node representing a struct
    // declaration.
    let input: ItemStruct = parse_macro_input!(item as ItemStruct);

    // Extract the identifier of the struct from the AST node.
    let vm_type_name: String = input.ident.to_string();

    // Build the VM name set.
    let names = VMNameSet::new(vm_type_name);

    // Parse the arguments for the macro.
    let meta_args = parse_macro_input!(args as AttributeArgs);
    let vm_data = VMMetaData::new(meta_args);

    // Get all the tokens from the respective helpers.
    let static_data_tokens = build_static_data(&names, &vm_data);
    let capabilities_tokens = build_capabilities_fn(vm_data.get_capabilities());
    let set_option_tokens = build_set_option_fn(&names);
    let create_tokens = build_create_fn(&names);
    let destroy_tokens = build_destroy_fn(&names);
    let execute_tokens = build_execute_fn(&names);

    let quoted = quote! {
        #input
        #static_data_tokens
        #capabilities_tokens
        #set_option_tokens
        #create_tokens
        #destroy_tokens
        #execute_tokens
    };

    quoted.into()
}

/// Generate tokens for the static data associated with an ZVMC VM.
fn build_static_data(names: &VMNameSet, metadata: &VMMetaData) -> proc_macro2::TokenStream {
    // Stitch together the VM name and the suffix _NAME
    let static_name_ident = names.get_caps_as_ident_append("_NAME");
    let static_version_ident = names.get_caps_as_ident_append("_VERSION");

    // Turn the stylized VM name and version into string literals.
    let stylized_name_literal = LitStr::new(
        metadata.get_name_stylized_nulterm().as_str(),
        metadata.get_name_stylized_nulterm().as_str().span(),
    );

    // Turn the version into a string literal.
    let version_string = metadata.get_custom_version_nulterm();
    let version_literal = LitStr::new(version_string.as_str(), version_string.as_str().span());

    quote! {
        static #static_name_ident: &'static str = #stylized_name_literal;
        static #static_version_ident: &'static str = #version_literal;
    }
}

/// Takes a capabilities flag and builds the zvmc_get_capabilities callback.
fn build_capabilities_fn(capabilities: u32) -> proc_macro2::TokenStream {
    let capabilities_string = capabilities.to_string();
    let capabilities_literal = LitInt::new(&capabilities_string, capabilities.span());

    quote! {
        extern "C" fn __zvmc_get_capabilities(instance: *mut ::zvmc_vm::ffi::zvmc_vm) -> ::zvmc_vm::ffi::zvmc_capabilities_flagset {
            #capabilities_literal
        }
    }
}

fn build_set_option_fn(names: &VMNameSet) -> proc_macro2::TokenStream {
    let type_name_ident = names.get_type_as_ident();

    quote! {
        extern "C" fn __zvmc_set_option(
            instance: *mut ::zvmc_vm::ffi::zvmc_vm,
            key: *const std::os::raw::c_char,
            value: *const std::os::raw::c_char,
        ) -> ::zvmc_vm::ffi::zvmc_set_option_result
        {
            use zvmc_vm::{ZvmcVm, SetOptionError};
            use std::ffi::CStr;

            assert!(!instance.is_null());

            if key.is_null() {
                return ::zvmc_vm::ffi::zvmc_set_option_result::ZVMC_SET_OPTION_INVALID_NAME;
            }

            let key = unsafe { CStr::from_ptr(key) };
            let key = match key.to_str() {
                Ok(k) => k,
                Err(e) => return ::zvmc_vm::ffi::zvmc_set_option_result::ZVMC_SET_OPTION_INVALID_NAME,
            };

            let value = if !value.is_null() {
                unsafe { CStr::from_ptr(value) }
            } else {
                unsafe { CStr::from_bytes_with_nul_unchecked(&[0]) }
            };

            let value = match value.to_str() {
                Ok(k) => k,
                Err(e) => return ::zvmc_vm::ffi::zvmc_set_option_result::ZVMC_SET_OPTION_INVALID_VALUE,
            };

            let mut container = unsafe {
                // Acquire ownership from ZVMC.
                ::zvmc_vm::ZvmcContainer::<#type_name_ident>::from_ffi_pointer(instance)
            };

            let result = match container.set_option(key, value) {
                Ok(()) => ::zvmc_vm::ffi::zvmc_set_option_result::ZVMC_SET_OPTION_SUCCESS,
                Err(SetOptionError::InvalidKey) => ::zvmc_vm::ffi::zvmc_set_option_result::ZVMC_SET_OPTION_INVALID_NAME,
                Err(SetOptionError::InvalidValue) => ::zvmc_vm::ffi::zvmc_set_option_result::ZVMC_SET_OPTION_INVALID_VALUE,
            };

            unsafe {
                // Release ownership to ZVMC.
                ::zvmc_vm::ZvmcContainer::into_ffi_pointer(container);
            }

            result
        }
    }
}

/// Takes an identifier and struct definition, builds an zvmc_create_* function for FFI.
fn build_create_fn(names: &VMNameSet) -> proc_macro2::TokenStream {
    let type_ident = names.get_type_as_ident();
    let fn_ident = names.get_lowercase_as_ident_prepend("zvmc_create_");

    let static_version_ident = names.get_caps_as_ident_append("_VERSION");
    let static_name_ident = names.get_caps_as_ident_append("_NAME");

    // Note: we can get CStrs unchecked because we did the checks on instantiation of VMMetaData.
    quote! {
        #[no_mangle]
        extern "C" fn #fn_ident() -> *const ::zvmc_vm::ffi::zvmc_vm {
            let new_instance = ::zvmc_vm::ffi::zvmc_vm {
                abi_version: ::zvmc_vm::ffi::ZVMC_ABI_VERSION as i32,
                destroy: Some(__zvmc_destroy),
                execute: Some(__zvmc_execute),
                get_capabilities: Some(__zvmc_get_capabilities),
                set_option: Some(__zvmc_set_option),
                name: unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(#static_name_ident.as_bytes()).as_ptr() },
                version: unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(#static_version_ident.as_bytes()).as_ptr() },
            };

            let container = ::zvmc_vm::ZvmcContainer::<#type_ident>::new(new_instance);

            unsafe {
                // Release ownership to ZVMC.
                ::zvmc_vm::ZvmcContainer::into_ffi_pointer(container)
            }
        }
    }
}

/// Builds a callback to dispose of the VM instance.
fn build_destroy_fn(names: &VMNameSet) -> proc_macro2::TokenStream {
    let type_ident = names.get_type_as_ident();

    quote! {
        extern "C" fn __zvmc_destroy(instance: *mut ::zvmc_vm::ffi::zvmc_vm) {
            if instance.is_null() {
                // This is an irrecoverable error that violates the ZVMC spec.
                std::process::abort();
            }
            unsafe {
                // Acquire ownership from ZVMC. This will deallocate it also at the end of the scope.
                ::zvmc_vm::ZvmcContainer::<#type_ident>::from_ffi_pointer(instance);
            }
        }
    }
}

/// Builds the main execution entry point.
fn build_execute_fn(names: &VMNameSet) -> proc_macro2::TokenStream {
    let type_name_ident = names.get_type_as_ident();

    quote! {
        extern "C" fn __zvmc_execute(
            instance: *mut ::zvmc_vm::ffi::zvmc_vm,
            host: *const ::zvmc_vm::ffi::zvmc_host_interface,
            context: *mut ::zvmc_vm::ffi::zvmc_host_context,
            revision: ::zvmc_vm::ffi::zvmc_revision,
            msg: *const ::zvmc_vm::ffi::zvmc_message,
            code: *const u8,
            code_size: usize
        ) -> ::zvmc_vm::ffi::zvmc_result
        {
            use zvmc_vm::ZvmcVm;

            // TODO: context is optional in case of the "precompiles" capability
            if instance.is_null() || msg.is_null() || (code.is_null() && code_size != 0) {
                // These are irrecoverable errors that violate the ZVMC spec.
                std::process::abort();
            }

            assert!(!instance.is_null());
            assert!(!msg.is_null());

            let execution_message: ::zvmc_vm::ExecutionMessage = unsafe {
                msg.as_ref().expect("ZVMC message is null").into()
            };

            let empty_code = [0u8;0];
            let code_ref: &[u8] = if code.is_null() {
                assert_eq!(code_size, 0);
                &empty_code
            } else {
                unsafe {
                    ::std::slice::from_raw_parts(code, code_size)
                }
            };

            let container = unsafe {
                // Acquire ownership from ZVMC.
                ::zvmc_vm::ZvmcContainer::<#type_name_ident>::from_ffi_pointer(instance)
            };

            let result = ::std::panic::catch_unwind(|| {
                if host.is_null() {
                    container.execute(revision, code_ref, &execution_message, None)
                } else {
                    let mut execution_context = unsafe {
                        ::zvmc_vm::ExecutionContext::new(
                            host.as_ref().expect("ZVMC host is null"),
                            context,
                        )
                    };
                    container.execute(revision, code_ref, &execution_message, Some(&mut execution_context))
                }
            });

            let result = if result.is_err() {
                // Consider a panic an internal error.
                ::zvmc_vm::ExecutionResult::new(::zvmc_vm::ffi::zvmc_status_code::ZVMC_INTERNAL_ERROR, 0, 0, None)
            } else {
                result.unwrap()
            };

            unsafe {
                // Release ownership to ZVMC.
                ::zvmc_vm::ZvmcContainer::into_ffi_pointer(container);
            }

            result.into()
        }
    }
}
