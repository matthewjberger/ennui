mod data;
mod queries;

use proc_macro::TokenStream;

#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn reflect(input: TokenStream) -> TokenStream {
    let item = queries::read::item_of(input);
    queries::write::written(&item)
        .parse()
        .expect("reflect writes valid rust")
}
