use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::Router;
use axum::http::{HeaderValue, Method, header, request};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use tower_http::cors::{AllowOrigin, CorsLayer};

mod schema;

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

/// Returns true for a page served from this machine, such as the site under `php -S`.
fn is_local_origin(origin: &HeaderValue, _: &request::Parts) -> bool {
    let origin = origin.to_str().unwrap_or("");
    ["http://localhost:", "http://127.0.0.1:"]
        .iter()
        .any(|prefix| origin.starts_with(prefix))
}

#[tokio::main]
async fn main() {
    let schema = schema::build();
    let app = Router::new()
        .route("/", get(graphiql))
        .route_service("/graphql", GraphQL::new(schema))
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(is_local_origin))
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        );

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    println!("listening on http://127.0.0.1:{port}");
    axum::serve(listener, app).await.unwrap();
}
