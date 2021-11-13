#![crate_name = "teamer_proc_macro"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{ItemFn, NestedMeta, Meta, AttributeArgs, Lit};

/// Use for routes that require authorization. Must be placed before get/post/etc attributes
/// #Optional arguments
/// * `redirect_to`:
/// Specifies path to redirect to if not authorized
#[proc_macro_attribute]
pub fn require_authorization(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ast = syn::parse::<ItemFn>(item).unwrap();

    let arg = quote! {validator: Validator};
    let arg = syn::parse(TokenStream::from(arg)).unwrap();
    ast.sig.inputs.push(arg);

    let mut redirect_to = "/login".to_owned();
    let mut custom_handler = true;

    let attrs = parse_macro_input!(attr as AttributeArgs);
    let args = attrs.to_vec();
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(meta)) => {
                let name = meta.path.to_token_stream().to_string();
                if name == "redirect_to" {
                    if let Lit::Str(path) = &meta.lit {
                        redirect_to = path.value();
                    }
                }
            }

            NestedMeta::Meta(Meta::Path(meta)) => {
                let ident = meta.get_ident();
                if let Some(ident) = ident {
                    if ident.to_string().as_str() == "custom_handler" {
                        custom_handler = true;
                    }
                }
            }
            _ => {}
        }
    }

    if !custom_handler {
        let body = quote! {
        if !validator.validated {
            return Err(Redirect::to(uri!(#redirect_to)))
        }
    };
        let body = syn::parse(TokenStream::from(body)).unwrap();
        ast.block.stmts.insert(0, body);
    }

    TokenStream::from(ast.into_token_stream())
}

#[proc_macro]
pub fn on_auth_failed(item: TokenStream) -> TokenStream {
    let body = item.to_string();
    let body = body.as_str();
    let mut condition = "if !validator.validated {".to_owned();
    condition.push_str(body);
    condition.push_str("}");

    condition.parse().unwrap()
}