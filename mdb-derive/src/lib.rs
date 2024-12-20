//! Derive macros for mdb walkers and dcmds.

use proc_macro2::Span;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, LitByteStr, Meta, Type};

#[proc_macro_derive(Walker)]
pub fn walker(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    // Construct a function for implementing the step initialization.
    let ident = derive_input.ident.clone();
    let snake_case = format!("{}", heck::AsSnakeCase(derive_input.ident.to_string()));
    let init_fn_name = quote::format_ident!("{}_init", snake_case);
    let init_fn = quote::quote! {
        extern "C" fn #init_fn_name(state: *mut ::mdb_api::sys::mdb_walk_state_t) -> ::std::ffi::c_int {
            ::mdb_api::mdb_println!("here");
            let me = #ident::default();
            let walk_data = Box::into_raw(Box::new(Box::new(me) as Box<dyn ::mdb_api::walk::WalkStep>));
            unsafe { (*state).walk_data = walk_data.cast() };
            ::mdb_api::mdb_println!("next");
            ::mdb_api::sys::WALK_NEXT
        }
    };

    // Implement walker itself.
    let walker_impl = quote::quote! {
        impl ::mdb_api::Walker for #ident {}
    };

    // Implement the walker linkage trait. Start by constructing the walker
    // name and description.
    let walk_name = str_to_lit_byte_str(&snake_case);
    let docstring = derive_input
        .attrs
        .iter()
        .find_map(|attr| {
            let Meta::NameValue(nv) = &attr.meta else {
                return None;
            };
            if nv.path.segments.len() == 1 && &nv.path.segments[0].ident == "doc" {
                // Quote so we can stringify, and then trim. This is terrible.
                let val = &nv.value;
                let walk_descr = quote::quote! { #val }.to_string();
                Some(walk_descr)
            } else {
                None
            }
        })
        .unwrap_or_default();
    let walk_descr =
        str_to_lit_byte_str(docstring.trim_matches(|c: char| c == '"' || c.is_whitespace()));

    let linkage_impl = quote::quote! {
        impl ::mdb_api::WalkerLinkage for #ident {
            fn linkage() -> ::mdb_api::sys::mdb_walker_t {
                ::mdb_api::sys::mdb_walker_t {
                    walk_name: #walk_name.as_ptr().cast(),
                    walk_descr: #walk_descr.as_ptr().cast(),
                    walk_init: Some(#init_fn_name),
                    walk_step: Some(global_step),
                    walk_fini: Some(global_fini),
                    walk_init_arg: ::std::ptr::null_mut(),
                }
            }
        }
    };

    quote::quote! {
        #init_fn
        #walker_impl
        #linkage_impl
    }
    .into()
}

fn str_to_lit_byte_str(s: &str) -> LitByteStr {
    let mut bytes = s.as_bytes().to_vec();
    bytes.push(0);
    LitByteStr::new(&bytes, Span::call_site())
}

#[proc_macro_derive(Dcmd)]
pub fn dcmd(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let Data::Struct(s) = &derive_input.data else {
        return Error::new_spanned(derive_input, "Dcmd can only be derived for structs")
            .into_compile_error()
            .into();
    };
    let Fields::Named(fields) = &s.fields else {
        return Error::new_spanned(
            derive_input,
            "Dcmd can only be derived for structs with named fields",
        )
        .into_compile_error()
        .into();
    };
    let mut args = Vec::with_capacity(fields.named.len());
    for field in fields.named.iter() {
        match &field.ty {
            Type::Path(path) => {
                if path.path.segments.len() == 1 {
                    let seg = path.path.segments.last().unwrap();
                    if seg.ident == "u64" {
                        args.push(DcmdArgType::U64);
                        continue;
                    }
                    if seg.ident == "String" {
                        args.push(DcmdArgType::String);
                        continue;
                    }
                    if seg.ident == "char" {
                        args.push(DcmdArgType::Char);
                        continue;
                    }
                }
                return Error::new_spanned(
                    &path,
                    "Unsupported walker type, must be a u64, char, or string",
                )
                .into_compile_error()
                .into();
            }
            Type::Array(_)
            | Type::BareFn(_)
            | Type::Group(_)
            | Type::ImplTrait(_)
            | Type::Infer(_)
            | Type::Macro(_)
            | Type::Never(_)
            | Type::Paren(_)
            | Type::Ptr(_)
            | Type::Reference(_)
            | Type::Slice(_)
            | Type::TraitObject(_)
            | Type::Tuple(_)
            | Type::Verbatim(_)
            | _ => {
                return Error::new_spanned(
                    &field.ty,
                    "Unsupported walker type, must be a u64, char, or string",
                )
                .into_compile_error()
                .into()
            }
        }
    }

    proc_macro::TokenStream::new()
}

enum DcmdArgType {
    U64,
    String,
    Char,
}
