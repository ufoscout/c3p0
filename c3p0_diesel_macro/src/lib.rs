#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use proc_macro2::Literal;
use proc_macro2::Span;
use quote::quote;
use syn;
use syn::Ident;

#[proc_macro_derive(C3p0Model, attributes(table_name))]
pub fn c3p0_model_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_c3p0model_macro(&ast)
}

const C3P0_TABLE_ATTR_NAME: &str = "table_name";

fn impl_c3p0model_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    /*
    let struct_body = match &ast.data {
        Data::Struct(body) => body,
        _ => panic!("expected a struct"),
    };
    */

    let table_name = get_attr_value(ast, C3P0_TABLE_ATTR_NAME).unwrap_or_else(|| {
        panic!(
            "C3p0Model macro requires the {} attribute to be specified.",
            C3P0_TABLE_ATTR_NAME
        )
    });

    let table_name = Ident::new(&table_name, Span::call_site());

    let gen_diesel_json_proxy = build_diesel_json_proxy(name);
    let gen_c3p0_model = build_c3p0_model(name);
    let gen_c3p0_new_model = build_c3p0_new_model(name, &table_name);

    let gen = quote! {
        #gen_diesel_json_proxy
        #gen_c3p0_new_model
        #gen_c3p0_model
    };

    gen.into()
}

#[proc_macro_derive(DieselJson)]
pub fn diesel_json_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_diesel_json_macro(&ast)
}

fn impl_diesel_json_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen_diesel_json_proxy = build_diesel_json_proxy(name);
    gen_diesel_json_proxy.into()
}

fn build_diesel_json_proxy(name: &Ident) -> proc_macro2::TokenStream {
    let proxy_name = syn::Ident::new(&format!("{}DieselJsonProxyAsExpression", name), name.span());

    let gen_proxy = quote! {
        #[derive(FromSqlRow, AsExpression)]
        #[diesel(foreign_derive)]
        #[sql_type = "diesel::sql_types::Json"]
        #[sql_type = "diesel::sql_types::Jsonb"]
        struct #proxy_name(#name);
    };

    let gen_json_from = quote! {
        impl diesel::deserialize::FromSql<diesel::sql_types::Json, diesel::pg::Pg> for #name {
            fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
                let bytes = not_none!(bytes);
                serde_json::from_slice(bytes).map_err(Into::into)
            }
        }
    };

    let gen_json_to = quote! {
        impl diesel::serialize::ToSql<diesel::sql_types::Json, diesel::pg::Pg> for #name {
            fn to_sql<W: std::io::Write>(&self, out: &mut diesel::serialize::Output<W, diesel::pg::Pg>) -> diesel::serialize::Result {
                serde_json::to_writer(out, self)
                    .map(|_| diesel::serialize::IsNull::No)
                    .map_err(Into::into)
            }
        }
    };

    let gen_jsonb_from = quote! {
        impl diesel::deserialize::FromSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for #name {
            fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
                let bytes = not_none!(bytes);
                if bytes[0] != 1 {
                    return Err("Unsupported JSONB encoding version".into());
                }
                serde_json::from_slice(&bytes[1..]).map_err(Into::into)
            }
        }
    };

    let gen_jsonb_to = quote! {
        impl diesel::serialize::ToSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for #name {
            fn to_sql<W: std::io::Write>(&self, out: &mut diesel::serialize::Output<W, diesel::pg::Pg>) -> diesel::serialize::Result {
                out.write_all(&[1])?;
                serde_json::to_writer(out, self)
                    .map(|_| diesel::serialize::IsNull::No)
                    .map_err(Into::into)
            }
        }
    };

    let gen = quote! {
        #gen_proxy
        #gen_json_from
        #gen_json_to
        #gen_jsonb_from
        #gen_jsonb_to
    };

    gen
}

fn build_c3p0_new_model(name: &Ident, table_name: &Ident) -> proc_macro2::TokenStream {
    let model_name = syn::Ident::new(&format!("New{}Model", name), name.span());

    let table_literal = Literal::string(&format!("{}", table_name));

    let gen = quote! {
        #[derive(Insertable)]
        #[table_name = #table_literal]
        pub struct #model_name {
            pub version: i32,
            pub data: #name,
        }
    };

    gen
}

fn build_c3p0_model(name: &Ident) -> proc_macro2::TokenStream {
    let model_name = syn::Ident::new(&format!("{}Model", name), name.span());

    let gen = quote! {
        #[derive(Queryable)]
        pub struct #model_name {
            pub id: i64,
            pub version: i32,
            pub data: #name,
        }
    };

    gen
}

fn get_attr_value(ast: &syn::DeriveInput, attr_name: &str) -> Option<String> {
    for a in &ast.attrs {
        if let Some(meta) = a.interpret_meta() {
            // println!("Found attribute: {:?}", meta.name());
            if meta.name().eq(attr_name) {
                if let syn::Meta::NameValue(named_value) = meta {
                    //println!("Is NameValue");
                    if let syn::Lit::Str(litstr) = named_value.lit {
                        //println!("litstr Is {}", litstr.value());
                        return Some(litstr.value());
                    }
                    //println!("value: {:?}", named_value.eq_token);
                }
            }
        }
    }
    None
}
