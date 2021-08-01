mod utils;
use openapiv3::*;

mod describe_impl;

pub use axum_openapi_derive::DescribeSchema;

pub trait DescribeSchema {
    fn describe_schema() -> Schema;
}

pub trait DescribeOperation<Out, Params> {
    fn describe_operation() -> Operation;
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
    inventory::collect!(SchemaDescription);
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
