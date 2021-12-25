use proc_macro::*;
use quote::quote;
use syn::{parse_quote, Stmt};

// #printer(#entering_format, "", #(#arg_idents,)* depth = DEPTH.with(|d| d.get()));
// #printer(#exiting_format, "", fn_return_value, depth = DEPTH.with(|d| d.get()));
fn wrap_line_with_instrumentation(original_line: &mut Stmt) {
    // let entering_format = "Entering";
    // let exiting_format = "Exiting";
    *original_line = parse_quote! {{
        println!("start here {}", DEPTH.with(|d|d.get()));
        DEPTH.with(|d| d.set(d.get() + 1));
        // let mut fn_closure = move || #original_line;
        // let fn_return_value = fn_closure();
        let fn_return_value = dbg!({ #original_line });
        DEPTH.with(|d| d.set(d.get() - 1)); // ICI
        fn_return_value
    }}
}

fn insert_new_line_at_each_line(stmts: &mut Vec<Stmt>, line_wrapper_callback: fn(&mut Stmt)) {
    stmts.iter_mut().map(line_wrapper_callback).collect::<()>();
    // let mut new_stmts = Vec::new();
    // for line in stmts.clone().into_iter() {
    //     wrap_line_with_instrumentation()
    //     new_stmts.push(line);
    //     new_stmts.push(new_line.clone());
    // }
    // *stmts = new_stmts
}

#[proc_macro_attribute]
pub fn counter(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("expected fn"),
    };
    println!("block size is => {}", fn_item.block.stmts.len());

    // let first_line: Stmt = parse_quote! {{
    //     fn print_line_and_time(__flameline_time: &mut std::time::Instant) {
    //         let __flameline_elapsed_time = __flameline_time.elapsed();
    //         println!("took {} seconds.", __flameline_elapsed_time.as_secs());
    //         *__flameline_time = std::time::Instant::now();
    //     }
    //     let mut __flameline_time = std::time::Instant::now();
    // }};

    // let each_line: Stmt = parse_quote! {{
    //     // print_line_and_time(&mut __flameline_time);
    //     // println!("A");
    //     // let __flameline_elapsed_time = __flameline_time.elapsed();
    //     // __flameline_time = std::time::Instant::now();
    // }};
    // let each_line: Stmt = syn::parse(
    //     quote!(
    //         // println!("took {} seconds.", __flameline_time.elapsed().as_secs());
    //         println!("took seconds.");
    //         // let __flameline_time = std::time::Instant::now();
    //     )
    //     .into(),
    // )
    // .unwrap();

    insert_new_line_at_each_line(&mut fn_item.block.stmts, wrap_line_with_instrumentation);
    // fn_item.block.stmts.insert(0, first_line);
    // fn_item.block.stmts.pop();

    use quote::ToTokens;
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
