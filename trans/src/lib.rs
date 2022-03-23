extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn py(input: TokenStream) -> TokenStream {
    let token_stream_input = proc_macro2::TokenStream::from(input);
    let buffer = token_stream_input.to_string();

    let result = quote! {
        fn: #buffer = {123}
    };

    result.into()
}
#[proc_macro]
pub fn eval(input: TokenStream) -> TokenStream {
    let token_stream_input = proc_macro2::TokenStream::from(input);
    // println!("stream: {token_stream_input:?}");
    let mut iter_stream = token_stream_input.into_iter();
    let tag = iter_stream.next().unwrap().to_string();
    // println!("tag: {tag}", );
    let buffer = iter_stream.collect::<proc_macro2::TokenStream>();
    // println!("buffer: {buffer}");
    let result = quote! {
        let func: #buffer;
    };

    result.into()
}
