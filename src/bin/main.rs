use webr::{get_page, init, prelude::*};

fn main() -> Result<()> {
    init()?;

    let page = get_page(String::from("Test page"));
    println!("{page}");

    Ok(())
}
