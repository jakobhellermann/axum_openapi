use openapiv3::*;
use serde::Serialize;

pub fn ty_schema(ty: Type) -> Schema {
    Schema {
        schema_data: SchemaData {
            nullable: false,
            ..Default::default()
        },
        schema_kind: SchemaKind::Type(ty),
    }
}

pub fn yaml_response<T: Serialize>(body: &T) -> hyper::Response<hyper::Body> {
    let bytes = match serde_yaml::to_vec(body) {
        Ok(res) => res,
        Err(err) => {
            return hyper::Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .header(hyper::header::CONTENT_TYPE, "text/plain")
                .body(hyper::Body::from(err.to_string()))
                .unwrap();
        }
    };

    let mut res = hyper::Response::new(hyper::Body::from(bytes));
    res.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("text/x-yaml"),
    );
    res
}
