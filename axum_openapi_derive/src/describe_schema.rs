use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput};

struct Config<'a> {
    ident: &'a syn::Ident,
    axum_openapi: TokenStream,
    macro_exports: TokenStream,
}

pub fn derive_schema(item: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = syn::parse2(item)?;

    let axum_openapi = quote!(axum_openapi);
    let macro_exports = quote!(#axum_openapi::__macro);

    let config = Config {
        ident: &input.ident,
        axum_openapi,
        macro_exports,
    };

    match input.data {
        syn::Data::Struct(data) => config.derive_schema_struct(data),
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }
}

impl Config<'_> {
    pub fn derive_schema_struct(&self, data: DataStruct) -> syn::Result<TokenStream> {
        let Config {
            ident,
            macro_exports,
            axum_openapi,
            ..
        } = self;
        let openapiv3 = quote!(#macro_exports::openapiv3);

        let properties = data.fields.iter().map(|field| {
            let ty = &field.ty;
            let name = field.ident.as_ref().expect("todo: tuple structs");
            let name = name.to_string();

            quote! {
                (#name.to_string(), #openapiv3::ReferenceOr::Item(Box::new(<#ty as #axum_openapi::DescribeSchema>::describe_schema()))),
            }
        });

        let ref_name = ident.to_string();

        Ok(quote! {
            impl #axum_openapi::DescribeSchema for #ident {
                fn describe_schema() -> #openapiv3::Schema {
                    let properties = std::array::IntoIter::new([
                        #(#properties)*
                    ]).collect();
                    let required = Vec::new();
                    let obj = #openapiv3::ObjectType {
                        properties,
                        required,
                        additional_properties: None,
                        min_properties: None,
                        max_properties: None,
                    };
                    #openapiv3::Schema {
                        schema_data: Default::default(),
                        schema_kind: #openapiv3::SchemaKind::Type(#openapiv3::Type::Object(obj)),
                    }
                }

                fn ref_name() -> Option<String> {
                    Some(#ref_name.to_string())
                }
            }

            #[cfg(feature = "macro-based")]
            #macro_exports::inventory::submit!(#![crate = #macro_exports] #macro_exports::SchemaDescription {
                schema: <#ident as #axum_openapi::DescribeSchema>::describe_schema(),
                name: #ref_name.to_string(),
            });
        })
    }
}
