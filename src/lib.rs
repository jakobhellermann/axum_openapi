mod utils;
use openapiv3::*;

mod describe_impl;

pub use axum_openapi_derive::handler;
pub use axum_openapi_derive::routes;
pub use axum_openapi_derive::DescribeSchema;

pub trait DescribeSchema {
    fn describe_schema() -> Schema;
}

pub trait OperationParameter {
    fn modify_op(operation: &mut OpenAPI);
}

#[doc(hidden)]
pub mod __macro {
    pub use inventory;
    pub use openapiv3;

    pub struct SchemaDescription {
        pub schema: openapiv3::Schema,
        pub name: String,
    }

    pub struct PathDescription {
        pub path: String,
        pub path_item: openapiv3::PathItem,
    }

    inventory::collect!(SchemaDescription);
    inventory::collect!(PathDescription);
}

pub async fn api_yaml() -> hyper::Response<hyper::Body> {
    utils::yaml_response(&*OPENAPI)
}
pub async fn api_json() -> axum::response::Json<openapiv3::OpenAPI> {
    axum::response::Json(OPENAPI.clone())
}

pub const OPENAPI: once_cell::sync::Lazy<openapiv3::OpenAPI> = once_cell::sync::Lazy::new(openapi);

fn openapi() -> openapiv3::OpenAPI {
    openapiv3::OpenAPI {
        openapi: "3.0.3".to_string(),
        paths: inventory::iter::<__macro::PathDescription>()
            .map(|path| (path.path.clone(), ReferenceOr::Item(path.path_item.clone())))
            .collect(),
        components: Some(openapiv3::Components {
            schemas: inventory::iter::<__macro::SchemaDescription>()
                .map(|desc| {
                    let reference = openapiv3::ReferenceOr::Item(desc.schema.clone());
                    (desc.name.clone(), reference)
                })
                .collect(),
            ..Default::default()
        }),
        ..Default::default()
    }
}
