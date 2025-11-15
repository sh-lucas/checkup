use poem::{Route, get, handler, web::Json};

pub fn with_routes(app: Route) -> Route {
    app.at("/", get(hello)).at("/test", get(test))
}

#[handler]
fn hello() -> &'static str {
    "Hello, world!"
}

#[derive(Debug, serde::Serialize)]
struct Poem {
    title: String,
    description: String,
}

#[handler]
fn test() -> Json<Poem> {
    let poem = Poem {
        title: "The Road Not Taken".to_string(),
        description: "Cool poem about choices.".to_string(),
    };
    Json(poem)
}
