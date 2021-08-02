mod utils;
use openapiv3::*;

mod describe_impl;
mod operation_impl;

mod global_collect;

pub use global_collect::{api_json, api_yaml, OPENAPI};

pub use axum_openapi_derive::handler;
pub use axum_openapi_derive::routes;
pub use axum_openapi_derive::DescribeSchema;

pub trait DescribeSchema {
    fn describe_schema() -> Schema;
    fn ref_name() -> Option<String> {
        None
    }

    fn reference_or_schema() -> ReferenceOr<Schema> {
        match Self::ref_name() {
            Some(ref_name) => ReferenceOr::Reference {
                reference: format!("#/components/schemas/{}", ref_name),
            },
            None => ReferenceOr::Item(Self::describe_schema()),
        }
    }
}

pub trait OperationParameter {
    fn modify_op(operation: &mut Operation, required: bool);
}
pub trait OperationResult {
    fn modify_op(operation: &mut Operation);
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

    pub struct OperationDescription {
        pub operation: openapiv3::Operation,
    }

    inventory::collect!(SchemaDescription);
    inventory::collect!(PathDescription);
    inventory::collect!(OperationDescription);
}
