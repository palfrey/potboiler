extern crate proc_macro;
extern crate quote;

use proc_macro::{TokenTree, TokenStream};
use antidote;
use std::collections::HashMap;
use std::cell::RefCell;
use quote::quote;

thread_local!(static LOCKS: RefCell<HashMap<String, antidote::Mutex<()>>> = RefCell::new(HashMap::new()));

#[proc_macro_attribute]
pub fn serial(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = attr.into_iter().collect::<Vec<TokenTree>>();
    if attrs.len() != 1 {
        panic!("Expected a single argument");
    }
    if let TokenTree::Ident(id) = &attrs[0] {
        let gen = quote! {
            let name = id.to_string();
            LOCKS.with(|ll| {
                let mut local_lock = ll.borrow_mut();
                if !local_lock.contains_key(&name) {
                    local_lock.insert(name.clone(), antidote::Mutex::new(()));
                }
                let _guard = local_lock[&name].lock();
                //return item;
            });
        };
        return gen.into();
    }
    else {
        panic!("Expected a single name as argument");
    }
}