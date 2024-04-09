
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{ImplItem, ItemImpl, Token, Type};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

pub struct MessageSetImpl {
    item_impl: ItemImpl,
    ident: Ident,
}

impl Parse for MessageSetImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item_impl: ItemImpl = input.parse()?;

        let ident = match item_impl.self_ty.as_ref() {
            Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .as_ref()
                .ok_or_else(|| syn::Error::new(type_path.path.span(), "missing ident from path"))?
                .ident
                .clone(),
            _ => {
                return Err(syn::Error::new(
                    item_impl.self_ty.span(),
                    "expected a path or ident",
                ))
            }
        };

        Ok(MessageSetImpl { item_impl, ident })
    }
}
impl ToTokens for MessageSetImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let item_impl = &self.item_impl;

        let mut handler_ident: Option<Type> = None;
        let mut async_msg_idents: Punctuated<Type, Token![,]> = Punctuated::default();
        let mut sync_msg_idents: Punctuated<Type, Token![,]> = Punctuated::default();
        let mut async_concurrent_msg_idents: Punctuated<Type, Token![,]> = Punctuated::default();
        let mut sync_concurrent_msg_idents: Punctuated<Type, Token![,]> = Punctuated::default();

        for item in item_impl.items.iter() {
            match item {
                ImplItem::Type(item_type) => {
                    let item_type_name = item_type.ident.to_string();
                    match &*item_type_name {
                        "Handler" => {
                            handler_ident = Some(item_type.ty.clone());
                        }
                        "Async" => {
                            let Type::Tuple(tuple) = &item_type.ty else {
                                panic!("Async type must is tuple");
                            };
                            // tuple.elems.iter().map(|n| {
                            //     match n {
                            //         // Type::ImplTrait(_) => {}
                            //         // Type::TraitObject(_) => {}
                            //         // Type::Tuple(_) => {}
                            //         Type::Path(n) => {
                            //             n.path
                            //         }
                            //         _ => {
                            //             panic!("{n:?} msg type is supported");
                            //         }
                            //     }
                            // })
                            async_msg_idents = tuple.elems.clone();
                        }
                        "Sync" => {
                            let Type::Tuple(tuple) = &item_type.ty else {
                                panic!("Async type must is tuple");
                            };
                            sync_msg_idents = tuple.elems.clone();
                        }
                        "AsyncConcurrent" => {
                            let Type::Tuple(tuple) = &item_type.ty else {
                                panic!("Async type must is tuple");
                            };
                            async_concurrent_msg_idents = tuple.elems.clone();
                        }
                        "SyncConcurrent" => {
                            let Type::Tuple(tuple) = &item_type.ty else {
                                panic!("Async type must is tuple");
                            };
                            sync_concurrent_msg_idents = tuple.elems.clone();
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }

        let handler_ident = handler_ident.unwrap();
        let ident = &self.ident;
        let actor_info_impl = {
            let async_msg = format_ident!("{}AsyncVariant", ident);
            let sync_msg = format_ident!("{}SyncVariant", ident);
            let async_concurrent_msg = format_ident!("{}AsyncConcurrentVariant", ident);
            let sync_concurrent_msg = format_ident!("{}SyncConcurrentVariant", ident);
            quote! {
                impl MessageVariantSet for #ident {
                    type AsyncVariant = #async_msg;
                    type SyncVariant = #sync_msg;
                    type AsyncConcurrentVariant = #async_concurrent_msg;
                    type SyncConcurrentVariant = #sync_concurrent_msg;
                }
            }
        };
        tokens.extend(quote! {
            #item_impl
            msg_channel::internal::impl_msg_handle_for_variant!(#ident;#handler_ident;Async;HandleAsync;#async_msg_idents);
            msg_channel::internal::impl_sync_msg_handle_for_variant!(#ident;#handler_ident;Sync;HandleSync;#sync_msg_idents);
            msg_channel::internal::impl_concurrent_msg_handle_for_variant!(#ident;#handler_ident;AsyncConcurrent;HandleAsyncConcurrent;#async_concurrent_msg_idents);
            msg_channel::internal::impl_sync_concurrent_msg_handle_for_variant!(#ident;#handler_ident;SyncConcurrent;HandleSyncConcurrent;#sync_concurrent_msg_idents);
            #actor_info_impl
        });
    }
}
