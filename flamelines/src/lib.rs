use proc_macro::*;
use quote::ToTokens;
use syn::{parse_quote, Stmt};

// #printer(#entering_format, "", #(#arg_idents,)* depth = DEPTH.with(|d| d.get()));
// #printer(#exiting_format, "", fn_return_value, depth = DEPTH.with(|d| d.get()));
type LineCallback = fn(&mut Stmt);

fn wrap_line_with_instrumentation(original_line: &mut Stmt) {
    let original_line_to_string = quote::quote!(#original_line).to_string();
    *original_line = parse_quote! {{
            DEPTH.with(|d| d.set(d.get() + 1));
            let padding = DEPTH.with(|d| d.get());
            let padding = " >".repeat(padding);
            let __flametime_before_execution = std::time::Instant::now();
            let fn_return_value = { #original_line };
            let elapsed_time = __flametime_before_execution.elapsed().as_millis();
            if elapsed_time > 50 {
                println!("{} {} took {}ms for {}", " ".repeat(20), padding, elapsed_time, #original_line_to_string);
            }
            DEPTH.with(|d| d.set(d.get() - 1));
            fn_return_value
    }}
}

fn patch_block(stmts: &mut Vec<Stmt>, hook_callback: LineCallback) {
    stmts.iter_mut().map(hook_callback).for_each(drop)
}

fn patch_impl(impl_item: &mut syn::ItemImpl, hook_callback: LineCallback) {
    impl_item.items.iter_mut().for_each(|item| {
        if let syn::ImplItem::Method(method_item) = item {
            patch_block(&mut method_item.block.stmts, hook_callback)
        }
    })
}

#[proc_macro_attribute]
pub fn time_lines(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: syn::Item = syn::parse(input).unwrap();
    match &mut item {
        syn::Item::Fn(fn_item) => {
            patch_block(&mut fn_item.block.stmts, wrap_line_with_instrumentation);
        }
        syn::Item::Impl(ref mut impl_item) => {
            patch_impl(impl_item, wrap_line_with_instrumentation);
        }
        _ => panic!("[-] flamelines expected fn"),
    };
    item.into_token_stream().into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
