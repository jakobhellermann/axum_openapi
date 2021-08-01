use axum_openapi_derive::all_tuples;
use openapiv3::*;

use crate::{DescribeSchema, OperationParameter, OperationResult};

impl<T: OperationParameter> OperationParameter for Option<T> {
    fn modify_op(op: &mut Operation, _: bool) {
        T::modify_op(op, false);
    }
}

impl<T: DescribeSchema> OperationParameter for axum::extract::Json<T> {
    fn modify_op(op: &mut Operation, required: bool) {
        if op.request_body.is_some() {
            todo!();
        }

        op.request_body = Some(ReferenceOr::Item(RequestBody {
            description: None,
            content: std::array::IntoIter::new([(
                "application/json".to_string(),
                MediaType {
                    schema: Some(T::reference_or_schema()),
                    example: None,
                    examples: Default::default(),
                    encoding: Default::default(),
                },
            )])
            .collect(),
            required,
            extensions: Default::default(),
        }));
    }
}

macro_rules! impl_url_params {
    ( $($param:ident),* ) => {

        #[allow(deprecated)]
        impl<$($param: DescribeSchema,)*> OperationParameter for axum::extract::UrlParams<($($param,)*)> {
            fn modify_op(op: &mut Operation, _: bool) {
                let parameters = vec![$(<$param as DescribeSchema>::reference_or_schema(),)*];
                url_params(op, parameters)
            }
        }
    };
}

fn url_params(op: &mut Operation, parameters: Vec<ReferenceOr<Schema>>) {
    for (i, schema) in parameters.into_iter().enumerate() {
        op.parameters.push(ReferenceOr::Item(Parameter::Path {
            parameter_data: ParameterData {
                name: format!("__parameter{}", i),
                description: None,
                required: true,
                deprecated: None,
                format: ParameterSchemaOrContent::Schema(schema),
                example: None,
                examples: Default::default(),
                explode: None,
                extensions: Default::default(),
            },
            style: PathStyle::Simple,
        }))
    }
}

all_tuples!(impl_url_params, 1, 6, T);

impl<T: DescribeSchema> OperationParameter for axum::extract::Query<T> {
    fn modify_op(op: &mut Operation, required: bool) {
        let schema = T::describe_schema();
        let obj = match schema.schema_kind {
            SchemaKind::Type(Type::Object(obj)) => obj,
            _ => panic!("unsupported schema for query parameters"),
        };

        for (name, schema) in &obj.properties {
            op.parameters.push(ReferenceOr::Item(Parameter::Query {
                parameter_data: ParameterData {
                    name: name.clone(),
                    description: None,
                    required,
                    deprecated: None,
                    format: ParameterSchemaOrContent::Schema(match schema.clone() {
                        ReferenceOr::Reference { reference } => {
                            ReferenceOr::Reference { reference }
                        }
                        ReferenceOr::Item(item) => ReferenceOr::Item(*item),
                    }),
                    example: None,
                    examples: Default::default(),
                    explode: None,
                    extensions: Default::default(),
                },
                allow_reserved: false,
                style: QueryStyle::default(),
                allow_empty_value: None,
            }))
        }
    }
}

impl OperationResult for () {
    fn modify_op(operation: &mut Operation) {
        operation.responses.default = Some(ReferenceOr::Item(Response {
            description: "Default OK response".to_string(),
            headers: Default::default(),
            content: Default::default(),
            links: Default::default(),
            extensions: Default::default(),
        }));
    }
}
