#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;


#[proc_macro_derive(Dieseljson)]
pub fn diesel_json_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_diesel_json_macro(&ast)
}

fn impl_diesel_json_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let proxy_name = syn::Ident::new(&format!("{}_DieselJsonProxyAsExpression", name), name.span());

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

    gen.into()
}
