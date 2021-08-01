use proc_macro2::TokenStream;
use quote::quote;

pub fn handler(item: TokenStream, _attr: TokenStream) -> syn::Result<TokenStream> {
    let fn_item: syn::ItemFn = syn::parse2(item)?;

    let axum_openapi = quote!(axum_openapi);
    let macro_exports = quote!(#axum_openapi::__macro);

    let types = fn_item.sig.inputs.iter().filter_map(|arg| match arg {
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat_ty) => Some(&pat_ty.ty),
    });

    let fn_name = &fn_item.sig.ident;

    let return_ty = match &fn_item.sig.output {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => quote! { #ty },
    };

    let submit = quote! {
        #macro_exports::inventory::submit!(#![crate=#macro_exports] {
            let mut operation = #macro_exports::openapiv3::Operation {
                operation_id: Some(stringify!(#fn_name).to_string()),
                ..Default::default()
            };
            #(<#types as #axum_openapi::OperationParameter>::modify_op(&mut operation, true);)*
            <#return_ty as #axum_openapi::OperationResult>::modify_op(&mut operation);

            #macro_exports::OperationDescription {
                operation
            }
        });
    };

    Ok(quote! {
        #submit
        #fn_item
    })
}
