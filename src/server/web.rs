use axum::{
    response::{Html, Response},
    http::{StatusCode, header},
};

pub async fn serve_index() -> Result<Response, StatusCode> {
    let html = include_str!("../../web/index.html");
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(html.to_string().into())
        .unwrap())
}

pub async fn serve_css() -> Result<Response, StatusCode> {
    let css = include_str!("../../web/styles.css");
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css")
        .body(css.to_string().into())
        .unwrap())
}

pub async fn serve_js() -> Result<Response, StatusCode> {
    let js = include_str!("../../web/app.js");
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        .body(js.to_string().into())
        .unwrap())
}