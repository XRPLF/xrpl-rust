#![no_std]

extern crate alloc;
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

/// Derive macro to implement `ValidateCurrencies` trait for structs.
/// This macro checks for fields of type `Amount`, `XRPAmount`, `IssuedCurrencyAmount`, `Currency`, `XRP`, or `IssuedCurrency`.
/// It generates a `validate_currencies` method that validates these values.
#[proc_macro_derive(ValidateCurrencies)]
pub fn derive_validate_currencies(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => {
                return syn::Error::new_spanned(name, "Only named fields supported")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "Only structs are supported")
                .to_compile_error()
                .into();
        }
    };

    let amount_field_validations = fields.iter().filter_map(|field| {
        let ident = &field.ident;
        match &field.ty {
            // Handle Option<T> where T is one of the valid types
            Type::Path(type_path) => {
                use alloc::string::ToString;
                let segments = &type_path.path.segments;
                if segments.len() == 1 && segments[0].ident == "Option" {
                    // Extract T from Option<T>
                    if let syn::PathArguments::AngleBracketed(angle_bracketed) =
                        &segments[0].arguments
                    {
                        if let Some(syn::GenericArgument::Type(Type::Path(inner_type_path))) =
                            angle_bracketed.args.first()
                        {
                            let inner_ident = &inner_type_path.path.segments.last().unwrap().ident;
                            if [
                                "Amount",
                                "XRPAmount",
                                "IssuedCurrencyAmount",
                                "Currency",
                                "XRP",
                                "IssuedCurrency",
                            ]
                            .contains(&inner_ident.to_string().as_str())
                            {
                                return Some(quote! {
                                    if let Some(x) = &self.#ident {
                                        x.validate()?;
                                    }
                                });
                            }
                        }
                    }
                }

                // Handle direct fields: Amount, XRPAmount, etc.
                let type_ident = &segments.last().unwrap().ident;
                if [
                    "Amount",
                    "XRPAmount",
                    "IssuedCurrencyAmount",
                    "Currency",
                    "XRP",
                    "IssuedCurrency",
                ]
                .contains(&type_ident.to_string().as_str())
                {
                    return Some(quote! {
                        self.#ident.validate()?;
                    });
                }

                None
            }
            _ => None,
        }
    });

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ValidateCurrencies for #name #ty_generics #where_clause {
            fn validate_currencies(&self) -> crate::models::XRPLModelResult<()> {
                #(#amount_field_validations)*

                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
