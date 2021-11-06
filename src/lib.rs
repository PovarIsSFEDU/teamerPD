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
/// ```rust
/// #[require_authorization(redirect_to = "/login")] //Does the same as without redirect_to
/// #[get("/authorized_page")]
/// fn authorized() { ... }
/// ```
#[proc_macro_attribute]
pub fn require_authorization(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ast = syn::parse::<ItemFn>(item.clone()).unwrap();

    let arg = quote! {validator: Validator};
    let arg = syn::parse(TokenStream::from(arg)).unwrap();
    ast.sig.inputs.push(arg);

    let mut redirect_to = "/login".to_owned();
    let attrs = parse_macro_input!(attr as AttributeArgs);
    let args = attrs.to_vec();
    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(meta)) = arg {
            let name = meta.path.to_token_stream().to_string();
            if name == "redirect_to" {
                if let Lit::Str(path) = &meta.lit {
                    redirect_to = path.value();
                }
            }
        }
    }

    let body = quote! {
        if !validator.validated {
            return Err(Redirect::to(uri!(#redirect_to)))
        }
    };
    let body = syn::parse(TokenStream::from(body)).unwrap();
    ast.block.stmts.insert(0, body);

    TokenStream::from(ast.into_token_stream())
}