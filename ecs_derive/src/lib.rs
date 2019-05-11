extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    DeriveInput, Path,
};

/// Custom derive macro for the `Component` trait.
///
/// ## Example
///
/// ```rust,ignore
/// extern crate map;
/// use map::VecMap;
///
/// #[derive(Component, Debug)]
/// #[storage(VecMap)] //  `VecMap` is a data structure for a storage component, This line is optional, defaults to `VecMap`
/// struct Pos(f32, f32, f32);
/// ```
#[proc_macro_derive(Component, attributes(storage))]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_component(&ast, false);
    gen.into()
}

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_component(&ast, true);
    gen.into()
}

struct StorageAttribute {
    storage: Path,
}

impl Parse for StorageAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _parenthesized_token = parenthesized!(content in input);

        Ok(StorageAttribute {
            storage: content.parse()?,
        })
    }
}

fn impl_component(ast: &DeriveInput, is_deref: bool) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let storage = ast
        .attrs
        .iter()
        .find(|attr| attr.path.segments[0].ident == "storage")
        .map(|attr| {
            syn::parse2::<StorageAttribute>(attr.tts.clone())
                .unwrap()
                .storage
        })
        .unwrap_or_else(|| parse_quote!(VecMap));

    let write = impl_write(ast, &ast.generics, is_deref);

    quote! {
        impl #impl_generics Component for #name #ty_generics #where_clause {
            type Storage = #storage<Self>;
        }

        #write
    }
}

fn impl_write(ast: &DeriveInput, generics: &syn::Generics, is_deref: bool) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let write_trait_name = ident(&(name.to_string() + "Write"));
    let trait_def = SetGetFuncs(ast);
    let trait_impl = SetGetFuncsImpl(ast, is_deref);

    let mut generics1 = generics.clone();
    generics1.params.insert(0, syn::GenericParam::Lifetime(syn::LifetimeDef::new(syn::Lifetime::new("'a", proc_macro2::Span::call_site()))));
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let (impl_generics, _, _) = generics1.split_for_impl();

    quote! {
        pub trait #write_trait_name#ty_generics #where_clause {
            #trait_def
        }

        impl#impl_generics #write_trait_name#ty_generics #where_clause for ecs::monitor::Write<'a, #name #ty_generics> #where_clause {
            #trait_impl
        }
    }
}

fn ident(sym: &str) -> syn::Ident {
    syn::Ident::new(sym, quote::__rt::Span::call_site())
}

struct SetGetFuncsImpl<'a>(&'a syn::DeriveInput, bool);

impl<'a> ToTokens for SetGetFuncsImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.0.ident;
        let (_, ty_generics, _) = self.0.generics.split_for_impl();
        match &self.0.data {
            syn::Data::Struct(s) => {
                let fields = &s.fields;
                match fields {
                    syn::Fields::Named(fields) => {
                        for field in fields.named.iter() {
                            let field_name = field.ident.as_ref().unwrap();
                            let field_name_str = field_name.clone().to_string();
                            let set_name = ident(&("set_".to_string() + field_name.clone().to_string().as_str()));
                            let ty = &field.ty;
                            // set field
                            if self.1 {
                                tokens.extend(quote! {
                                    fn #set_name(&mut self, value: #ty) {
                                        (self.value.0).#field_name = value; // TODO?
                                        self.notify.modify_event(self.id, #field_name_str, 0);
                                    } 
                                });
                            }else {
                                tokens.extend(quote! {
                                    fn #set_name(&mut self, value: #ty) {
                                        self.value.#field_name = value; // TODO?
                                        self.notify.modify_event(self.id, #field_name_str, 0);
                                    } 
                                });
                            }
                        }
                    },
                    syn::Fields::Unnamed(fields) => {
                        let mut i: usize = 0;
                        for field in fields.unnamed.iter() {
                            let set_name = ident(&("set_".to_string() + i.to_string().as_str()));
                            let ty = &field.ty;
                            let index = syn::Index::from(i);
                            // set index
                            if self.1 {
                                tokens.extend(quote! {
                                    fn #set_name(&mut self, value: #ty) {
                                        (self.value.0).#index = value; // TODO?
                                        self.notify.modify_event(self.id, "", #i);
                                    } 
                                });
                            }else {
                                tokens.extend(quote! {
                                    fn #set_name(&mut self, value: #ty) {
                                        self.value.#index = value; // TODO?
                                        self.notify.modify_event(self.id, "", #i);
                                    } 
                                });
                            }
                            i += 1;
                        }
                    },
                    syn::Fields::Unit => panic!("Unit Can not be Component"),
                };
            },
            _ => ()
        };
        // modify
        tokens.extend(quote! {
            fn modify<F: Fn(&mut #name#ty_generics)>(&mut self, callback: F) {
                callback(self.value);
                self.notify.modify_event(self.id, "", 0);
            } 
        });
    }
}

struct SetGetFuncs<'a>(&'a syn::DeriveInput);

impl<'a> ToTokens for SetGetFuncs<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.0.ident;
        let (_, ty_generics, _) = self.0.generics.split_for_impl();
        match &self.0.data {
            syn::Data::Struct(s) => {
                let fields = &s.fields;
                match fields {
                    syn::Fields::Named(fields) => {
                        for field in fields.named.iter() {
                            let field_name = field.ident.as_ref().unwrap();
                            let set_name = ident(&("set_".to_string() + field_name.clone().to_string().as_str()));
                            let ty = &field.ty;
                            // set field def
                            tokens.extend(quote! {
                                fn #set_name(&mut self, #ty);
                            });
                        }
                    },
                    syn::Fields::Unnamed(fields) => {
                        let mut i: usize = 0;
                        for field in fields.unnamed.iter() {
                            let set_name = ident(&("set_".to_string() + i.to_string().as_str()));
                            let ty = &field.ty;
                            // set index def
                            tokens.extend(quote! {
                                fn #set_name(&mut self, #ty);
                            });
                            i += 1;
                        }
                    },
                    syn::Fields::Unit => panic!("Unit Can not be Component"),
                };
            },
            _ => ()
        };
        // modify def
        tokens.extend(quote! {
            fn modify<F: Fn(&mut #name#ty_generics)>(&mut self, callback: F); 
        });
    }
}