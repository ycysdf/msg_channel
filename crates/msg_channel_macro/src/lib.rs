use proc_macro::TokenStream;

use quote::ToTokens;
use syn::parse_macro_input;

use crate::msg_set::MessageSetImpl;

mod msg_set;

#[proc_macro_attribute]
pub fn msg_set(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let actor = parse_macro_input!(item as MessageSetImpl);
    {
        // let a = 1;
    }
    TokenStream::from(actor.into_token_stream())
}
