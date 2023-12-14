#![recursion_limit = "512"]

use webr::{create_footer, get_page, init, prelude::*};

fn main() -> Result<()> {
    init()?;

    let page = get_page(String::from("Test page"));
    println!("{page}");

    Ok(())
}
