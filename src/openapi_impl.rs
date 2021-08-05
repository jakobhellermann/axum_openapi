use std::borrow::Cow;

use crate::openapi_traits::{
    OpenapiApp, OperationAtPath, OperationHandler, OperationParameter, OperationResult,
};
use openapiv3::*;

use axum::handler::IntoService;
use axum::routing::EmptyRouter;
use axum::routing::MethodFilter;
use axum::routing::Route;

macro_rules! impl_function_operation {
    ( $($param:ident),* ) => {

        impl<F, Ret, Fut, $($param: OperationParameter,)*> OperationHandler<(Ret, $($param,)*)> for F
        where
            F: Fn($($param,)*) -> Fut,
            Fut: std::future::Future<Output = Ret>,
            Ret: OperationResult
        {
            #[allow(unused)]
            fn modify_op(&self, openapi: &mut OpenAPI, op: &mut Operation) {
                 $(<$param as OperationParameter>::modify_op(openapi, op, true);)*
            }
        }
    };
}
axum_openapi_derive::all_tuples!(impl_function_operation, 0, 16, P);

impl<Params, FallbackParams, H, B, T, Fallback> OperationAtPath<(Params, FallbackParams)>
    for axum::handler::OnMethod<IntoService<H, B, T>, Fallback>
where
    H: OperationHandler<Params>,
    Fallback: OperationAtPath<FallbackParams>,
{
    fn modify_path_item(&self, openapi: &mut OpenAPI, path_item: &mut PathItem) {
        Fallback::modify_path_item(&self.fallback, openapi, path_item);
        match self.method {
            MethodFilter::Get => H::modify_op(
                &self.svc.handler,
                openapi,
                path_item.get.get_or_insert_with(Default::default),
            ),
            MethodFilter::Post => H::modify_op(
                &self.svc.handler,
                openapi,
                path_item.post.get_or_insert_with(Default::default),
            ),
            MethodFilter::Patch => H::modify_op(
                &self.svc.handler,
                openapi,
                path_item.patch.get_or_insert_with(Default::default),
            ),
            MethodFilter::Delete => H::modify_op(
                &self.svc.handler,
                openapi,
                path_item.delete.get_or_insert_with(Default::default),
            ),
            MethodFilter::Head => H::modify_op(
                &self.svc.handler,
                openapi,
                path_item.head.get_or_insert_with(Default::default),
            ),
            MethodFilter::Options => H::modify_op(
                &self.svc.handler,
                openapi,
                path_item.options.get_or_insert_with(Default::default),
            ),
            MethodFilter::Put => H::modify_op(
                &self.svc.handler,
                openapi,
                path_item.put.get_or_insert_with(Default::default),
            ),
            MethodFilter::Trace => H::modify_op(
                &self.svc.handler,
                openapi,
                path_item.trace.get_or_insert_with(Default::default),
            ),
            MethodFilter::Any | MethodFilter::Connect => todo!(),
        }
    }
}
impl OperationAtPath<()> for EmptyRouter {
    fn modify_path_item(&self, _: &mut OpenAPI, _: &mut PathItem) {}
}

impl<Service, Fallback, ServiceParams, FallbackParams> OpenapiApp<(ServiceParams, FallbackParams)>
    for Route<Service, Fallback>
where
    Service: OperationAtPath<ServiceParams>,
    Fallback: OpenapiApp<FallbackParams>,
{
    fn modify_openapi(&self, api: &mut OpenAPI) {
        let mut path_item = PathItem::default();
        OpenapiApp::modify_openapi(&self.fallback, api);
        OperationAtPath::modify_path_item(&self.svc, api, &mut path_item);
        let path = axum_path_to_openapi(&self.path);
        api.paths.insert(path, ReferenceOr::Item(path_item));
    }
}
impl OpenapiApp<()> for EmptyRouter {
    fn modify_openapi(&self, _: &mut OpenAPI) {}
}

fn axum_path_to_openapi(path: &str) -> String {
    let mut string = String::with_capacity(path.len());
    let iter = path
        .split('/')
        .map(|segment| match segment.strip_prefix(':') {
            None => Cow::Borrowed(segment),
            Some(name) => Cow::Owned(format!("{{{}}}", name)),
        });
    for segment in iter {
        string.push_str(&*segment);
        string.push('/');
    }

    string.truncate(string.len() - 1);
    string
}

#[cfg(test)]
mod tests {
    use super::axum_path_to_openapi;

    #[test]
    fn axum_path() {
        assert_eq!(axum_path_to_openapi("/pets"), "/pets");
        assert_eq!(axum_path_to_openapi("/pets/:id"), "/pets/{id}");
        assert_eq!(axum_path_to_openapi("/pets/:id/"), "/pets/{id}/");
        assert_eq!(axum_path_to_openapi("pets/:id/"), "pets/{id}/");
    }
}
