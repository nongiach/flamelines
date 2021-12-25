use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, Stmt};

extern crate proc_macro;

type LineCallback = fn(&mut Stmt, &str);

fn wrap_line_with_instrumentation(original_line: &mut Stmt, function_name: &str) {
    let original_line_to_string = quote::quote!(#original_line).to_string();
    *original_line = parse_quote! {{
            DEPTH.with(|d| d.set(d.get() + 1));
            let padding = DEPTH.with(|d| d.get());
            let padding = " >".repeat(padding);
            let __flametime_before_execution = std::time::Instant::now();
            let fn_return_value = { #original_line };
            let elapsed_time = __flametime_before_execution.elapsed().as_millis();
            if elapsed_time > 50 {
                println!("{} took {}ms {} in fn {} => {}",
                    " ".repeat(20), elapsed_time, padding,
                    #function_name,
                    #original_line_to_string);
            }
            DEPTH.with(|d| d.set(d.get() - 1));
            fn_return_value
    }}
}

fn patch_block(stmts: &mut Vec<Stmt>, hook_callback: LineCallback, function_name: &str) {
    stmts
        .iter_mut()
        .map(|item| hook_callback(item, function_name))
        .for_each(drop)
}

fn patch_impl(impl_item: &mut syn::ItemImpl, hook_callback: LineCallback, module_name: &str) {
    impl_item.items.iter_mut().for_each(|item| {
        if let syn::ImplItem::Method(method_item) = item {
            let function_name = &format!("{}::{}", module_name, method_item.sig.ident);
            patch_block(&mut method_item.block.stmts, hook_callback, function_name)
        }
    })
}

#[proc_macro_attribute]
pub fn time_lines(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: syn::Item = syn::parse(input).unwrap();
    match &mut item {
        syn::Item::Fn(fn_item) => {
            let function_name = fn_item.sig.ident.to_string();
            patch_block(
                &mut fn_item.block.stmts,
                wrap_line_with_instrumentation,
                &function_name,
            );
        }
        syn::Item::Impl(ref mut impl_item) => {
            patch_impl(impl_item, wrap_line_with_instrumentation, "module");
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
