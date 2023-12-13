use html::content::Footer;

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
