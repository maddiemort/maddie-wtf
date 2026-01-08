use maud::{html, Markup, DOCTYPE};

use crate::{state::Theme, templates::partials};

pub async fn base(title: Option<&str>, theme: Theme, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en-GB" dir="ltr" {
            (partials::head(title, theme).await)
            body {
                script {
                    "let FF_FOUC_FIX;"
                }

                header class="siteheader" role="banner" {
                    a href="/" class="sitetitle" {
                        "Madeleine Mortensen"
                    }

                    nav role="navigation" {
                        ul {
                            li { a href="/projects" { "projects" } }
                            li { a href="/posts" { "posts" } }
                            li { a href="/chrono" { "chrono" } }
                            li { a href="/tags" { "tags" } }
                        }
                    }
                }

                (content)

                (partials::footer().await)
            }
        }
    }
}
