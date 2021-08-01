use axum::prelude::*;
use std::net::SocketAddr;

use axum_openapi::DescribeSchema;

#[tokio::main]
async fn main() {
    let app = route("/pets", get(find_pets).post(add_pet))
        .route("/pets/:id", get(find_pet_by_id).delete(delete_pet))
        .route("/openapi.yaml", get(axum_openapi::api_yaml))
        .route("/openapi.json", get(axum_openapi::api_json));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    hyper::server::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

mod model {
    use axum_openapi::DescribeSchema;

    #[derive(Debug, serde::Serialize, serde::Deserialize, DescribeSchema)]
    pub struct Pet {
        #[serde(flatten)]
        new_pet: NewPet,
        #[serde(flatten)]
        pet_extra: PetExtra,
    }
    #[derive(Debug, serde::Serialize, serde::Deserialize, DescribeSchema)]
    pub struct PetExtra {
        id: i64,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, DescribeSchema)]
    pub struct NewPet {
        name: String,
        tag: Option<String>,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, DescribeSchema)]
    pub struct Error {
        code: i32,
        message: String,
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, DescribeSchema)]
pub struct FindPetsQueryParams {
    tags: Option<Vec<String>>,
    limit: Option<i32>,
}

/// Returns all pets from the system that the user has access to
async fn find_pets(query_params: Option<axum::extract::Query<FindPetsQueryParams>>) {
    println!("find_pets called");
    println!("Query params: {:?}", query_params);
}

#[derive(Debug, serde::Serialize, serde::Deserialize, DescribeSchema)]
pub struct AddPetRequestBody {
    name: String,
    tag: Option<String>,
}

/// Creates a new pet in the store. Duplicates are allowed.
async fn add_pet(request_body: axum::extract::Json<AddPetRequestBody>) {
    println!("add_pet called");
    println!("Request body: {:?}", request_body);
}

/// Returns a user based on a single ID, if the user does not have access to the pet
async fn find_pet_by_id() {
    println!("find_pet_by_id called");
    println!("id = {}", id);
}

/// deletes a single pet based on the ID supplied
async fn delete_pet() {
    println!("delete_pet called");
    println!("id = {}", id);
}
