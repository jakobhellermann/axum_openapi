use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use quote::quote;
use regex::{Captures, Regex};
use syn::{punctuated::Punctuated, spanned::Spanned};

#[derive(PartialEq, Default)]
struct Routes {
    paths: BTreeMap<String, Vec<Operation>>,
}
#[derive(Debug, PartialEq)]
struct Operation {
    method: HttpMethod,
    path: Option<syn::Path>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}
impl PartialEq<str> for HttpMethod {
    fn eq(&self, other: &str) -> bool {
        let s = match self {
            HttpMethod::Get => "get",
            HttpMethod::Put => "put",
            HttpMethod::Post => "post",
            HttpMethod::Delete => "delete",
            HttpMethod::Options => "options",
            HttpMethod::Head => "head",
            HttpMethod::Patch => "patch",
            HttpMethod::Trace => "trace",
        };
        other == s
    }
}

pub fn routes(tokens: TokenStream) -> syn::Result<TokenStream> {
    let (call, routes) = parse_routes(tokens)?;

    let axum_openapi = quote!(axum_openapi);
    let macro_exports = quote!(#axum_openapi::__macro);
    let openapiv3 = quote!(#macro_exports::openapiv3);

    let submit = routes.paths.into_iter().map(|(path, operations)| {
        let operation = |method| {
            operations
                .iter()
                .find(|op| op.method == method)
                .map(|op| {
                    let op_id = op
                        .path
                        .as_ref()
                        .and_then(|path| path.segments.last())
                        .map(|segment| segment.ident.to_string())
                        .map_or(quote!(None), |str| quote!(Some(#str.to_string())));

                    quote! { Some(#openapiv3::Operation {
                        operation_id: #op_id,
                        ..Default::default()
                    }) }
                })
                .unwrap_or_else(|| quote! { None })
        };

        let get = operation(HttpMethod::Get);
        let put = operation(HttpMethod::Put);
        let post = operation(HttpMethod::Post);
        let delete = operation(HttpMethod::Delete);
        let options = operation(HttpMethod::Options);
        let head = operation(HttpMethod::Head);
        let patch = operation(HttpMethod::Patch);
        let trace = operation(HttpMethod::Trace);

        let path_item = quote! {
            #openapiv3::PathItem {
                get: #get,
                put: #put,
                post: #post,
                delete: #delete,
                options: #options,
                head: #head,
                patch: #patch,
                trace: #trace,
                ..Default::default()
            }
        };
        quote! {
            #macro_exports::inventory::submit!(#![crate = #macro_exports] #macro_exports::PathDescription {
                path: #path.to_string(),
                path_item: #path_item,
            });
        }
    });

    Ok(quote! {{
        #(#submit)*
        #call
    }})
}

fn parse_routes(tokens: TokenStream) -> syn::Result<(syn::Expr, Routes)> {
    let call: syn::Expr = syn::parse2(tokens)?;

    let mut routes = Routes::default();

    method_or_call(&call, |ident, args| {
        if ident != "route" {
            return Err(syn::Error::new(ident.span(), "expected two arguments"));
        }

        let mut args_iter = args.iter();
        let (path, handler) = match (args_iter.next(), args_iter.next()) {
            (Some(path), Some(handler)) => (path, handler),
            _ => return Err(syn::Error::new(args.span(), "expected two arguments")),
        };

        let path = expr_string_lit(path)
            .ok_or_else(|| syn::Error::new(path.span(), "expected path string literal"))?;

        let mut ops = Vec::new();
        method_or_call(handler, |method, handler_args| {
            let method = method.to_string();
            let method = match method.as_str() {
                "get" => HttpMethod::Get,
                "put" => HttpMethod::Put,
                "post" => HttpMethod::Post,
                "delete" => HttpMethod::Delete,
                "options" => HttpMethod::Options,
                "head" => HttpMethod::Head,
                "patch" => HttpMethod::Patch,
                "trace" => HttpMethod::Trace,
                _ => return Err(syn::Error::new(method.span(), "unknown http method")),
            };

            let handler_path = handler_args
                .first()
                .ok_or_else(|| syn::Error::new(handler_args.span(), "expected one argument"))?;
            let handler_path = expr_path(handler_path);

            ops.push(Operation {
                method,
                path: handler_path.cloned(),
            });

            Ok(())
        })?;
        ops.reverse();

        let axum_path = axum_path(&path);

        routes
            .paths
            .entry(axum_path)
            .or_default()
            .extend(ops.into_iter());

        Ok(())
    })?;

    Ok((call, routes))
}

const REGEX_AXUM_PATH: Lazy<Regex> = Lazy::new(|| Regex::new(r#":(\w+)"#).unwrap());

fn axum_path(path: &str) -> String {
    REGEX_AXUM_PATH
        .replace_all(path, |caps: &Captures| format!("{{{}}}", &caps[1]))
        .into_owned()
}

fn method_or_call(
    expr: &syn::Expr,
    mut f: impl FnMut(&syn::Ident, &Punctuated<syn::Expr, syn::Token![,]>) -> syn::Result<()>,
) -> syn::Result<()> {
    match expr {
        syn::Expr::Call(call) => {
            let path = expr_path(&*call.func)
                .ok_or_else(|| syn::Error::new(call.func.span(), "expected call to function"))?;
            let ident = &path
                .segments
                .last()
                .ok_or_else(|| syn::Error::new(path.segments.span(), "empty path"))?
                .ident;
            f(ident, &call.args)?;

            Ok(())
        }
        syn::Expr::MethodCall(method_call) => {
            f(&method_call.method, &method_call.args)?;
            method_or_call(&*method_call.receiver, f)?;

            Ok(())
        }
        _ => Err(syn::Error::new(
            expr.span(),
            "expected method or function call",
        )),
    }
}

fn expr_string_lit(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) => Some(lit.value()),
        _ => None,
    }
}

fn expr_path(expr: &syn::Expr) -> Option<&syn::Path> {
    match expr {
        syn::Expr::Path(path) => Some(&path.path),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{axum_path, parse_routes, HttpMethod, Operation};
    use pretty_assertions::assert_eq;
    use quote::quote;

    #[test]
    fn call_single() {
        let tokens = quote! { route("/path", get(get_handler)) };
        let (_, routes) = parse_routes(tokens).unwrap();

        assert_eq!(
            routes.paths.into_iter().collect::<Vec<_>>(),
            vec![(
                "/path".to_string(),
                vec![Operation {
                    method: HttpMethod::Get,
                    path: Some(syn::parse_quote!(get_handler)),
                },]
            )]
        );
    }
    #[test]
    fn call_multiple_ops() {
        let tokens =
            quote! { route("/path", get(get_handler).post(post_handler).patch(patch_handler)) };
        let (_, routes) = parse_routes(tokens).unwrap();

        assert_eq!(
            routes.paths.into_iter().collect::<Vec<_>>(),
            vec![(
                "/path".to_string(),
                vec![
                    Operation {
                        method: HttpMethod::Get,
                        path: Some(syn::parse_quote!(get_handler)),
                    },
                    Operation {
                        method: HttpMethod::Post,
                        path: Some(syn::parse_quote!(post_handler)),
                    },
                    Operation {
                        method: HttpMethod::Patch,
                        path: Some(syn::parse_quote!(patch_handler)),
                    },
                ]
            ),]
        );
    }

    #[test]
    fn full() {
        let tokens = quote! { route("/path", get(get_handler).post(post_handler).patch(patch_handler)).route("/path2", get(get_handler_2)) };
        let (_, routes) = parse_routes(tokens).unwrap();

        assert_eq!(
            routes.paths.into_iter().collect::<Vec<_>>(),
            vec![
                (
                    "/path".to_string(),
                    vec![
                        Operation {
                            method: HttpMethod::Get,
                            path: Some(syn::parse_quote!(get_handler)),
                        },
                        Operation {
                            method: HttpMethod::Post,
                            path: Some(syn::parse_quote!(post_handler)),
                        },
                        Operation {
                            method: HttpMethod::Patch,
                            path: Some(syn::parse_quote!(patch_handler)),
                        },
                    ]
                ),
                (
                    "/path2".to_string(),
                    vec![Operation {
                        method: HttpMethod::Get,
                        path: Some(syn::parse_quote!(get_handler_2)),
                    },]
                ),
            ]
        );
    }

    #[test]
    fn closure() {
        let tokens = quote! { route("/path", get(|| async {})) };
        let (_, routes) = parse_routes(tokens).unwrap();

        assert_eq!(
            routes.paths.into_iter().collect::<Vec<_>>(),
            vec![(
                "/path".to_string(),
                vec![Operation {
                    method: HttpMethod::Get,
                    path: None
                }]
            )]
        );
    }

    #[test]
    fn axum_path_regular() {
        assert_eq!(axum_path("/path/foo"), "/path/foo");
    }

    #[test]
    fn axum_path_params() {
        assert_eq!(axum_path("/path/:id/:bla"), "/path/{id}/{bla}");
    }
}
