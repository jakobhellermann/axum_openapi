mod utils;
use openapiv3::*;

mod describe_impl;

pub use axum_openapi_derive::DescribeSchema;
#[doc(hidden)]
pub mod __macro {
    pub use openapiv3;
}

pub trait DescribeSchema {
    fn describe_schema() -> Schema;
}

pub trait DescribeOperation<Out, Params> {
    fn describe_operation() -> Operation;
}
pub trait OperationParameter {
    fn modify_op(operation: &mut OpenAPI);
}

/*macro_rules! impl_describe_operation {
    ($($param: ident),*) => {
        impl<Op, Fut, Out, $($param),*> DescribeOperation<Out, ($($param,)*)> for Op
        where
            Op: Fn($($param),*) -> Fut,
            Fut: Future<Output = Out>,
        {
            fn describe_operation() -> Operation {
                todo!()
            }
        }
    };
}

axum_openapi_derive::all_tuples!(impl_describe_operation, 1, 2, P);*/
