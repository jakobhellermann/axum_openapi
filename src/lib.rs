mod utils;

mod describe_impl;
mod openapi_impl;
mod operation_impl;

pub mod openapi_adapters;
pub mod openapi_traits;

pub use axum_openapi_derive::DescribeSchema;
pub use openapi_traits::DescribeSchema;

#[cfg(feature = "macro_based")]
mod global_collect;

#[cfg(feature = "macro_based")]
pub use axum_openapi_derive::handler;
#[cfg(feature = "macro_based")]
pub use axum_openapi_derive::routes;

pub mod prelude {
    pub use crate::openapi_adapters::HandlerExt;
    pub use crate::openapi_traits::{DescribeSchema, OpenapiApp};
    pub use axum_openapi_derive::DescribeSchema;
}

#[cfg(feature = "macro_based")]
pub use global_collect::{api_json, api_yaml, OPENAPI};

use axum::{
    handler::{IntoService, OnMethod},
    prelude::*,
    routing::EmptyRouter,
};
use openapiv3::OpenAPI;

/// [axum] handler function responding with the provided [OpenAPI] yaml file
pub fn openapi_yaml_endpoint<B: Send + Sync + 'static>(
    api: OpenAPI,
) -> OnMethod<IntoService<impl Handler<B, ()> + Clone, B, ()>, EmptyRouter> {
    get(|| async move { utils::yaml_response(&api) })
}

/// [axum] handler function responding with the provided [OpenAPI] json file
pub fn openapi_json_endpoint<B: Send + Sync + 'static>(
    api: OpenAPI,
) -> OnMethod<IntoService<impl Handler<B, ()> + Clone, B, ()>, EmptyRouter> {
    get(|| async { axum::response::Json(api) })
}

#[doc(hidden)]
pub mod __macro {
    pub use openapiv3;

    #[cfg(feature = "macro_based")]
    pub use inventory;

    #[cfg(feature = "macro_based")]
    pub struct SchemaDescription {
        pub schema: openapiv3::Schema,
        pub name: String,
    }

    #[cfg(feature = "macro_based")]
    pub struct PathDescription {
        pub path: String,
        pub path_item: openapiv3::PathItem,
    }

    #[cfg(feature = "macro_based")]
    pub struct OperationDescription {
        pub operation: openapiv3::Operation,
    }

    #[cfg(feature = "macro_based")]
    inventory::collect!(SchemaDescription);
    #[cfg(feature = "macro_based")]
    inventory::collect!(PathDescription);
    #[cfg(feature = "macro_based")]
    inventory::collect!(OperationDescription);
}
