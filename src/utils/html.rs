use html::{
    content::{
        builders::{FooterBuilder, HeaderBuilder, MainBuilder},
        Footer, Header,
    },
    metadata::builders::HeadBuilder,
    root::{builders::BodyBuilder, Html},
};

use crate::Document;

pub fn get_page(doc: Document) -> Html {
    Html::builder()
        .lang("en")
        .head(|h| head(h, &doc))
        .body(|b| body(b, &doc))
        .build()
}

fn head<'a>(h: &'a mut HeadBuilder, doc: &'static Document) -> &'a mut HeadBuilder {
    h.meta(|m| m.charset("UTF-8"))
        .meta(|m| {
            m.name("viewport")
                .content("width=device-width, initial-scale=1.0")
        })
        .link(|l| l.rel("stylesheet").href("/style.css"))
        .title(|t| t.text(&doc.metadata.title))
}

fn body<'a>(b: &'a mut BodyBuilder, doc: &Document) -> &'a mut BodyBuilder {
    b.header(|h| header(h, &doc))
        .main(|m| main(m, &doc))
        .footer(|f| footer(f, &doc))
}

fn header<'a>(h: &'a mut HeaderBuilder, doc: &Document) -> &'a mut HeaderBuilder {
    h
}

fn main<'a>(m: &'a mut MainBuilder, doc: &Document) -> &'a mut MainBuilder {
    m
}

fn footer<'a>(f: &'a mut FooterBuilder, doc: &Document) -> &'a mut FooterBuilder {
    let email = "contact@daniellaing.com";
    let socials = [
        ("Github", "https://github.com/Bodleum"),
        ("Gitlab", "https://gitlab.com/Bodleum"),
        ("Instagram", "https://www.instagram.com/_thebakerdan"),
    ];

    f.anchor(|a| {
        a.href(format!("mailto:{}", email))
            .text(email)
            .class("email")
    })
    .unordered_list(|ul| {
        for (name, link) in socials {
            ul.list_item(|li| {
                li.class("social-list-item")
                    .anchor(|a| a.href(link).text(name).class("social-list-link"))
            });
        }
        ul
    })
}
