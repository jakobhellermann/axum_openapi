use openapiv3::*;

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use axum::body::BoxBody;
use axum::prelude::*;
use hyper::Request;

use crate::openapi_traits::OperationHandler;

type BodyFuture<'a> = Pin<Box<dyn Future<Output = hyper::Response<BoxBody>> + Send + 'a>>;

pub trait HandlerExt<B, In>: Handler<B, In> {
    fn ignore_openapi(self) -> IgnoreOpenapiHandler<Self, B, In>;
    fn with_openapi<F>(self, supplier: F) -> WithOpenapiHandler<Self, B, In, F>
    where
        F: Fn() -> Operation + Clone;
}
impl<H: Handler<B, In>, B, In> HandlerExt<B, In> for H {
    fn ignore_openapi(self) -> IgnoreOpenapiHandler<Self, B, In> {
        IgnoreOpenapiHandler(self, PhantomData)
    }

    fn with_openapi<F>(self, supplier: F) -> WithOpenapiHandler<Self, B, In, F>
    where
        F: Fn() -> Operation + Clone,
    {
        WithOpenapiHandler::new(self, supplier)
    }
}

pub struct IgnoreOpenapiHandler<H: Handler<B, In>, B, In>(H, PhantomData<fn() -> (B, In)>);
impl<H: Handler<B, In> + Clone, B, In> IgnoreOpenapiHandler<H, B, In> {
    pub fn new(handler: H) -> Self {
        Self(handler, PhantomData)
    }
    pub fn service(&self) -> &H {
        &self.0
    }
}

impl<H: Handler<B, In> + Clone, B, In> Clone for IgnoreOpenapiHandler<H, B, In> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}
impl<H: Handler<B, In> + Sized, B, In> Handler<B, In> for IgnoreOpenapiHandler<H, B, In> {
    type Sealed = axum::handler::sealed::Hidden;

    fn call<'a>(self, req: Request<B>) -> BodyFuture<'a>
    where
        Self: 'a,
    {
        self.0.call(req)
    }
}
impl<H: Handler<B, In>, B, In> OperationHandler<()> for IgnoreOpenapiHandler<H, B, In> {
    fn modify_op(&self, _: &mut OpenAPI, _: &mut Operation) {}
}

pub struct WithOpenapiHandler<H, B, In, F>(H, F, PhantomData<fn() -> (B, In)>)
where
    H: Handler<B, In>,
    F: Fn() -> Operation + Clone;

impl<H, B, In, F> WithOpenapiHandler<H, B, In, F>
where
    H: Handler<B, In>,
    F: Fn() -> Operation + Clone,
{
    pub fn new(handler: H, supplier: F) -> Self {
        Self(handler, supplier, PhantomData)
    }
    pub fn service(&self) -> &H {
        &self.0
    }
}
impl<H, B, In, F> Clone for WithOpenapiHandler<H, B, In, F>
where
    H: Handler<B, In> + Clone,
    F: Fn() -> Operation + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), PhantomData)
    }
}
impl<H: Handler<B, In> + Sized, B, In, F> Handler<B, In> for WithOpenapiHandler<H, B, In, F>
where
    F: Fn() -> Operation + Clone,
{
    type Sealed = axum::handler::sealed::Hidden;

    fn call<'a>(self, req: Request<B>) -> BodyFuture<'a>
    where
        Self: 'a,
    {
        self.0.call(req)
    }
}
impl<H: Handler<B, In>, B, In, F> OperationHandler<()> for WithOpenapiHandler<H, B, In, F>
where
    F: Fn() -> Operation + Clone,
{
    fn modify_op(&self, _: &mut OpenAPI, op: &mut Operation) {
        *op = (self.1)();
    }
}
