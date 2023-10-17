#![feature(proc_macro_hygiene)]

extern crate proc_macro;

use dotenvy::dotenv;
use std::env;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::LitStr;

#[proc_macro]
pub fn fetch_hex_env(input: TokenStream) -> TokenStream {
    match dotenv() {
        Ok(_) => {}
        Err(_e) => {}
    }

    let var_name = parse_macro_input!(input as LitStr).value();

    let hex_string = match env::var(&var_name) {
        Ok(val) => val,
        Err(e) => {
            let err_msg = e.to_string();
            return TokenStream::from(quote! {
                compile_error!(concat!(
                    "Failed to fetch environment variable: ",
                    #var_name,
                    "\nError: ",
                    #err_msg
                ));
            });
        }
    };

    let hex_string = hex_string.trim_start_matches("0x").replace(" ", "");
    let hex_bytes = hex_string
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).expect("Invalid hex digit")
        })
        .collect::<Vec<_>>();

    let hex_bytes_literal = hex_bytes.iter().map(|&b| quote! {#b,});
    let hex_literal = quote! {
        [ #(#hex_bytes_literal)* ]
    };

    TokenStream::from(hex_literal)
}
