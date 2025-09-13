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

pub async fn large_text_handler() -> Result<Html<String>, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "large_text.html")]
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

pub async fn text_handler() -> Result<Html<String>, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "text.html")]
    struct Tmpl {
        title: String,
        words: String,
    }

    let template = Tmpl {
        title: "My title".into(),
        words: "Three Rings for the Elven-kings under the sky,
Seven for the Dwarf-lords in their halls of stone,
Nine for Mortal Men doomed to die,
One for the Dark Lord on his dark throne
In the Land of Mordor where the Shadows lie.
One Ring to rule them all, One Ring to find them,
One Ring to bring them all, and in the darkness bind them
In the Land of Mordor where the Shadows lie."
            .into(),
    };
    Ok(Html(template.render()?))
}
