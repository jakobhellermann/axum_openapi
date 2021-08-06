WIP crate for auto-generating [openapi](https://swagger.io/specification/) descriptions for web services written using the [axum](https://github.com/tokio-rs/axum/) framework.


## Example usage:

```rust,no_run
use axum::prelude::*;
use std::net::SocketAddr;

use axum_openapi::prelude::*;
use axum_openapi::{openapi_json_endpoint, openapi_yaml_endpoint};

#[tokio::main]
async fn main() {
    let app = route("/pets", get(find_pets).post(add_pet))
        .route("/pets/:id", get(find_pet_by_id).delete(delete_pet));
    let openapi = app.openapi();

    let app = app
        .route("/openapi.yaml", openapi_yaml_endpoint(openapi.clone()))
        .route("/openapi.json", openapi_json_endpoint(openapi));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    hyper::server::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn find_pets(/* */) { /* */ }
async fn add_pet(/* */) { /* */ }
async fn find_pet_by_id(/* */) { /* */ }
async fn delete_pet(/* */) { /* */ }
```

See the full example at [./examples/petstore.rs](https://github.com/jakobhellermann/axum_openapi/blob/main/examples/petstore.rs).