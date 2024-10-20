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

                header class="siteheader" {
                    h1 class="sitetitle" {
                        a href="/" {
                            "maddie, wtf?!"
                        }
                    }

                    nav {
                        ul {
                            li { a href="/posts" { "posts" } }
                            li { a href="/chrono" { "chrono" } }
                            // li { a href="/tags" { "tags" } }
                            li { a href="/read" { "read" } }
                        }
                    }
                }

                (content)

                (partials::footer().await)
            }
        }
    }
}
