extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct CollectionAssociation {
    pub entity_name: LitStr,
    pub from: LitStr,
    pub to: LitStr,
    pub alias: Option<LitStr>,
}

impl syn::parse::Parse for CollectionAssociation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);

        let entity_name: syn::LitStr = content.parse()?;
        content.parse::<syn::Token![,]>()?;
        let from: LitStr = content.parse()?;
        content.parse::<syn::Token![,]>()?;
        let to: LitStr = content.parse()?;
        content.parse::<syn::Token![,]>().ok();
        let alias: Option<LitStr> = if content.is_empty() {
            None
        } else {
            content.parse().ok()
        };

        Ok(CollectionAssociation {
            entity_name,
            from,
            to,
            alias,
        })
    }
}

struct DeebArgs {
    pub entity_name: Option<LitStr>,
    pub primary_key: Option<LitStr>,
    pub associations: Vec<CollectionAssociation>,
}

impl Parse for DeebArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut entity_name = None;
        let mut primary_key = None;
        let mut associations = Vec::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "name" => {
                    entity_name = Some(input.parse()?);
                }
                "primary_key" => {
                    primary_key = Some(input.parse()?);
                }
                "associate" => {
                    match input.parse::<CollectionAssociation>() {
                        Ok(a) => associations.push(a),
                        Err(_err) => {
                            return Err(syn::Error::new_spanned(
                                ident,
                                "Faield to parse association",
                            ));
                        }
                    };
                }
                _ => return Err(syn::Error::new_spanned(ident, "Unknown argument")),
            }

            // Optional comma
            let _ = input.parse::<Token![,]>();
        }

        Ok(DeebArgs {
            entity_name,
            primary_key,
            associations,
        })
    }
}

#[proc_macro_derive(Collection, attributes(deeb))]
pub fn derive_deeb(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // parse #[deeb(...)] args
    let args: Option<DeebArgs> = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("deeb"))
        .and_then(|attr| attr.parse_args::<DeebArgs>().ok());

    // Extract each arg
    let entity_name = args
        .as_ref()
        .and_then(|a| a.entity_name.as_ref())
        .map(|n| n.value())
        .unwrap_or_else(|| name.to_string().to_lowercase());

    let primary_key = args
        .as_ref()
        .and_then(|a| a.primary_key.as_ref())
        .map(|n| n.value())
        .unwrap_or_else(|| "id".to_string());

    let empty = Vec::new();
    let associations = args
        .as_ref()
        .map(|a| &a.associations)
        .unwrap_or(&empty)
        .iter()
        .map(|assoc| {
            let entity_name = &assoc.entity_name;
            let from = &assoc.from;
            let to = &assoc.to;
            let alias = &assoc.alias;

            match alias {
                Some(alias_expr) => {
                    quote! { .associate(#entity_name, #from, #to, Some(&(#alias_expr).to_string())).expect(&format!("Failed to create `{}` entity.", #entity_name)) }
                }
                None => {
                    quote! { .associate(#entity_name, #from, #to, None).expect(&format!("Failed to create `{}` entity.", #entity_name)) }
                }
            }
        });

    let expanded = quote! {
        impl #name {
            pub fn entity() -> Entity {
                let mut entity = Entity::new(#entity_name)
                    .primary_key(#primary_key);

                #(entity = entity #associations;)*

                entity
            }

            pub async fn find_one(db: &Deeb, query: Query, transaction: Option<&mut Transaction>) -> DbResult<Option<Self>> {
                Ok(db.find_one::<#name>(&Self::entity(), query, transaction).await?)
            }

            pub async fn find_many(db: &Deeb, query: Query, find_many_options: Option<FindManyOptions>, transaction: Option<&mut Transaction>) -> DbResult<Option<Vec<Self>>> {
                Ok(db.find_many::<#name>(&Self::entity(), query, find_many_options, transaction).await?)
            }

            pub async fn insert_one<InsertModel: serde::Serialize>(db: &Deeb, value: InsertModel, transaction: Option<&mut Transaction>) -> DbResult<Self> {
                Ok(db.insert_one::<InsertModel, #name>(&Self::entity(), value, transaction).await?)
            }

            pub async fn insert_many<InsertModel: serde::Serialize>(db: &Deeb, value: Vec<InsertModel>, transaction: Option<&mut Transaction>) -> DbResult<Vec<Self>> {
                Ok(db.insert_many::<InsertModel, #name>(&Self::entity(), value, transaction).await?)
            }

            pub async fn delete_one(db: &Deeb, query: Query, transaction: Option<&mut Transaction>) -> DbResult<Option<bool>> {
                Ok(db.delete_one(&Self::entity(), query, transaction).await?)
            }

            pub async fn delete_many(db: &Deeb, query: Query, transaction: Option<&mut Transaction>) -> DbResult<Option<bool>> {
                Ok(db.delete_many(&Self::entity(), query, transaction).await?)
            }

            pub async fn update_one<UpdateModel: serde::Serialize>(db: &Deeb, query: Query, update: UpdateModel, transaction: Option<&mut Transaction>) -> DbResult<Option<Self>> {
                Ok(db.update_one(&Self::entity(), query, update, transaction).await?)
            }

            pub async fn update_many<UpdateModel: serde::Serialize>(db: &Deeb, query: Query, update: UpdateModel, transaction: Option<&mut Transaction>) -> DbResult<Option<Vec<Self>>> {
                Ok(db.update_many(&Self::entity(), query, update, transaction).await?)
            }
        }
    };

    TokenStream::from(expanded)
}
