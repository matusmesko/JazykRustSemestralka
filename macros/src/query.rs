use proc_macro::TokenStream;
use quote::{quote};
use syn::{parse_macro_input, ItemFn, LitStr, FnArg, Pat};

pub fn query_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let sql_query = parse_macro_input!(attr as LitStr);
    let func = parse_macro_input!(item as ItemFn);

    let func_name = &func.sig.ident;
    let func_vis = &func.vis;
    let func_async = &func.sig.asyncness;
    let func_inputs = &func.sig.inputs;
    let func_output = &func.sig.output;

    let mut bind_params = Vec::new();
    for input in func_inputs {
        if let FnArg::Typed(pat_type) = input {
            if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                let param_name = &pat_ident.ident;
                if param_name != "pool" {
                    bind_params.push(param_name);
                }
            }
        }
    }

    let type_str = quote!(#func_output).to_string().replace(" ", "");

    let is_void_result = type_str.contains("<()>") || type_str.ends_with("->()");
    let is_u64_result = type_str.contains("<u64>");

    let is_returning_rows = !is_void_result && !is_u64_result &&
        (type_str.contains("Vec<") || type_str.contains("Option<") || type_str.contains("Result<"));

    let query_execution = if is_returning_rows {
        let fetch_method = if type_str.contains("Vec<") {
            quote!(fetch_all(pool))
        } else if type_str.contains("Option<") {
            quote!(fetch_optional(pool))
        } else {
            quote!(fetch_one(pool))
        };

        quote! {
            let mut query = ::sqlx::query_as(#sql_query);
            #( query = query.bind(#bind_params); )*
            query.#fetch_method.await.map_err(::anyhow::Error::from)
        }
    } else {
        let return_expr = if is_void_result {
            quote!(Ok(()))
        } else {
            quote!(Ok(result.rows_affected()))
        };

        quote! {
            let mut query = ::sqlx::query(#sql_query);
            #( query = query.bind(#bind_params); )*
            let result = query.execute(pool).await?;
            #return_expr
        }
    };

    let expanded = quote! {
        #func_vis #func_async fn #func_name(#func_inputs) #func_output {
            #query_execution
        }
    };

    TokenStream::from(expanded)
}