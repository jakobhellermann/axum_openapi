mod utils;
use openapiv3::*;

mod describe_impl;
mod operation_impl;

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

pub async fn api_yaml() -> hyper::Response<hyper::Body> {
    utils::yaml_response(&*OPENAPI)
}
pub async fn api_json() -> axum::response::Json<openapiv3::OpenAPI> {
    axum::response::Json(OPENAPI.clone())
}

pub const OPENAPI: once_cell::sync::Lazy<openapiv3::OpenAPI> = once_cell::sync::Lazy::new(openapi);

fn openapi() -> openapiv3::OpenAPI {
    let handler_ops: std::collections::HashMap<&str, &Operation> =
        inventory::iter::<__macro::OperationDescription>()
            .filter_map(|op| {
                let op_id = op.operation.operation_id.as_deref()?;
                Some((op_id, &op.operation))
            })
            .collect();

    openapiv3::OpenAPI {
        openapi: "3.0.3".to_string(),
        paths: inventory::iter::<__macro::PathDescription>()
            .map(|path| {
                let mut item = path.path_item.clone();
                patch_operations(&mut item, &handler_ops, &path.path);

                (path.path.clone(), ReferenceOr::Item(item))
            })
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

fn patch_operations(
    path_item: &mut PathItem,
    handler_ops: &std::collections::HashMap<&str, &Operation>,
    path: &str,
) {
    let path_params: Vec<_> = path
        .split('/')
        .filter_map(|component| {
            if component.starts_with('{') && component.ends_with('}') {
                Some(&component[1..component.len() - 1])
            } else {
                None
            }
        })
        .collect();

    let ops = std::array::IntoIter::new([
        path_item.get.as_mut(),
        path_item.put.as_mut(),
        path_item.post.as_mut(),
        path_item.delete.as_mut(),
        path_item.options.as_mut(),
        path_item.head.as_mut(),
        path_item.patch.as_mut(),
        path_item.trace.as_mut(),
    ]);
    for (handler_op, op) in ops
        .into_iter()
        .flatten()
        .filter_map(|op| Some((*handler_ops.get(op.operation_id.as_deref()?)?, op)))
    {
        *op = handler_op.clone();

        op.parameters
            .iter_mut()
            .filter_map(|param| match param {
                ReferenceOr::Item(Parameter::Path { parameter_data, .. }) => Some(parameter_data),
                _ => None,
            })
            .for_each(|param| {
                if let Some(i) = param.name.strip_prefix("__parameter") {
                    if let Ok(i) = i.parse::<usize>() {
                        param.name = path_params[i].to_string();
                    }
                }
            });
    }
}
