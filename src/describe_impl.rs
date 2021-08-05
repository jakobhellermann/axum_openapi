use openapiv3::*;

use crate::openapi_traits::DescribeSchema;
use crate::utils;

impl DescribeSchema for i32 {
    fn describe_schema() -> Schema {
        utils::ty_schema(Type::Integer(IntegerType {
            format: VariantOrUnknownOrEmpty::Item(IntegerFormat::Int32),
            ..Default::default()
        }))
    }
}
impl DescribeSchema for i64 {
    fn describe_schema() -> Schema {
        utils::ty_schema(Type::Integer(IntegerType {
            format: VariantOrUnknownOrEmpty::Item(IntegerFormat::Int64),
            ..Default::default()
        }))
    }
}
impl DescribeSchema for f32 {
    fn describe_schema() -> Schema {
        utils::ty_schema(Type::Number(NumberType {
            format: VariantOrUnknownOrEmpty::Item(NumberFormat::Float),
            ..Default::default()
        }))
    }
}
impl DescribeSchema for f64 {
    fn describe_schema() -> Schema {
        utils::ty_schema(Type::Number(NumberType {
            format: VariantOrUnknownOrEmpty::Item(NumberFormat::Double),
            ..Default::default()
        }))
    }
}
impl DescribeSchema for bool {
    fn describe_schema() -> Schema {
        utils::ty_schema(Type::Boolean {})
    }
}
impl DescribeSchema for String {
    fn describe_schema() -> Schema {
        utils::ty_schema(Type::String(StringType::default()))
    }
}
impl DescribeSchema for &str {
    fn describe_schema() -> Schema {
        utils::ty_schema(Type::String(StringType::default()))
    }
}
impl<T: DescribeSchema> DescribeSchema for Vec<T> {
    fn describe_schema() -> Schema {
        utils::ty_schema(Type::Array(ArrayType {
            items: ReferenceOr::Item(Box::new(T::describe_schema())),
            min_items: None,
            max_items: None,
            unique_items: false,
        }))
    }
}
impl<T: DescribeSchema, const N: usize> DescribeSchema for [T; N] {
    fn describe_schema() -> Schema {
        utils::ty_schema(Type::Array(ArrayType {
            items: ReferenceOr::Item(Box::new(T::describe_schema())),
            min_items: Some(N),
            max_items: Some(N),
            unique_items: false,
        }))
    }
}

impl<T: DescribeSchema> DescribeSchema for Option<T> {
    fn describe_schema() -> Schema {
        let mut schema = T::describe_schema();
        schema.schema_data.nullable = true;
        schema
    }
}
