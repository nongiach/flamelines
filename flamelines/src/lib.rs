use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, Stmt};

use syn::visit_mut;
use syn::visit_mut::VisitMut;
use syn::{File, ImplItemMethod, ItemFn};

extern crate proc_macro;

type LineCallback = fn(&mut Stmt, &str);

// // this is unused now
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

struct FnVisitor {
    time_warning_threshold_ms: u128, // in miniseconds
}

impl FnVisitor {
    fn new(time_warning_threshold_ms: u128) -> Self {
        Self {
            time_warning_threshold_ms,
        }
    }

    fn default_hook_functions(&self) -> Vec<Stmt> {
        let time_warning_threshold_ms = self.time_warning_threshold_ms;
        parse_quote!(
        use crate::DEPTH;
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
            if elapsed_time > #time_warning_threshold_ms {
                log::warn!(
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
        )
    }

    /// This takes a block a parameter and instrument every single lines
    /// It also makes sure that the return values is not altered
    fn patch_block(
        &self,
        block: &mut Vec<Stmt>,
        _hook_callback: LineCallback,
        function_name: &str,
    ) {
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
        new_block.extend(self.default_hook_functions());
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
}

#[proc_macro_attribute]
pub fn time_lines(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut syntax_tree: File = syn::parse2(input.into()).unwrap();
    let mut visitor = FnVisitor::new(5);
    visitor.visit_file_mut(&mut syntax_tree);
    syntax_tree.into_token_stream().into()
}

impl VisitMut for FnVisitor {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        visit_mut::visit_item_fn_mut(self, node);
        let function_name = node.sig.ident.to_string();
        self.patch_block(
            &mut node.block.stmts,
            wrap_line_with_instrumentation,
            &function_name,
        );
    }
    fn visit_impl_item_method_mut(&mut self, node: &mut ImplItemMethod) {
        visit_mut::visit_impl_item_method_mut(self, node);

        let function_name = format!("module::{}", node.sig.ident.to_string());
        self.patch_block(
            &mut node.block.stmts,
            wrap_line_with_instrumentation,
            &function_name,
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
