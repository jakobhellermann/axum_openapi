use openapiv3::*;

/// Trait which describes a rust type as an [`openapiv3::Schema`]
pub trait DescribeSchema {
    fn describe_schema() -> Schema;

    /// If this returns a string, then the schema will be written to the `components/schemas` section instead of inlined at its use.
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

/// Describes an [axum] app as [`openapiv3::OpenAPI`]
/// ```rust,no_run
/// use axum::prelude::*;
/// use axum_openapi::prelude::*;
/// # async fn index() {}
/// # async fn handler() {}
///
/// let app = route("/", get(index))
///     .route("/", get(handler));
/// # hyper::server::Server::bind(todo!()).serve(app.into_make_service());
///
/// println!("{:?}", app.openapi());
/// ```
pub trait OpenapiApp<Params> {
    fn modify_openapi(&self, api: &mut OpenAPI);

    fn openapi(&self) -> OpenAPI {
        let mut openapi = OpenAPI::default();
        self.modify_openapi(&mut openapi);

        fix_path_params(&mut openapi);

        openapi
    }
}

/// Implemented for [`axum::handler::get/post/...`](axum::handler)
pub trait OperationAtPath<Params> {
    fn modify_path_item(&self, openapi: &mut OpenAPI, path_item: &mut PathItem);
}

/// Describes an [`axum::handler::Handler`] as a [`openapiv3::Operation`]
pub trait OperationHandler<Params> {
    fn modify_op(&self, openapi: &mut OpenAPI, operation: &mut Operation);
}

/// Implemeted for most types in [`axum::extract`], i.e. parameters to handler functions.
pub trait OperationParameter {
    fn modify_op(openapi: &mut OpenAPI, operation: &mut Operation, required: bool);
}
/// Describes the return value of a handler function for an [`openapiv3::Operation`]
pub trait OperationResult {
    fn modify_op(openapi: &mut OpenAPI, operation: &mut Operation);
}

fn fix_path_params(openapi: &mut OpenAPI) {
    openapi.paths.iter_mut().for_each(|(path, val)| {
        let val = match val {
            ReferenceOr::Reference { .. } => return,
            ReferenceOr::Item(item) => item,
        };

        patch_operations(val, path);
    });
}

fn patch_operations(path_item: &mut PathItem, path: &str) {
    let path_params: Vec<_> = path
        .split('/')
        .filter_map(|component| {
            component
                .strip_prefix('{')
                .and_then(|component| component.strip_suffix('}'))
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
    ])
    .into_iter()
    .flatten();
    for op in ops {
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
