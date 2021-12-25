use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, Stmt};

extern crate proc_macro;

type LineCallback = fn(&mut Stmt, &str);

// this is unused now
fn wrap_line_with_instrumentation(_original_line: &mut Stmt, _function_name: &str) {
    // let original_line_to_string = quote::quote!(#original_line).to_string();
    // *original_line = parse_quote! {{
    //         DEPTH.with(|d| d.set(d.get() + 1));
    //         let padding = DEPTH.with(|d| d.get());
    //         let padding = " >".repeat(padding);
    //         let __flametime_before_execution = std::time::Instant::now();
    //         let fn_return_value = { #original_line };
    //         let elapsed_time = __flametime_before_execution.elapsed().as_millis();
    //         if elapsed_time > 50 {
    //             println!("{} took {}ms {} in fn {} => {}",
    //                 " ".repeat(20), elapsed_time, padding,
    //                 #function_name,
    //                 #original_line_to_string);
    //         }
    //         DEPTH.with(|d| d.set(d.get() - 1));
    //         fn_return_value
    // }}
}

fn patch_block(block: &mut Vec<Stmt>, _hook_callback: LineCallback, function_name: &str) {
    let hook_functions: Vec<Stmt> = parse_quote!(
        fn __flamelines_before_hook() -> std::time::Instant {
            DEPTH.with(|d| d.set(d.get() + 1));
            std::time::Instant::now()
        }
        fn __flamelines_after_hook(
            time_before: std::time::Instant,
            function_name: &str,
            original_line_to_string: &str,
        ) {
            let padding = DEPTH.with(|d| d.get());
            let padding = " >".repeat(padding);
            let elapsed_time = time_before.elapsed().as_millis();
            if elapsed_time > 50 {
                println!(
                    "{} took {}ms {} in fn {} => {}",
                    " ".repeat(20),
                    elapsed_time,
                    padding,
                    function_name,
                    original_line_to_string
                );
            }
            DEPTH.with(|d| d.set(d.get() - 1));
        }
    );
    let before_line: Vec<Stmt> = parse_quote!(
        let __flametime_before_execution = __flamelines_before_hook();
    );
    fn after_line(
        function_name: &str,
        _original_line: &Stmt,
        original_line_to_string: &str,
    ) -> Vec<Stmt> {
        parse_quote! (
            __flamelines_after_hook(__flametime_before_execution, #function_name, #original_line_to_string);
        )
    }
    let mut new_block: Vec<Stmt> = Vec::new();
    new_block.extend(hook_functions);
    for (i, original_line) in block.iter().enumerate() {
        let original_line_to_string = quote::quote!(#original_line).to_string();
        new_block.extend(before_line.clone());
        if i < (block.len() - 1) {
            new_block.push(original_line.clone());
            new_block.extend(after_line(
                function_name,
                original_line,
                &original_line_to_string,
            ));
            // otherwise insert the line as is
        } else {
            // if last line then save the result and return it
            new_block.push(parse_quote! {{
                let fn_return_value = { #original_line };
                __flamelines_after_hook(__flametime_before_execution, #function_name, #original_line_to_string);
                fn_return_value
            }});
        }
    }
    *block = new_block;
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
