use proc_macro2::TokenStream;
use quote::quote;

pub fn handler(item: TokenStream, _attr: TokenStream) -> syn::Result<TokenStream> {
    let fn_item: syn::ItemFn = syn::parse2(item)?;

    let axum_openapi = quote!(axum_openapi);
    let macro_exports = quote!(#axum_openapi::__macro);

    let fn_name = &fn_item.sig.ident;

    let _operation = quote! {
        #macro_exports::openapiv3::Operation {
            operation_id: Some(stringify!(#fn_name).to_string()),
            ..Default::default()
        }
    };

    Ok(quote! {
        #fn_item
    })
}
