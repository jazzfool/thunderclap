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
                let func = quote::format_ident!("unwrap_as_{}", key);

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
                            pub fn #func(self) -> Option<#ty> {
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
            let func = quote::format_ident!("unwrap_as_{}", key);

            {
                quote! {
                    impl #impl_generics #crate_name::pipe::Event for #name #ty_generics #where_clause {
                        fn get_key(&self) -> &'static str {
                            std::stringify!(#key)
                        }
                    }

                    impl #impl_generics #name #ty_generics #where_clause {
                        pub fn #func(self) -> Option<Self> {
                            Some(self)
                        }
                    }
                }
            }.into()
        }
        _ => panic!("derive(PipelineEvent) only supports structs and enums."),
    }
}

fn find_crate_name(attrs: &[syn::Attribute]) -> Option<syn::Ident> {
    for attr in attrs {
        if attr.path.segments.first().map(|i| i.ident == "reui_crate").unwrap_or(false) {
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
        if attr.path.segments.first().map(|i| i.ident == "event_key").unwrap_or(false) {
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

enum IdentOrIndex {
    Ident(syn::Ident),
    Index(syn::Index),
}

#[proc_macro_derive(LayableWidget, attributes(widget_layout, reui_crate))]
pub fn layable_widget_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    impl_layable_widget_macro(ast)
}

fn impl_layable_widget_macro(ast: syn::DeriveInput) -> TokenStream {
    match &ast.data {
        syn::Data::Struct(ref data) => {
            let crate_name = find_crate_name(&ast.attrs)
                .unwrap_or(syn::Ident::new("reui", proc_macro2::Span::call_site()));
            let mut layout_ident = None;
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let name = &ast.ident;

            match &data.fields {
                syn::Fields::Named(fields) => {
                    for field in fields.named.iter() {
                        if let Some(ref ident) = field.ident {
                            if chk_attrs_is_layout(&field.attrs) {
                                layout_ident = IdentOrIndex::Ident(ident.clone()).into();
                                break;
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        if chk_attrs_is_layout(&field.attrs) {
                            layout_ident = IdentOrIndex::Index(i.into()).into();
                            break;
                        }
                    }
                }
                syn::Fields::Unit => panic!("Unit structs aren't capable of having a layout field"),
            }

            if let Some(layout_ident) = layout_ident {
                let ident = match layout_ident {
                    IdentOrIndex::Ident(ident) => quote! { self.#ident },
                    IdentOrIndex::Index(index) => quote! { self.#index },
                };

                {
                    quote!{
                        impl #impl_generics #crate_name::base::LayableWidget for #name #ty_generics #where_clause {
                            #[inline]
                            fn listen_to_layout(&mut self, layout: impl Into<Option<#crate_name::base::WidgetLayoutEventsInner>>) {
                                #ident.update(layout);
                            }

                            #[inline]
                            fn layout_id(&self) -> Option<u64> {
                                #ident.id()
                            }
                        }
                    }
                }.into()
            } else {
                panic!("Could not find [widget_layout] attribute on any field")
            }
        }
        _ => panic!("derive(LayableWidget) only supports structs."),
    }
}

fn chk_attrs_is_layout(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path.segments.first().map(|i| i.ident == "widget_layout").unwrap_or(false) {
            return true;
        }
    }
    false
}

#[proc_macro_derive(DropNotifier, attributes(widget_drop_event, reui_crate))]
pub fn drop_notifier_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    impl_drop_notifier_macro(ast)
}

fn impl_drop_notifier_macro(ast: syn::DeriveInput) -> TokenStream {
    match &ast.data {
        syn::Data::Struct(ref data) => {
            let crate_name = find_crate_name(&ast.attrs)
                .unwrap_or(syn::Ident::new("reui", proc_macro2::Span::call_site()));
            let mut drop_event_ident = None;
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let name = &ast.ident;

            match &data.fields {
                syn::Fields::Named(fields) => {
                    for field in fields.named.iter() {
                        if let Some(ref ident) = field.ident {
                            if chk_attrs_is_drop_event(&field.attrs) {
                                drop_event_ident = IdentOrIndex::Ident(ident.clone()).into();
                                break;
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        if chk_attrs_is_drop_event(&field.attrs) {
                            drop_event_ident = IdentOrIndex::Index(i.into()).into();
                            break;
                        }
                    }
                }
                syn::Fields::Unit => {
                    panic!("Unit structs aren't capable of having a drop event field")
                }
            }

            if let Some(drop_event_ident) = drop_event_ident {
                let ident = match drop_event_ident {
                    IdentOrIndex::Ident(ident) => quote! { self.#ident },
                    IdentOrIndex::Index(index) => quote! { self.#index },
                };

                {
                    quote!{
                        impl #impl_generics #crate_name::base::DropNotifier for #name #ty_generics #where_clause {
                            #[inline(always)]
                            fn drop_event(&self) -> &#crate_name::reclutch::event::RcEventQueue<#crate_name::base::DropEvent> {
                                &#ident
                            }
                        }
                    }
                }.into()
            } else {
                panic!("Could not find [widget_drop_event] attribute on any field")
            }
        }
        _ => panic!("derive(DropNotifier) only supports structs."),
    }
}

fn chk_attrs_is_drop_event(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path.segments.first().map(|i| i.ident == "widget_drop_event").unwrap_or(false) {
            return true;
        }
    }
    false
}

#[proc_macro_derive(HasVisibility, attributes(widget_visibility, reui_crate))]
pub fn has_visibility_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    impl_has_visibility_macro(ast)
}

fn impl_has_visibility_macro(ast: syn::DeriveInput) -> TokenStream {
    match &ast.data {
        syn::Data::Struct(ref data) => {
            let crate_name = find_crate_name(&ast.attrs)
                .unwrap_or(syn::Ident::new("reui", proc_macro2::Span::call_site()));
            let mut vis_ident = None;
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let name = &ast.ident;

            match &data.fields {
                syn::Fields::Named(fields) => {
                    for field in fields.named.iter() {
                        if let Some(ref ident) = field.ident {
                            if chk_attrs_is_visibility(&field.attrs) {
                                vis_ident = IdentOrIndex::Ident(ident.clone()).into();
                                break;
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        if chk_attrs_is_visibility(&field.attrs) {
                            vis_ident = IdentOrIndex::Index(i.into()).into();
                            break;
                        }
                    }
                }
                syn::Fields::Unit => {
                    panic!("Unit structs aren't capable of having a visibility field")
                }
            }

            if let Some(vis_ident) = vis_ident {
                let ident = match vis_ident {
                    IdentOrIndex::Ident(ident) => quote! { self.#ident },
                    IdentOrIndex::Index(index) => quote! { self.#index },
                };

                {
                    quote!{
                        impl #impl_generics #crate_name::base::HasVisibility for #name #ty_generics #where_clause {
                            #[inline]
                            fn set_visibility(&mut self, visibility: #crate_name::base::Visibility) {
                                #ident = visibility;
                            }

                            #[inline]
                            fn visibility(&self) -> #crate_name::base::Visibility {
                                #ident
                            }
                        }
                    }
                }.into()
            } else {
                panic!("Could not find [widget_visibility] attribute on any field")
            }
        }
        _ => panic!("derive(HasVisibility) only supports structs."),
    }
}

fn chk_attrs_is_visibility(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path.segments.first().map(|i| i.ident == "widget_visibility").unwrap_or(false) {
            return true;
        }
    }
    false
}

#[proc_macro_derive(Repaintable, attributes(repaint_target, reui_crate))]
pub fn repaintable_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    impl_repaintable_macro(ast)
}

fn impl_repaintable_macro(ast: syn::DeriveInput) -> TokenStream {
    match &ast.data {
        syn::Data::Struct(ref data) => {
            let crate_name = find_crate_name(&ast.attrs)
                .unwrap_or(syn::Ident::new("reui", proc_macro2::Span::call_site()));
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let name = &ast.ident;

            let mut repaint_targets = Vec::new();

            match &data.fields {
                syn::Fields::Named(fields) => {
                    for field in fields.named.iter() {
                        if let Some(ref ident) = field.ident {
                            if chk_attrs_is_repaint_target(&field.attrs) {
                                repaint_targets.push(quote! {
                                    self.#ident.repaint();
                                });
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        if chk_attrs_is_repaint_target(&field.attrs) {
                            let index: syn::Index = i.into();

                            repaint_targets.push(quote! {
                                self.#index.repaint();
                            })
                        }
                    }
                }
                _ => {}
            }

            {
                quote!{
                    impl #impl_generics #crate_name::base::Repaintable for #name #ty_generics #where_clause {
                        #[inline]
                        fn repaint(&mut self) {
                            #(#repaint_targets)*

                            for child in #crate_name::base::WidgetChildren::children_mut(self) {
                                child.repaint();
                            }
                        }
                    }
                }
            }.into()
        }
        _ => panic!("derive(Repaintable) only supports structs."),
    }
}

fn chk_attrs_is_repaint_target(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path.segments.first().map(|i| i.ident == "repaint_target").unwrap_or(false) {
            return true;
        }
    }
    false
}

#[proc_macro_derive(
    Movable,
    attributes(widget_position, widget_rect, widget_transform_callback, reui_crate)
)]
pub fn movable_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    impl_movable_macro(ast)
}

fn impl_movable_macro(ast: syn::DeriveInput) -> TokenStream {
    match &ast.data {
        syn::Data::Struct(ref data) => {
            let crate_name = find_crate_name(&ast.attrs)
                .unwrap_or(syn::Ident::new("reui", proc_macro2::Span::call_site()));
            let mut assignment = None;
            let mut return_val = None;
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let name = &ast.ident;
            let callback = find_widget_transform_callback(&ast.attrs)
                .map(|ident| quote! { self.#ident(); })
                .unwrap_or_else(|| quote! {});

            match &data.fields {
                syn::Fields::Named(fields) => {
                    for field in fields.named.iter() {
                        if let Some(ref ident) = field.ident {
                            if chk_attrs_is_rect(&field.attrs) {
                                assignment = Some(quote! {
                                    self.#ident.origin = position;
                                });
                                return_val = Some(quote! {
                                    self.#ident.origin
                                });
                                break;
                            } else if chk_attrs_is_position(&field.attrs) {
                                assignment = Some(quote! {
                                    self.#ident = position;
                                });
                                return_val = Some(quote! {
                                    self.#ident
                                });
                                break;
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        let index: syn::Index = i.into();
                        if chk_attrs_is_rect(&field.attrs) {
                            assignment = Some(quote! {
                                self.#index.origin = position;
                            });
                            return_val = Some(quote! {
                                self.#index.origin
                            });
                            break;
                        } else if chk_attrs_is_position(&field.attrs) {
                            assignment = Some(quote! {
                                self.#index = position;
                            });
                            return_val = Some(quote! {
                                self.#index
                            });
                            break;
                        }
                    }
                }
                syn::Fields::Unit => {
                    panic!("Unit structs aren't capable of having a position/rectangle field")
                }
            }

            if let Some(assignment) = assignment {
                {
                    quote!{
                        impl #impl_generics #crate_name::base::Movable for #name #ty_generics #where_clause {
                            fn set_position(&mut self, position: #crate_name::reclutch::display::Point) {
                                #assignment
                                #crate_name::base::Repaintable::repaint(self);
                                #callback
                            }

                            #[inline]
                            fn position(&self) -> #crate_name::reclutch::display::Point {
                                #return_val
                            }
                        }
                    }
                }.into()
            } else {
                panic!("Could not find [widget_position] or [widget_rect] attribute on any field")
            }
        }
        _ => panic!("derive(Movable) only supports structs."),
    }
}

fn chk_attrs_is_position(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path.segments.first().map(|i| i.ident == "widget_position").unwrap_or(false) {
            return true;
        }
    }
    false
}

fn chk_attrs_is_rect(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path.segments.first().map(|i| i.ident == "widget_rect").unwrap_or(false) {
            return true;
        }
    }
    false
}

fn find_widget_transform_callback(attrs: &[syn::Attribute]) -> Option<syn::Ident> {
    for attr in attrs {
        if attr
            .path
            .segments
            .first()
            .map(|i| i.ident == "widget_transform_callback")
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

#[proc_macro_derive(
    Resizable,
    attributes(widget_size, widget_rect, widget_transform_callback, reui_crate)
)]
pub fn resizable_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    impl_resizable_macro(ast)
}

fn impl_resizable_macro(ast: syn::DeriveInput) -> TokenStream {
    match &ast.data {
        syn::Data::Struct(ref data) => {
            let crate_name = find_crate_name(&ast.attrs)
                .unwrap_or(syn::Ident::new("reui", proc_macro2::Span::call_site()));
            let mut assignment = None;
            let mut return_val = None;
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let name = &ast.ident;
            let callback = find_widget_transform_callback(&ast.attrs)
                .map(|ident| quote! { self.#ident(); })
                .unwrap_or_else(|| quote! {});

            match &data.fields {
                syn::Fields::Named(fields) => {
                    for field in fields.named.iter() {
                        if let Some(ref ident) = field.ident {
                            if chk_attrs_is_rect(&field.attrs) {
                                assignment = Some(quote! {
                                    self.#ident.size = size;
                                });
                                return_val = Some(quote! {
                                    self.#ident.size
                                });
                                break;
                            } else if chk_attrs_is_size(&field.attrs) {
                                assignment = Some(quote! {
                                    self.#ident = size;
                                });
                                return_val = Some(quote! {
                                    self.#ident
                                });
                                break;
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        let index: syn::Index = i.into();
                        if chk_attrs_is_rect(&field.attrs) {
                            assignment = Some(quote! {
                                self.#index.size = size;
                            });
                            return_val = Some(quote! {
                                self.#index.size
                            });
                            break;
                        } else if chk_attrs_is_size(&field.attrs) {
                            assignment = Some(quote! {
                                self.#index = size;
                            });
                            return_val = Some(quote! {
                                self.#index
                            });
                            break;
                        }
                    }
                }
                syn::Fields::Unit => {
                    panic!("Unit structs aren't capable of having a position/rectangle field")
                }
            }

            if let Some(assignment) = assignment {
                {
                    quote!{
                        impl #impl_generics #crate_name::base::Resizable for #name #ty_generics #where_clause {
                            fn set_size(&mut self, size: #crate_name::reclutch::display::Size) {
                                #assignment
                                #crate_name::base::Repaintable::repaint(self);
                                #callback
                            }

                            #[inline]
                            fn size(&self) -> #crate_name::reclutch::display::Size {
                                #return_val
                            }
                        }
                    }
                }.into()
            } else {
                panic!("Could not find [widget_position] or [widget_rect] attribute on any field")
            }
        }
        _ => panic!("derive(Movable) only supports structs."),
    }
}

fn chk_attrs_is_size(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path.segments.first().map(|i| i.ident == "widget_size").unwrap_or(false) {
            return true;
        }
    }
    false
}
