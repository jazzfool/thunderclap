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
                            fn set_position(&mut self, position: #crate_name::geom::RelativePoint) {
                                #assignment
                                #crate_name::base::Repaintable::repaint(self);
                                #callback
                            }

                            #[inline]
                            fn position(&self) -> #crate_name::geom::RelativePoint {
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
                                    self.#ident.size = size.cast_unit();
                                });
                                return_val = Some(quote! {
                                    self.#ident.size.cast_unit()
                                });
                                break;
                            } else if chk_attrs_is_size(&field.attrs) {
                                assignment = Some(quote! {
                                    self.#ident = size.cast_unit();
                                });
                                return_val = Some(quote! {
                                    self.#ident.cast_unit()
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
                                self.#index.size = size.cast_unit();
                            });
                            return_val = Some(quote! {
                                self.#index.size.cast_unit()
                            });
                            break;
                        } else if chk_attrs_is_size(&field.attrs) {
                            assignment = Some(quote! {
                                self.#index = size.cast_unit();
                            });
                            return_val = Some(quote! {
                                self.#index.cast_unit()
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

#[derive(Debug)]
struct DataField {
    name: syn::Ident,
    default: syn::Expr,
    field_type: syn::Type,
}

#[derive(Debug)]
struct DataFieldList {
    list: Vec<DataField>,
}

#[derive(Debug)]
struct RooftopData {
    struct_name: syn::Ident,
    output_event: syn::Type,
    data_fields: DataFieldList,
    widget_tree_root: WidgetNode,
    bindings: Vec<proc_macro2::TokenStream>,
    terminals: Vec<proc_macro2::TokenStream>,
    bind_propagation: Vec<proc_macro2::TokenStream>,
    functions: Vec<(syn::Ident, syn::Block)>,
}

impl syn::parse::Parse for DataField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![:]>()?;
        let field_type = input.parse::<syn::Type>()?;
        input.parse::<syn::Token![=]>()?;
        let default = input.parse::<syn::Expr>()?;

        Ok(DataField { name, default, field_type })
    }
}

impl syn::parse::Parse for DataFieldList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(DataFieldList { list: Vec::new() })
        } else {
            let fields: syn::punctuated::Punctuated<_, syn::Token![,]> =
                input.parse_terminated(DataField::parse)?;
            let data_fields = fields.into_iter().collect();
            Ok(DataFieldList { list: data_fields })
        }
    }
}

enum FunctionBody<'a> {
    View(syn::parse::ParseBuffer<'a>),
    Other(syn::Block),
}

fn parse_function(
    stream: syn::parse::ParseStream,
) -> syn::Result<(syn::Ident, Option<DataFieldList>, FunctionBody, bool)> {
    stream.parse::<syn::Token![fn]>()?;

    let fn_name = stream.parse::<syn::Ident>()?;
    let parameters;
    syn::parenthesized!(parameters in stream);

    let dfl =
        if parameters.is_empty() { None } else { parameters.parse::<DataFieldList>()?.into() };

    let fn_body = if fn_name.to_string() == "build" {
        let body;
        syn::braced!(body in stream);
        FunctionBody::View(body)
    } else {
        let body = stream.parse::<syn::Block>()?;
        FunctionBody::Other(body)
    };

    let next_fn = stream.peek(syn::Token![fn]);

    Ok((fn_name, dfl, fn_body, next_fn))
}

#[derive(Debug, Clone)]
struct WidgetNode {
    type_name: syn::Ident,
    var_name: syn::Ident,
    data_assignments: Vec<DataAssignment>,
    children: Vec<WidgetNode>,
}

#[derive(Debug, Clone)]
struct DataAssignment {
    var: syn::Ident,
    value: syn::Expr,
    binding: bool,
}

mod bind_syntax {
    syn::custom_keyword!(bind);
}

impl DataAssignment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let var = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![=]>()?;
        let binding = input.peek(bind_syntax::bind);
        let value = if binding {
            input.parse::<bind_syntax::bind>()?;
            let value;
            syn::parenthesized!(value in input);
            value.parse::<syn::Expr>()?
        } else {
            input.parse::<syn::Expr>()?
        };
        Ok(DataAssignment { var, value, binding })
    }
}

fn parse_view(
    stream: syn::parse::ParseStream,
    bindings: &mut Vec<proc_macro2::TokenStream>,
    terminals: &mut Vec<proc_macro2::TokenStream>,
    bind_propagation: &mut Vec<proc_macro2::TokenStream>,
    count: &mut u64,
) -> syn::Result<(WidgetNode, bool)> {
    let type_name = stream.parse::<syn::Ident>()?;
    let assignments;
    syn::parenthesized!(assignments in stream);
    let data_assignments: syn::punctuated::Punctuated<_, syn::Token![,]> =
        assignments.parse_terminated(DataAssignment::parse)?;
    let mut data_assignments: Vec<_> = data_assignments.into_iter().collect();
    let var_name = if stream.parse::<syn::Token![as]>().is_ok() {
        stream.parse::<syn::Ident>()?
    } else {
        *count += 1;
        quote::format_ident!("unnamed_widget_{}", count)
    };

    for assignment in &data_assignments {
        if assignment.binding {
            let value = assignment.value.clone();
            let var = assignment.var.clone();
            bindings.push(quote! {
                {
                    widget.#var_name.default_data().#var = #value;
                }
            });
            bind_propagation.push(quote! {
                self.#var_name.perform_bind(aux);
            });
        }
    }

    data_assignments.retain(|assignment| !assignment.binding);

    let mut parse_terminals = true;
    let mut events = Vec::new();
    while parse_terminals {
        if stream.parse::<syn::token::At>().is_ok() {
            let event_name = stream.parse::<syn::Ident>()?;
            let handler_body = stream.parse::<syn::Block>()?;
            events.push(quote! {
                #event_name {
                    { #handler_body }
                }
            });
        } else {
            parse_terminals = false;
        }
    }

    if !events.is_empty() {
        terminals.push(
            {
                quote! {
                    event in #var_name.default_event_queue() => {
                        #(#events)*
                    }
                }
            }
            .into(),
        );
    }

    let mut children = Vec::new();
    if stream.peek(syn::token::Brace) {
        let children_parse;
        syn::braced!(children_parse in stream);

        let mut parse_child = true;
        while parse_child {
            if children_parse.is_empty() {
                parse_child = false;
            } else {
                let (node, found_comma) =
                    parse_view(&children_parse, bindings, terminals, bind_propagation, count)?;
                children.push(node);
                parse_child = found_comma;
            }
        }
    }

    let found_comma = stream.parse::<syn::Token![,]>().is_ok();

    Ok((WidgetNode { type_name, var_name, data_assignments, children }, found_comma))
}

impl syn::parse::Parse for RooftopData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![struct]>()?;
        let struct_name = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let output_event = input.parse()?;
        let struct_content;
        syn::braced!(struct_content in input);

        let mut parse_fn = struct_content.peek(syn::Token![fn]);
        let mut view_body = None;
        let mut data_fields = None;
        let mut other_functions = Vec::new();
        while parse_fn {
            let (fn_name, param_fields, body, next_fn) = parse_function(&struct_content)?;
            parse_fn = next_fn;

            match body {
                FunctionBody::View(body) => {
                    view_body = Some(body);
                    data_fields = param_fields.unwrap().into();
                }
                FunctionBody::Other(body) => {
                    other_functions.push((fn_name, body));
                }
            }
        }

        let view_body = view_body.expect("no build() pseudo-function found");

        let mut bindings = Vec::new();
        let mut terminals = Vec::new();
        let mut bind_propagation = Vec::new();
        let mut count = 0;
        let widget_tree_root = parse_view(
            &view_body,
            &mut bindings,
            &mut terminals,
            &mut bind_propagation,
            &mut count,
        )?
        .0;

        Ok(RooftopData {
            struct_name,
            output_event,
            data_fields: data_fields
                .expect("failed to find data fields (parameters of build() pseudo-function)"),
            widget_tree_root,
            bindings,
            terminals,
            bind_propagation,
            functions: other_functions,
        })
    }
}

fn flatten_widget_node_tree(root: &WidgetNode, output: &mut Vec<WidgetNode>) {
    output.push(root.clone());
    for child in &root.children {
        flatten_widget_node_tree(child, output);
    }
}

fn find_pseudo_function(
    name: &'static str,
    functions: &[(syn::Ident, syn::Block)],
) -> Option<proc_macro2::TokenStream> {
    use quote::ToTokens;
    let block = functions.iter().find(|func| func.0.to_string() == name)?.1.clone();
    let mut tokens = proc_macro2::TokenStream::new();
    block.to_tokens(&mut tokens);
    tokens.into()
}

impl WidgetNode {
    fn compile_layout(&self) -> proc_macro2::TokenStream {
        let name = &self.var_name;
        if self.children.is_empty() {
            quote! {
                &mut #name
            }
        } else {
            let children: Vec<_> = self
                .children
                .iter()
                .map(|child| {
                    let layout = child.compile_layout();
                    quote! {
                        None => #layout,
                    }
                })
                .collect();
            quote! {
                define_layout! {
                    for #name => {
                        #(#children)*
                    }
                }
            }
        }
    }
}

impl RooftopData {
    fn compile(self) -> TokenStream {
        let struct_name = self.struct_name;
        let output_event = self.output_event;

        let data_fields: Vec<proc_macro2::TokenStream> = self
            .data_fields
            .list
            .iter()
            .map(|data_field| {
                let name = &data_field.name;
                let field_type = &data_field.field_type;
                quote! {
                    pub #name: #field_type,
                }
            })
            .collect();

        let data_field_init: Vec<proc_macro2::TokenStream> = self
            .data_fields
            .list
            .iter()
            .map(|data_field| {
                let name = &data_field.name;
                let default = &data_field.default;
                quote! {
                    #name: #default,
                }
            })
            .collect();

        let widget_name = quote::format_ident!("{}Widget", struct_name);

        let reui = quote::format_ident!("reui");

        let mut flattened_nodes = Vec::new();
        flatten_widget_node_tree(&self.widget_tree_root, &mut flattened_nodes);

        let widget_declarations: Vec<proc_macro2::TokenStream> = flattened_nodes
            .iter()
            .map(|node| {
                let name = &node.var_name;
                let type_name = &node.type_name;
                let assignments: Vec<proc_macro2::TokenStream> = node
                    .data_assignments
                    .iter()
                    .map(|assignment| {
                        let var = &assignment.var;
                        let value = &assignment.value;
                        quote! {
                            #var: #value,
                        }
                    })
                    .collect();
                quote! {
                    let mut #name = #type_name {
                        #(#assignments)*
                        ..#type_name::from_theme(theme)
                    }.construct(theme, u_aux, g_aux);
                }
            })
            .collect();

        let widget_names: Vec<proc_macro2::TokenStream> = flattened_nodes
            .iter()
            .map(|node| {
                let name = &node.var_name;
                quote! {
                    #name,
                }
            })
            .collect();

        let widgets_as_fields: Vec<proc_macro2::TokenStream> = flattened_nodes
            .iter()
            .rev()
            .map(|node| {
                let name = &node.var_name;
                let type_name = &node.type_name;
                quote! {
                    #[widget_child]
                    #[repaint_target]
                    #name: <#type_name as #reui::ui::WidgetDataTarget<U, G>>::Target,
                }
            })
            .collect();

        let bindings = &self.bindings;
        let terminals = &self.terminals;
        let bind_propagation = &self.bind_propagation;

        let build_pipeline =
            find_pseudo_function("build_pipeline", &self.functions).unwrap_or(quote! { { pipe } });
        let before_pipeline = find_pseudo_function("before_pipeline", &self.functions)
            .unwrap_or(proc_macro2::TokenStream::new());
        let after_pipeline = find_pseudo_function("after_pipeline", &self.functions)
            .unwrap_or(proc_macro2::TokenStream::new());
        let draw = find_pseudo_function("draw", &self.functions).unwrap_or(quote! { &[] });

        let define_layout = self.widget_tree_root.compile_layout();

        let root_name = &self.widget_tree_root.var_name;
        {
            quote! {
                pub struct #struct_name {
                    #(#data_fields)*
                }

                impl #struct_name {
                    pub fn from_theme(theme: &dyn #reui::draw::Theme) -> Self {
                        #struct_name {
                            #(#data_field_init)*
                        }
                    }

                    pub fn construct<U, G>(self, theme: &dyn #reui::draw::Theme, u_aux: &mut U, g_aux: &mut G) -> #widget_name<U, G>
                    where
                        U: #reui::base::UpdateAuxiliary,
                        G: #reui::base::GraphicalAuxiliary,
                    {
                        let mut data = #reui::base::Observed::new(self);
                        #(#widget_declarations)*
                        #define_layout;

                        use #reui::ui::DefaultEventQueue;
                        let mut pipe = pipeline! {
                            #widget_name<U, G> as widget,
                            U as aux,
                            #(#terminals)*
                        };

                        pipe = #build_pipeline;

                        let mut bind_pipe = pipeline! {
                            #widget_name<U, G> as widget,
                            U as aux,
                            event in &data.on_change => {
                                change {
                                    use #reui::ui::DefaultWidgetData;
                                    let bind = &mut widget.data;
                                    #(#bindings)*
                                }
                            }
                        };

                        // emits false positive event to apply bindings
                        data.get_mut();

                        let mut output_widget = #widget_name {
                            event_queue: Default::default(),
                            data,
                            pipe: pipe.into(),
                            bind_pipe: bind_pipe.into(),
                            parent_position: Default::default(),

                            visibility: Default::default(),
                            command_group: Default::default(),
                            layout: Default::default(),
                            drop_event: Default::default(),

                            phantom_themed: Default::default(),
                            phantom_g: Default::default(),

                            #(#widget_names)*
                        };

                        {
                            use #reui::reclutch::widget::Widget;
                            output_widget.update(u_aux);
                        }

                        output_widget

                        // Phew, all that parsing just to generate this
                    }
                }

                impl<U, G> #reui::ui::WidgetDataTarget<U, G> for #struct_name
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    type Target = #widget_name<U, G>;
                }

                #[derive(
                    WidgetChildren,
                    LayableWidget,
                    DropNotifier,
                    HasVisibility,
                    Repaintable,
                )]
                #[widget_children_trait(base::WidgetChildren)]
                #[reui_crate(#reui)]
                pub struct #widget_name<U, G>
                where
                    U: base::UpdateAuxiliary,
                    G: base::GraphicalAuxiliary,
                {
                    pub event_queue: #reui::reclutch::event::RcEventQueue<#output_event>,
                    pub data: #reui::base::Observed<#struct_name>,
                    pipe: Option<#reui::pipe::Pipeline<Self, U>>,
                    bind_pipe: Option<#reui::pipe::Pipeline<Self, U>>,
                    parent_position: #reui::geom::AbsolutePoint,

                    #[widget_visibility]
                    visibility: #reui::base::Visibility,
                    #[repaint_target]
                    command_group: #reui::reclutch::display::CommandGroup,
                    #[widget_layout]
                    layout: #reui::base::WidgetLayoutEvents,
                    #[widget_drop_event]
                    drop_event: #reui::reclutch::event::RcEventQueue<#reui::base::DropEvent>,

                    #(#widgets_as_fields)*

                    phantom_themed: #reui::draw::PhantomThemed,
                    phantom_g: std::marker::PhantomData<G>,
                }

                impl<U, G> #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    fn on_transform(&mut self) {
                        use #reui::{base::{Repaintable}, geom::ContextuallyRectangular};
                        self.repaint();
                        self.layout.notify(self.#root_name.abs_rect());
                    }
                }

                impl<U, G> #reui::reclutch::widget::Widget for #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    type UpdateAux = U;
                    type GraphicalAux = G;
                    type DisplayObject = #reui::reclutch::display::DisplayCommand;

                    #[inline]
                    fn bounds(&self) -> #reui::reclutch::display::Rect {
                        self.#root_name.bounds().cast_unit()
                    }

                    fn update(&mut self, aux: &mut U) {
                        #reui::base::invoke_update(self, aux);

                        #before_pipeline
                        let mut pipe = self.pipe.take().unwrap();
                        pipe.update(self, aux);
                        self.pipe = Some(pipe);
                        #after_pipeline
                        if let Some(rect) = self.layout.receive() {
                            use #reui::geom::ContextuallyRectangular;
                            self.#root_name.set_ctxt_rect(rect);
                            self.command_group.repaint();
                        }

                        {
                            use #reui::ui::Bindable;
                            self.perform_bind(aux);
                        }
                    }

                    fn draw(&mut self, display: &mut dyn #reui::reclutch::display::GraphicsDisplay, aux: &mut G) {
                        self.command_group.push(display, { #draw }, None, None);
                    }
                }

                impl<U, G> #reui::ui::Bindable<U> for #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn perform_bind(&mut self, aux: &mut U) {
                        let mut bind_pipe = self.bind_pipe.take().unwrap();
                        bind_pipe.update(self, aux);
                        self.bind_pipe = Some(bind_pipe);

                        #(#bind_propagation)*
                    }
                }

                impl<U, G> #reui::base::Movable for #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn set_position(&mut self, position: #reui::geom::RelativePoint) {
                        self.#root_name.set_position(position);
                    }

                    #[inline]
                    fn position(&self) -> #reui::geom::RelativePoint {
                        self.#root_name.position()
                    }
                }

                impl<U, G> #reui::base::Resizable for #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn set_size(&mut self, size: #reui::reclutch::display::Size) {
                        self.#root_name.set_size(size);
                    }

                    #[inline]
                    fn size(&self) -> #reui::reclutch::display::Size {
                        self.#root_name.size()
                    }
                }

                impl<U, G> #reui::geom::StoresParentPosition for #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    fn set_parent_position(&mut self, parent_pos: #reui::geom::AbsolutePoint) {
                        self.parent_position = parent_pos;
                        self.on_transform();
                    }

                    #[inline(always)]
                    fn parent_position(&self) -> #reui::geom::AbsolutePoint {
                        self.parent_position
                    }
                }

                impl<U, G> #reui::draw::HasTheme for #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn theme(&mut self) -> &mut dyn #reui::draw::Themed {
                        &mut self.phantom_themed
                    }

                    fn resize_from_theme(&mut self) {}
                }

                impl<U, G> #reui::ui::DefaultEventQueue<#output_event> for #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn default_event_queue(&self) -> &#reui::reclutch::event::RcEventQueue<#output_event> {
                        &self.event_queue
                    }
                }

                impl<U, G> #reui::ui::DefaultWidgetData<#struct_name> for #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn default_data(&mut self) -> &mut #reui::base::Observed<#struct_name> {
                        &mut self.data
                    }
                }

                impl<U, G> Drop for #widget_name<U, G>
                where
                    U: #reui::base::UpdateAuxiliary,
                    G: #reui::base::GraphicalAuxiliary,
                {
                    fn drop(&mut self) {
                        use #reui::reclutch::prelude::*;
                        self.drop_event.emit_owned(#reui::base::DropEvent);
                    }
                }
            }
        }
            .into()
    }
}

#[proc_macro]
pub fn rooftop(stream: TokenStream) -> TokenStream {
    let data = syn::parse_macro_input!(stream as RooftopData);
    data.compile()
}
