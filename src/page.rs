use askama::Template;
use axum::response::{Html, IntoResponse};

use crate::AppError;

pub async fn page_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "page.html")]
    struct Tmpl {
        title: String,
    }

    let template = Tmpl {
        title: "My title".to_owned(),
    };
    Ok(Html(template.render()?))
}

pub async fn text_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "text.html")]
    struct Tmpl {
        title: String,
        words: String,
    }

    let template = Tmpl {
        title: "My title".into(),
        words: "Hello there".into(),
    };
    Ok(Html(template.render()?))
}
