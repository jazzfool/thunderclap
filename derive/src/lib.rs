extern crate proc_macro;

use {proc_macro::TokenStream, quote::quote};

#[proc_macro_derive(PipelineEvent, attributes(event_key, reui_crate))]
pub fn pipeline_event_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    impl_pipeline_event_macro(ast)
}

fn impl_pipeline_event_macro(ast: syn::DeriveInput) -> TokenStream {
    match ast.data {
        syn::Data::Enum(enum_data) => {
            let crate_name = find_crate_name(&ast.attrs)
            .unwrap_or(syn::Ident::new("reui", proc_macro2::Span::call_site()));
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let name = ast.ident;
            
            let mut key_pats: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut cast_fns: Vec<proc_macro2::TokenStream> = Vec::new();
            
            for variant in enum_data.variants {
                let key = find_event_key(&variant.attrs);
                let um: proc_macro2::TokenStream = get_unmatched_variant(&variant).into();
                
                let (match_ext, ty, ret) = get_variant_matched_tuples(&variant);
                let (match_ext, ty, ret): (
                    proc_macro2::TokenStream,
                    proc_macro2::TokenStream,
                    proc_macro2::TokenStream,
                ) = (match_ext.into(), ty.into(), ret.into());
                
                key_pats.push(
                    {
                        quote! { #name::#um => std::stringify!(#key) }
                    }
                    .into(),
                );
                
                cast_fns.push(
                    {
                        quote! {
                            pub fn #key(self) -> Option<#ty> {
                                if let #name::#match_ext = self {
                                    Some(#ret)
                                } else {
                                    None
                                }
                            }
                        }
                    }
                    .into(),
                );
            }
            
            {
                quote! {
                    impl #impl_generics #crate_name::pipe::Event for #name #ty_generics #where_clause {
                        fn get_key(&self) -> &'static str {
                            match self {
                                #(#key_pats),*
                            }
                        }
                    }
                    
                    impl #impl_generics #name #ty_generics #where_clause {
                        #(#cast_fns)*
                    }
                }
            }
            .into()
        }
        syn::Data::Struct(_) => {
            let crate_name = find_crate_name(&ast.attrs)
            .unwrap_or(syn::Ident::new("reui", proc_macro2::Span::call_site()));
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let name = ast.ident;
            let key = find_event_key(&ast.attrs);

            {
                quote! {
                    impl #impl_generics #crate_name::pipe::Event for #name #ty_generics #where_clause {
                        fn get_key(&self) -> &'static str {
                            std::stringify!(#key)
                        }
                    }

                    impl #impl_generics #name #ty_generics #where_clause {
                        pub fn #key(self) -> Option<Self> {
                            Some(self)
                        }
                    }
                }
            }.into()
        }
        _ => panic!("derive(PipelineEvent) only supports structs and enums.")
    }
}

fn find_crate_name(attrs: &[syn::Attribute]) -> Option<syn::Ident> {
    for attr in attrs {
        if attr
            .path
            .segments
            .first()
            .map(|i| i.ident == "reui_crate")
            .unwrap_or(false)
        {
            if let proc_macro2::TokenTree::Group(grp) =
                attr.clone().tokens.into_iter().nth(0).unwrap()
            {
                if let proc_macro2::TokenTree::Ident(ident) =
                    grp.stream().into_iter().nth(0).unwrap()
                {
                    return Some(ident);
                }
            }
        }
    }

    None
}

fn get_variant_matched_tuples(variant: &syn::Variant) -> (TokenStream, TokenStream, TokenStream) {
    let name = &variant.ident;
    match &variant.fields {
        syn::Fields::Unit => (
            {
                quote! { #name }
            }
            .into(),
            {
                quote! { () }
            }
            .into(),
            {
                quote! { () }
            }
            .into(),
        ),
        syn::Fields::Unnamed(fields) => {
            let mut matching: Vec<syn::Ident> = Vec::new();
            let mut types: Vec<syn::Type> = Vec::new();
            let mut idx = 0;

            for field in &fields.unnamed {
                matching.push(quote::format_ident!("x{}", idx.to_string()));
                types.push(field.ty.clone());
                idx += 1;
            }

            (
                {
                    quote! {
                        #name(#(#matching),*)
                    }
                }
                .into(),
                {
                    quote! {
                        (#(#types),*)
                    }
                }
                .into(),
                {
                    quote! {
                        (#(#matching),*)
                    }
                }
                .into(),
            )
        }
        syn::Fields::Named(fields) => {
            let mut matching: Vec<syn::Ident> = Vec::new();
            let mut types: Vec<syn::Type> = Vec::new();
            for field in &fields.named {
                matching.push(field.ident.clone().unwrap());
                types.push(field.ty.clone());
            }

            (
                {
                    quote! {
                        #name{#(#matching),*}
                    }
                }
                .into(),
                {
                    quote! {
                        (#(#types),*)
                    }
                }
                .into(),
                {
                    quote! {
                        (#(#matching),*)
                    }
                }
                .into(),
            )
        }
    }
}

fn get_unmatched_variant(variant: &syn::Variant) -> TokenStream {
    match variant.fields {
        syn::Fields::Unit => {
            let ident = variant.ident.clone();

            {
                quote! {
                    #ident
                }
            }
            .into()
        }
        syn::Fields::Unnamed(_) => {
            let ident = variant.ident.clone();

            {
                quote! {
                    #ident(..)
                }
            }
            .into()
        }
        syn::Fields::Named(_) => {
            let ident = variant.ident.clone();

            {
                quote! {
                    #ident{..}
                }
            }
            .into()
        }
    }
}

fn find_event_key(attrs: &[syn::Attribute]) -> syn::Ident {
    for attr in attrs {
        if attr
            .path
            .segments
            .first()
            .map(|i| i.ident == "event_key")
            .unwrap_or(false)
        {
            if let proc_macro2::TokenTree::Group(grp) =
                attr.clone().tokens.into_iter().nth(0).unwrap()
            {
                if let proc_macro2::TokenTree::Ident(ident) =
                    grp.stream().into_iter().nth(0).unwrap()
                {
                    return ident;
                }
            }
        }
    }
    panic!("Variant missing an event_key")
}
