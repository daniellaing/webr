use html::{
    content::{builders::HeaderBuilder, Footer, Header},
    metadata::builders::HeadBuilder,
    root::{builders::BodyBuilder, Html},
};

pub fn get_page(title: String) -> Html {
    Html::builder()
        .lang("en")
        .head(|h| head(h, title))
        .body(|b| body(b))
        .build()
}

fn head(h: &mut HeadBuilder, title: String) -> &mut HeadBuilder {
    h.meta(|m| m.charset("UTF-8"))
        .meta(|m| {
            m.name("viewport")
                .content("width=device-width, initial-scale=1.0")
        })
        .link(|l| l.rel("stylesheet").href("/style.css"))
        .title(|t| t.text(title))
}

fn body(b: &mut BodyBuilder) -> &mut BodyBuilder {
    b.header(|h| h).main(|m| m).footer(|f| f)
}

pub fn footer<A, S>(email: S, items: A) -> Footer
where
    A: IntoIterator<Item = (S, S)>,
    S: Into<std::borrow::Cow<'static, str>> + std::fmt::Display,
{
    Footer::builder()
        .anchor(|a| {
            a.href(format!("mailto:{}", &email))
                .text(email)
                .class("email")
        })
        .unordered_list(|ul| {
            for (name, link) in items {
                ul.list_item(|li| {
                    li.class("social-list-item")
                        .anchor(|a| a.href(link).text(name).class("social-list-link"))
                });
            }
            ul
        })
        .build()
}
