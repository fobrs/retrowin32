//! Code generator for winapi functions.
//! Generates functions that pop arguments off the x86 stack.
//! TODO: move this code to lib, and switch to using a macro for codegen.

use std::io::Write;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
mod gen;

enum Attribute {
    DllExport,
}

fn parse_attr(attr: &syn::Attribute) -> anyhow::Result<Option<Attribute>> {
    if attr.path.leading_colon.is_some()
        || attr.path.segments.len() != 2
        || attr.path.segments[0].ident != "win32_derive"
    {
        return Ok(None);
    }
    let seg = &attr.path.segments[1];
    if seg.ident == "dllexport" {
        Ok(Some(Attribute::DllExport))
    } else {
        anyhow::bail!("bad win32_derive attribute")
    }
}

/// Process one module, generating the wrapper functions and resolve helper.
fn process_mod(module: &syn::Ident, path: &str) -> anyhow::Result<TokenStream> {
    let buf = std::fs::read_to_string(path)?;
    let file = syn::parse_file(&buf)?;
    let mut fns = Vec::new();
    let mut matches = Vec::new();
    for item in &file.items {
        match item {
            syn::Item::Fn(func) => {
                let mut dllexport = false;
                for attr in func.attrs.iter() {
                    if let Some(attr) = parse_attr(attr)? {
                        match attr {
                            Attribute::DllExport => dllexport = true,
                        }
                    }
                }

                if dllexport {
                    fns.push(gen::fn_wrapper(quote! { winapi::#module }, func));
                    let ident = &func.sig.ident;
                    let quoted = ident.to_string();
                    matches.push(quote!(#quoted => #ident));
                }
            }
            // syn::Item::Struct(_) => todo!(),
            _ => {}
        }
    }
    if fns.is_empty() {
        return Ok(quote!());
    }
    Ok(quote! {
        pub mod #module {
            use super::*;
            use winapi::#module::*;

            #(#fns)*
            pub fn resolve(sym: &winapi::ImportSymbol) -> Option<fn(&mut Machine)> {
                Some(match *sym {
                    winapi::ImportSymbol::Name(name) => match name {
                        #(#matches,)*
                       _ => return None,
                    }
                    _ => return None, // TODO: ordinal
                })
            }
        }
    })
}

/// Process multiple files, generating a single Rust output file.
fn process(args: std::env::Args) -> anyhow::Result<TokenStream> {
    let mut names = Vec::new();
    let mut mods = Vec::new();
    for path in args {
        let module = std::path::Path::new(&path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        let module = quote::format_ident!("{}", module);
        mods.push(process_mod(&module, &path)?);
        names.push(module);
    }
    Ok(quote! {
        /// Generated code, do not edit.

        use crate::{winapi, machine::Machine, winapi::shims::{from_x86, ToX86}, winapi::types::*};

        #(#mods)*
    })
}

fn rustfmt(tokens: &mut String) -> anyhow::Result<()> {
    // Stolen from https://github.com/microsoft/windows-rs/blob/master/crates/tools/lib/src/lib.rs
    let mut child = std::process::Command::new("rustfmt")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or(anyhow::anyhow!("no stdin"))?;
    stdin.write_all(tokens.as_bytes())?;
    drop(stdin);
    let output = child.wait_with_output()?;

    if !output.status.success() {
        anyhow::bail!("rustfmt failed: {}", std::str::from_utf8(&output.stderr)?);
    }
    *tokens = String::from_utf8(output.stdout)?;
    Ok(())
}

fn print(tokens: TokenStream) -> anyhow::Result<()> {
    //println!("{}", tokens);
    let file = syn::parse2::<syn::File>(tokens)?;
    println!("#![allow(non_snake_case)]"); // parse2 seems to fail if it sees this.
    println!("#![allow(unused_imports)]");
    let mut text = file.to_token_stream().to_string();
    rustfmt(&mut text)?;
    print!("{}", text);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    args.next();
    let tokens = process(args)?;
    print(tokens)?;
    Ok(())
}
