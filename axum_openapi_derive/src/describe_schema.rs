use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput};

struct Config<'a> {
    ident: &'a syn::Ident,
    openapiv3: TokenStream,
    axum_openapi: TokenStream,
}

pub fn derive_schema(item: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = syn::parse2(item)?;

    let axum_openapi = quote!(axum_openapi);
    let openapiv3 = quote!(#axum_openapi::__macro::openapiv3);

    let config = Config {
        ident: &input.ident,
        openapiv3,
        axum_openapi,
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
            openapiv3,
            axum_openapi,
            ..
        } = self;

        let properties = data.fields.iter().map(|field| {
            let ty = &field.ty;
            let name = field.ident.as_ref().expect("todo: tuple structs");
            let name = name.to_string();

            quote! {
                (#name.to_string(), #openapiv3::ReferenceOr::Item(Box::new(<#ty as #axum_openapi::DescribeSchema>::describe_schema()))),
            }
        });

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
            }
        })
    }
}
