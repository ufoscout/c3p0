#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::Data;
use syn::Type;

#[proc_macro_derive(C3p0Model, attributes(c3p0_table))]
pub fn c3p0_model_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_c3p0model_macro(&ast)
}

const C3P0_TABLE_ATTR_NAME: &'static str = "c3p0_table";

fn impl_c3p0model_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let struct_body = match &ast.data {
        Data::Struct(body) => body,
        _ => panic!("expected a struct"),
    };

    println!("Attr value: [{:?}]", get_attr_value(ast, C3P0_TABLE_ATTR_NAME));

    let has_id = has_id(&struct_body.fields.iter().collect::<Vec<_>>());
    let ty = get_data_type(&struct_body.fields.iter().collect::<Vec<_>>());

    let gen_queryable = quote! {
        impl c3p0::C3p0ModelQueryable<#ty> for #name {
            fn c3p0_get_id(&self) -> &i64 {
                &self.id
            }

            fn c3p0_get_version(&self) -> &i32 {
                &self.version
            }

            fn c3p0_get_data(&self) -> &#ty {
                &self.data
            }
        }
    };

    let gen_insertable = quote! {
        impl c3p0::C3p0ModelInsertable<#ty> for #name {
            fn c3p0_get_version(&self) -> &i32 {
                &self.version
            }

            fn c3p0_get_data(&self) -> &#ty {
                &self.data
            }
        }
    };

    if has_id {
        return gen_queryable.into();
    }
    gen_insertable.into()
}

fn get_data_type<'a>(fields: &[&'a syn::Field]) -> &'a Type {
    for field in fields {
        let ident = &field.ident;
        let ty = &field.ty;
        if let Some(some_field) = ident {
            if some_field.to_string() == "data" {
                println!("HAS DATA!");
                return ty;
            }
        }
    }
    println!("DOES NOT HAVE DATA!");
    panic!("Expected to have field \"data\"")
}

fn has_id(fields: &[&syn::Field]) -> bool {
    for field in fields {
        let ident = &field.ident;
        if let Some(some_field) = ident {
            if some_field.to_string() == "id" {
                println!("HAS ID!");
                return true;
            }
        }
    }
    println!("DOES NOT HAVE ID!");
    false
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
    let proxy_name = syn::Ident::new(
        &format!("{}_DieselJsonProxyAsExpression", name),
        name.span(),
    );

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


fn get_attr_value(ast: &syn::DeriveInput, attr_name: &str) -> Option<String> {
    for a in &ast.attrs {
        if let Some(meta) = a.interpret_meta() {
            // println!("Found attribute: {:?}", meta.name());
            if meta.name().eq(attr_name) {
                match meta {
                    syn::Meta::NameValue(named_value) => {
                        //println!("Is NameValue");
                        match named_value.lit {
                            syn::Lit::Str(litstr) => {
                                //println!("litstr Is {}", litstr.value());
                                return Some(litstr.value())
                            },
                            _ => {}
                        }
                        //println!("value: {:?}", named_value.eq_token);
                    }
                    _ => {}
                }
            }
        }
    }
    None
}