use proc_macro::TokenStream;
use quote::{quote};
use syn::{parse_macro_input, ItemImpl, LitStr, ImplItem, Attribute};

pub fn controller_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let base_path = parse_macro_input!(attr as LitStr);
    let mut item_impl = parse_macro_input!(item as ItemImpl);

    let struct_name = match &*item_impl.self_ty {
        syn::Type::Path(tp) => &tp.path.segments.last().unwrap().ident,
        _ => panic!("The #[controller] attribute must be used on a struct impl block."),
    };

    let mut routes = Vec::new();

    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            let fn_name = &method.sig.ident;

            if let Some((method_type, path)) = extract_mapping_info(&mut method.attrs) {
                let actix_method = match method_type.as_str() {
                    "getMapping" => quote!(get()),
                    "postMapping" => quote!(post()),
                    "putMapping" => quote!(put()),
                    "deleteMapping" => quote!(delete()),
                    _ => unreachable!(),
                };

                routes.push(quote! {
                    scope = scope.route(#path, ::actix_web::web::#actix_method.to(<#struct_name>::#fn_name));
                });
            }
        }
    }

    let expanded = quote! {
        #item_impl

        impl #struct_name {
            pub fn configure_routes(cfg: &mut ::actix_web::web::ServiceConfig) {
                let mut scope = ::actix_web::web::scope(#base_path);
                #(#routes)*
                cfg.service(scope);
            }
        }

        ::inventory::submit! {
            ::library::controller_registry::ControllerRegistry {
                name: #base_path,
                configure: <#struct_name>::configure_routes,
            }
        }
    };

    TokenStream::from(expanded)
}


fn extract_mapping_info(attrs: &mut Vec<Attribute>) -> Option<(String, LitStr)> {
    let mapping_names = ["getMapping", "postMapping", "putMapping", "deleteMapping"];
    let mut result = None;

    let mut index_to_remove = None;
    for (i, attr) in attrs.iter().enumerate() {
        for name in mapping_names.iter() {
            if attr.path().is_ident(name) {
                let path: LitStr = attr.parse_args().expect("Expected a path string in mapping attribute");
                result = Some((name.to_string(), path));
                index_to_remove = Some(i);
                break;
            }
        }
    }

    if let Some(i) = index_to_remove {
        attrs.remove(i);
    }

    result
}