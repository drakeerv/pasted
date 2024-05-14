use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    routing, Form, Router,
};
use serde::Deserialize;
use tera::Context;
use ureq_multipart::MultipartBuilder;

use crate::{
    client::Client,
    constants::{TEMPLATES, URL},
    paste::get_csrftoken,
};

#[derive(Deserialize)]
struct Post {
    text: String,
    category: u8,
    tags: String,
    format: u16,
    expiration: String,
    exposure: u8,
    password: String,
    name: String,
}

pub fn get_router(client: Client) -> Router {
    Router::new()
        .route("/", routing::post(post))
        .with_state(client)
}

async fn post(State(client): State<Client>, Form(data): Form<Post>) -> impl IntoResponse {
    let csrf = get_csrftoken(client.get_html(format!("{URL}/").as_str()));

    let form = MultipartBuilder::new()
        .add_text("_csrf-frontend", &csrf)
        .unwrap()
        .add_text("PostForm[text]", &data.text)
        .unwrap()
        .add_text("PostForm[category_id]", &data.category.to_string())
        .unwrap()
        .add_text("PostForm[tag]", &data.tags)
        .unwrap()
        .add_text("PostForm[format]", &data.format.to_string())
        .unwrap()
        .add_text("PostForm[expiration]", &data.expiration.to_string())
        .unwrap()
        .add_text("PostForm[status]", &data.exposure.to_string())
        .unwrap()
        .add_text(
            "PostForm[is_password_enabled]",
            if data.password.is_empty() { "0" } else { "1" },
        )
        .unwrap()
        .add_text("PostForm[password]", &data.password)
        .unwrap()
        .add_text(
            "PostForm[is_burn]",
            if data.expiration == "B" { "1" } else { "0" },
        )
        .unwrap()
        .add_text("PostForm[name]", &data.name)
        .unwrap()
        .add_text("PostForm[is_guest]", "1")
        .unwrap()
        .finish()
        .unwrap();

    let response = client
        .get_agent()
        .post(format!("{URL}/").as_str())
        .set("Content-Type", &form.0)
        .send_bytes(&form.1)
        .unwrap();
    let paste_id = response
        .header("Location")
        .unwrap()
        .split("/")
        .last()
        .unwrap();

    Response::builder()
        .status(response.status())
        .header("Location", format!("/{paste_id}"))
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES.render("index.html", &Context::new()).unwrap(),
        ))
        .unwrap()
}