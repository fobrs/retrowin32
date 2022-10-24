use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, TypeReference};

#[proc_macro_attribute]
pub fn winapi_macro_derive(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::Item);
    let tokens = impl_hello_macro(&input);
    println!("tok:: {} ::tok", tokens);
    tokens
}

fn impl_hello_macro(item: &syn::Item) -> TokenStream {
    let func = match item {
        syn::Item::Fn(func) => func,
        _ => unimplemented!("item {:?}", item),
    };
    let mut pops: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut args: Vec<proc_macro2::TokenStream> = Vec::new();
    for (i, arg) in func.sig.inputs.iter().enumerate() {
        let arg = match arg {
            syn::FnArg::Typed(arg) => arg,
            _ => unimplemented!(),
        };

        let name = match &*arg.pat {
            syn::Pat::Ident(ident) => &ident.ident,
            _ => unimplemented!(),
        };
        if i == 0 {
            args.push(quote!(x86));
        } else {
            args.push(quote!(#name));
            let get = match arg.ty.as_ref() {
                syn::Type::Path(path) if path.path.is_ident("u32") => {
                    quote!(x86.pop())
                }
                syn::Type::Reference(TypeReference {
                    lifetime: None,
                    mutability: None,
                    elem,
                    ..
                }) => match elem.as_ref() {
                    syn::Type::Path(path) if path.path.is_ident("str") => {
                        quote! {{
                            let ofs = x86.pop() as usize; 
                            let strz = x86.mem[ofs..].read_strz();
                            unsafe { winapi::smuggle(strz) }
                        }}
                    }
                    _ => todo!(),
                },
                ty => unimplemented!("type {ty:?}"),
            };
            pops.push(quote! {let #name = #get;});
        }
    }

    let func_name = &func.sig.ident;
    let shim_name = quote::format_ident!("{}_shim", func_name);
    let gen = quote! {
        fn #shim_name(x86: &mut X86) -> u32 {
            #(#pops)*
            #func_name(#(#args,)*);
            0
        }
        #item
    };
    gen.into()
}
