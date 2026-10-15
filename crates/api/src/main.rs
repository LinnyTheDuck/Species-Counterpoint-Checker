use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_axum::GraphQL;
use axum::Router;
use axum::response::{Html, IntoResponse};
use axum::routing::get;

struct Query;

#[Object]
impl Query {
    /// Engine version.
    async fn version(&self) -> &'static str {
        counterpoint_core::version()
    }
}

/// Serves the GraphiQL explorer.
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();
    let app = Router::new()
        .route("/", get(graphiql))
        .route_service("/graphql", GraphQL::new(schema));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    println!("listening on http://127.0.0.1:{port}");
    axum::serve(listener, app).await.unwrap();
}
