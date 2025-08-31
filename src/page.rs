use askama::Template;
use axum::response::Html;

use crate::AppError;

pub async fn page_handler() -> Result<Html<String>, AppError> {
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

pub async fn text_handler() -> Result<Html<String>, AppError> {
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
