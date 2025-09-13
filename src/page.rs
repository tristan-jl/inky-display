use crate::AppError;
use askama::Template;
use axum::response::Html;

pub async fn large_text_handler() -> Result<Html<String>, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "large_text.html")]
    struct Tmpl {
        words: String,
    }

    let template = Tmpl {
        words: "Hello there".to_string(),
    };
    Ok(Html(template.render()?))
}
