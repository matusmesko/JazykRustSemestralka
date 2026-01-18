use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Attribute, LitStr};

pub fn entity_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let table_name = parse_macro_input!(attr as LitStr);
    let mut input = parse_macro_input!(item as DeriveInput);
    let name = input.ident.clone();

    let mut col_definitions = Vec::new();
    let mut insert_columns = Vec::new();
    let mut insert_bindings = Vec::<proc_macro2::TokenStream>::new();
    let mut id_field_name = None;

    if let Data::Struct(ref s) = input.data {
        if let Fields::Named(ref fields) = s.fields {
            for field in &fields.named {
                let field_ident = field.ident.as_ref().unwrap();
                let col_name = field_ident.to_string();

                let is_id = has_attribute(&field.attrs, "id");
                let custom_type = get_attribute_value(&field.attrs, "value")
                    .map(|lit| lit.value());

                if is_id {
                    id_field_name = Some(field_ident.clone());
                }

                let sql_type = if let Some(ct) = custom_type {
                    ct
                } else {
                    match_sql_type(&field.ty)
                };

                let mut def = format!("{} {}", col_name, sql_type);
                if is_id {
                    def.push_str(" PRIMARY KEY AUTO_INCREMENT");
                }
                col_definitions.push(def);

                if !is_id {
                    insert_columns.push(col_name);
                    insert_bindings.push(quote! { .bind(&self.#field_ident) });
                }
            }
        }
    }

    let id_field = id_field_name.expect("Each Entity must have one field marked with #[id]");
    let id_column = id_field.to_string();

    if let Data::Struct(ref mut data_struct) = input.data {
        if let Fields::Named(ref mut fields) = data_struct.fields {
            for field in fields.named.iter_mut() {
                field.attrs.retain(|attr| {
                    !attr.path().is_ident("id") && !attr.path().is_ident("value")
                });
            }
        }
    }

    let create_sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table_name.value(),
        col_definitions.join(", ")
    );

    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name.value(),
        insert_columns.join(", "),
        vec!["?"; insert_columns.len()].join(", ")
    );

    let expanded = quote! {
        #input

        impl #name {
            pub const TABLE_NAME: &'static str = #table_name;

            pub async fn create_table(pool: &::sqlx::MySqlPool) -> ::anyhow::Result<()> {
                ::sqlx::query(#create_sql).execute(pool).await?;
                Ok(())
            }

            pub async fn save(&self, pool: &::sqlx::MySqlPool) -> ::anyhow::Result<()> {
                ::sqlx::query(#insert_sql)
                    #(#insert_bindings)*
                    .execute(pool).await?;
                Ok(())
            }

            pub async fn delete_by_id(id: i64, pool: &::sqlx::MySqlPool) -> ::anyhow::Result<()> {
                let sql = format!("DELETE FROM {} WHERE {} = ?", Self::TABLE_NAME, #id_column);
                ::sqlx::query(&sql).bind(id).execute(pool).await?;
                Ok(())
            }
        }

        ::inventory::submit! {
            ::library::entity_registry::EntityRegistry {
                name: #table_name,
                run: |pool| Box::pin(async move { #name::create_table(pool).await }),
            }
        }
    };

    TokenStream::from(expanded)
}

fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

fn get_attribute_value(attrs: &[Attribute], name: &str) -> Option<LitStr> {
    attrs.iter()
        .find(|a| a.path().is_ident(name))
        .and_then(|a| a.parse_args::<LitStr>().ok())
}

fn match_sql_type(ty: &syn::Type) -> String {
    let type_name = ty.to_token_stream().to_string();
    match type_name.as_str() {
        "u32" | "i32" | "u64" | "i64" => "BIGINT".to_string(),
        "String" => "VARCHAR(255)".to_string(),
        "bool" => "BOOLEAN".to_string(),
        "f64" => "DOUBLE".to_string(),
        _ => "TEXT".to_string(),
    }
}
