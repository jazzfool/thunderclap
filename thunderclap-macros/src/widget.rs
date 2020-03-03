use quote::quote;

#[derive(Debug, Clone, Copy)]
enum DeclType {
    Meta,
    Field,
    Impl,
    InitField,
    InitImpl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WidgetTrait {
    WidgetChildren,
    LayableWidget,
    DropNotifier,
    HasVisibility,
    Repaintable,
    Rectangular,
    OperatesVerbGraph,
    StoresParentPosition,
    EventQueue(Box<syn::Type>),
    State(Box<syn::Type>),
    Painter(Box<syn::Type>),
}

impl WidgetTrait {
    fn is_painter(&self) -> bool {
        match self {
            WidgetTrait::Painter(_) => true,
            _ => false,
        }
    }
}

fn widget_children_decl(ty: DeclType) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => {
            quote! {
                #[derive(WidgetChildren)]
                #[widget_children_trait(thunderclap::base::WidgetChildren)]
            }
        }
        DeclType::Field | DeclType::Impl | DeclType::InitField | DeclType::InitImpl => {
            Default::default()
        }
    }
}

fn layable_widget_decl(ty: DeclType) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => {
            quote! {
                #[derive(LayableWidget)]
            }
        }
        DeclType::Field => {
            quote! {
                #[widget_layout]
                layout: thunderclap::base::WidgetLayoutEvents
            }
        }
        DeclType::Impl | DeclType::InitField => Default::default(),
        DeclType::InitImpl => {
            quote! {
                layout: Default::default()
            }
        }
    }
}

fn drop_notifier_decl(
    ty: DeclType,
    generic_list: &proc_macro2::TokenStream,
    where_clause: &proc_macro2::TokenStream,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => {
            quote! {
                #[derive(DropNotifier)]
            }
        }
        DeclType::Field => {
            quote! {
                #[widget_drop_event]
                drop_event: thunderclap::reclutch::event::RcEventQueue<thunderclap::base::DropEvent>
            }
        }
        DeclType::Impl => {
            quote! {
                impl<U, G, #generic_list> Drop for #name<U, G, #generic_list>
                #where_clause
                    U: thunderclap::base::UpdateAuxiliary,
                    G: thunderclap::base::GraphicalAuxiliary,
                {
                    fn drop(&mut self) {
                        use thunderclap::reclutch::prelude::*;
                        self.drop_event.emit_owned(base::DropEvent);
                    }
                }
            }
        }
        DeclType::InitField => Default::default(),
        DeclType::InitImpl => {
            quote! {
                drop_event: Default::default()
            }
        }
    }
}

fn has_visibility_decl(ty: DeclType) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => {
            quote! {
                #[derive(HasVisibility)]
            }
        }
        DeclType::Field => {
            quote! {
                #[widget_visibility]
                visibility: thunderclap::base::Visibility
            }
        }
        DeclType::Impl => Default::default(),
        DeclType::InitField => Default::default(),
        DeclType::InitImpl => {
            quote! {
                visibility: Default::default()
            }
        }
    }
}

fn repaintable_decl(ty: DeclType) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => {
            quote! {
                #[derive(Repaintable)]
            }
        }
        DeclType::Field => {
            quote! {
                #[repaint_target]
                command_group: thunderclap::reclutch::display::CommandGroup
            }
        }
        DeclType::Impl | DeclType::InitField => Default::default(),
        DeclType::InitImpl => {
            quote! {
                command_group: Default::default()
            }
        }
    }
}

fn rectangular_decl(ty: DeclType) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => {
            quote! {
                #[derive(Resizable, Movable)]
                #[widget_transform_callback(on_transform)]
            }
        }
        DeclType::Field => {
            quote! {
                #[widget_rect]
                rect: thunderclap::geom::RelativeRect
            }
        }
        DeclType::Impl => Default::default(),
        DeclType::InitField => {
            quote! {
                rect: thunderclap::geom::RelativeRect
            }
        }
        DeclType::InitImpl => {
            quote! {
                rect: self.rect
            }
        }
    }
}

fn operates_verb_graph_decl(
    ty: DeclType,
    generic_list: &proc_macro2::TokenStream,
    where_clause: &proc_macro2::TokenStream,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => {
            quote! {
                #[derive(OperatesVerbGraph)]
            }
        }
        DeclType::Field => {
            quote! {
                graph: thunderclap::reclutch::verbgraph::OptionVerbGraph<Self, U>
            }
        }
        DeclType::Impl => {
            quote! {
                impl<U, G, #generic_list> thunderclap::reclutch::verbgraph::HasVerbGraph for #name<U, G, #generic_list>
                #where_clause
                    U: thunderclap::base::UpdateAuxiliary,
                    G: thunderclap::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn verb_graph(&mut self) -> &mut thunderclap::reclutch::verbgraph::OptionVerbGraph<Self, U> {
                        &mut self.graph
                    }
                }
            }
        }
        DeclType::InitField => {
            quote! {
                graph: thunderclap::reclutch::verbgraph::OptionVerbGraph<#name<U, G, #generic_list>, U>
            }
        }
        DeclType::InitImpl => {
            quote! {
                graph: self.graph
            }
        }
    }
}

fn stores_parent_position_decl(
    ty: DeclType,
    generic_list: &proc_macro2::TokenStream,
    where_clause: &proc_macro2::TokenStream,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => Default::default(),
        DeclType::Field => {
            quote! {
                parent_position: thunderclap::geom::AbsolutePoint
            }
        }
        DeclType::Impl => {
            quote! {
                impl<U, G, #generic_list> thunderclap::geom::StoresParentPosition for #name<U, G, #generic_list>
                #where_clause
                    U: thunderclap::base::UpdateAuxiliary,
                    G: thunderclap::base::GraphicalAuxiliary,
                {
                    fn set_parent_position(&mut self, parent_pos: thunderclap::geom::AbsolutePoint) {
                        use thunderclap::ui::core::CoreWidget;
                        self.parent_position = parent_pos;
                        self.on_transform();
                    }

                    #[inline]
                    fn parent_position(&self) -> thunderclap::geom::AbsolutePoint {
                        self.parent_position
                    }
                }
            }
        }
        DeclType::InitField => Default::default(),
        DeclType::InitImpl => {
            quote! {
                parent_position: Default::default()
            }
        }
    }
}

fn event_queue_decl(
    gty: syn::Type,
    ty: DeclType,
    generic_list: &proc_macro2::TokenStream,
    where_clause: &proc_macro2::TokenStream,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => Default::default(),
        DeclType::Field => {
            quote! {
                pub event_queue: thunderclap::reclutch::event::RcEventQueue<#gty>
            }
        }
        DeclType::Impl => {
            quote! {
                impl<U, G, #generic_list> thunderclap::ui::DefaultEventQueue<#gty> for #name<U, G, #generic_list>
                #where_clause
                    U: thunderclap::base::UpdateAuxiliary,
                    G: thunderclap::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn default_event_queue(&self) -> &thunderclap::reclutch::event::RcEventQueue<#gty> {
                        &self.event_queue
                    }
                }
            }
        }
        DeclType::InitField => Default::default(),
        DeclType::InitImpl => {
            quote! {
                event_queue: Default::default()
            }
        }
    }
}

fn state_decl(
    gty: syn::Type,
    ty: DeclType,
    generic_list: &proc_macro2::TokenStream,
    where_clause: &proc_macro2::TokenStream,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => Default::default(),
        DeclType::Field => {
            quote! {
                pub data: thunderclap::base::Observed<#gty>
            }
        }
        DeclType::Impl => {
            quote! {
                impl<U, G, #generic_list> thunderclap::ui::DefaultWidgetData<#gty> for #name<U, G, #generic_list>
                #where_clause
                    U: thunderclap::base::UpdateAuxiliary,
                    G: thunderclap::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn default_data(&mut self) -> &mut thunderclap::base::Observed<#gty> {
                        &mut self.data
                    }
                }
            }
        }
        DeclType::InitField => {
            quote! {
                data: thunderclap::base::Observed<#gty>
            }
        }
        DeclType::InitImpl => {
            quote! {
                data: self.data
            }
        }
    }
}

fn painter_decl(
    gty: syn::Type,
    ty: DeclType,
    generic_list: &proc_macro2::TokenStream,
    where_clause: &proc_macro2::TokenStream,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    match ty {
        DeclType::Meta => Default::default(),
        DeclType::Field => {
            quote! {
                painter: Box<dyn thunderclap::draw::Painter<#gty>>
            }
        }
        DeclType::Impl => {
            quote! {
                impl<U, G, #generic_list> thunderclap::draw::HasTheme for #name<U, G, #generic_list>
                #where_clause
                    U: thunderclap::base::UpdateAuxiliary,
                    G: thunderclap::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn theme(&mut self) -> &mut dyn thunderclap::draw::Themed {
                        &mut self.painter
                    }

                    fn resize_from_theme(&mut self) {
                        use thunderclap::{base::Resizable, ui::core::CoreWidget};
                        self.set_size(self.painter.size_hint(self.derive_state()));
                    }
                }
            }
        }
        DeclType::InitField => {
            quote! {
                painter: Box<dyn thunderclap::draw::Painter<#gty>>
            }
        }
        DeclType::InitImpl => {
            quote! {
                painter: self.painter
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Generics {
    params: proc_macro2::TokenStream,
    where_clause: proc_macro2::TokenStream,
}

impl syn::parse::Parse for Generics {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let generics = input.parse::<syn::Generics>()?;
        let params: Vec<syn::GenericParam> = generics.params.into_iter().collect();
        let mut where_clause: Vec<proc_macro2::TokenStream> = generics
            .where_clause
            .map(|x| x.predicates.into_iter().map(|x| quote! { #x, }).collect())
            .unwrap_or_default();
        let mut simple_params = Vec::new();

        // Move all bounds to the where clause
        for param in &params {
            match param {
                syn::GenericParam::Type(p) => {
                    let ident = &p.ident;
                    if !p.bounds.is_empty() {
                        let bounds = &p.bounds;
                        where_clause.push(quote! {
                            #ident: #bounds,
                        });
                    }

                    simple_params.push(quote! { #ident });
                }
                syn::GenericParam::Lifetime(p) => {
                    let ident = &p.lifetime;
                    if !p.bounds.is_empty() {
                        let bounds = &p.bounds;
                        where_clause.push(quote! {
                            #ident: #bounds,
                        });
                    }

                    simple_params.push(quote! { #ident });
                }
                syn::GenericParam::Const(p) => simple_params.push(quote! { #p }),
            }
        }

        Ok(Generics {
            params: quote! {
                #(#simple_params),*
            },
            where_clause: quote! {
                where #(#where_clause)*
            },
        })
    }
}

fn decl_for(
    tr: WidgetTrait,
    ty: DeclType,
    generics: Option<&Generics>,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let generic_list = generics.map(|x| x.params.clone()).unwrap_or(quote! {});
    let where_clause = generics.map(|x| x.where_clause.clone()).unwrap_or(quote! { where });

    match tr {
        WidgetTrait::WidgetChildren => widget_children_decl(ty),
        WidgetTrait::LayableWidget => layable_widget_decl(ty),
        WidgetTrait::DropNotifier => drop_notifier_decl(ty, &generic_list, &where_clause, name),
        WidgetTrait::HasVisibility => has_visibility_decl(ty),
        WidgetTrait::Repaintable => repaintable_decl(ty),
        WidgetTrait::Rectangular => rectangular_decl(ty),
        WidgetTrait::OperatesVerbGraph => {
            operates_verb_graph_decl(ty, &generic_list, &where_clause, name)
        }
        WidgetTrait::StoresParentPosition => {
            stores_parent_position_decl(ty, &generic_list, &where_clause, name)
        }
        WidgetTrait::EventQueue(gty) => {
            event_queue_decl(*gty, ty, &generic_list, &where_clause, name)
        }
        WidgetTrait::State(gty) => state_decl(*gty, ty, &generic_list, &where_clause, name),
        WidgetTrait::Painter(gty) => painter_decl(*gty, ty, &generic_list, &where_clause, name),
    }
}

struct WidgetImpl {
    tr: Option<WidgetTrait>,
    meta_decl: proc_macro2::TokenStream,
    field_decl: proc_macro2::TokenStream,
    impl_decl: proc_macro2::TokenStream,
    init_field_decl: proc_macro2::TokenStream,
    init_impl_decl: proc_macro2::TokenStream,
}

impl WidgetImpl {
    fn new(field: WidgetField, generics: Option<&Generics>, name: &syn::Ident) -> Vec<Self> {
        match field {
            WidgetField::WidgetMax => [
                "WidgetChildren",
                "LayableWidget",
                "DropNotifier",
                "HasVisibility",
                "Repaintable",
                "Rectangular",
                "OperatesVerbGraph",
                "StoresParentPosition",
            ]
            .iter()
            .map(|x| {
                WidgetImpl::new(WidgetField::Pseudo(quote::format_ident!("{}", x)), generics, name)
                    .remove(0)
            })
            .collect(),
            WidgetField::Pseudo(ident) => {
                let tr = match &ident.to_string()[..] {
                    "WidgetChildren" => WidgetTrait::WidgetChildren,
                    "LayableWidget" => WidgetTrait::LayableWidget,
                    "DropNotifier" => WidgetTrait::DropNotifier,
                    "HasVisibility" => WidgetTrait::HasVisibility,
                    "Repaintable" => WidgetTrait::Repaintable,
                    "Rectangular" => WidgetTrait::Rectangular,
                    "OperatesVerbGraph" => WidgetTrait::OperatesVerbGraph,
                    "StoresParentPosition" => WidgetTrait::StoresParentPosition,
                    _ => panic!("Unknown trait '{}'", ident.to_string()),
                };

                vec![WidgetImpl {
                    meta_decl: decl_for(tr.clone(), DeclType::Meta, generics, name),
                    field_decl: decl_for(tr.clone(), DeclType::Field, generics, name),
                    impl_decl: decl_for(tr.clone(), DeclType::Impl, generics, name),
                    init_field_decl: decl_for(tr.clone(), DeclType::InitField, generics, name),
                    init_impl_decl: decl_for(tr.clone(), DeclType::InitImpl, generics, name),
                    tr: tr.into(),
                }]
            }
            WidgetField::Generic(b) => {
                let (ident, ty) = *b;
                let tr = match &ident.to_string()[..] {
                    "EventQueue" => WidgetTrait::EventQueue(Box::new(ty)),
                    "State" => WidgetTrait::State(Box::new(ty)),
                    "Painter" => WidgetTrait::Painter(Box::new(ty)),
                    _ => panic!("Unknown generic trait '{}'", ident.to_string()),
                };

                vec![WidgetImpl {
                    meta_decl: decl_for(tr.clone(), DeclType::Meta, generics, name),
                    field_decl: decl_for(tr.clone(), DeclType::Field, generics, name),
                    impl_decl: decl_for(tr.clone(), DeclType::Impl, generics, name),
                    init_field_decl: decl_for(tr.clone(), DeclType::InitField, generics, name),
                    init_impl_decl: decl_for(tr.clone(), DeclType::InitImpl, generics, name),
                    tr: tr.into(),
                }]
            }
            WidgetField::Fields(fields) => {
                let struct_fields: Vec<_> = fields
                    .iter()
                    .cloned()
                    .map(|field| {
                        let attrs = field.attrs;
                        let vis = field.vis;
                        let name = field.ident.unwrap();
                        let ty = field.ty;

                        quote! {
                            #(#[#attrs])*
                            #vis #name: #ty
                        }
                    })
                    .collect();

                let init_fields: Vec<_> = fields
                    .iter()
                    .cloned()
                    .map(|field| {
                        let name = field.ident.unwrap();
                        let ty = field.ty;

                        quote! {
                            #name: #ty
                        }
                    })
                    .collect();

                let init_impls: Vec<_> = fields
                    .iter()
                    .cloned()
                    .map(|field| {
                        let name = field.ident.unwrap();

                        quote! {
                            #name: self.#name
                        }
                    })
                    .collect();

                vec![WidgetImpl {
                    tr: None,
                    meta_decl: Default::default(),
                    field_decl: quote! {
                        #(#struct_fields),*
                    },
                    impl_decl: Default::default(),
                    init_field_decl: quote! {
                        #(#init_fields),*
                    },
                    init_impl_decl: quote! {
                        #(#init_impls),*
                    },
                }]
            }
        }
    }
}

enum WidgetField {
    WidgetMax,
    Pseudo(syn::Ident),
    Generic(Box<(syn::Ident, syn::Type)>),
    Fields(syn::punctuated::Punctuated<syn::Field, syn::Token![,]>),
}

mod kw {
    syn::custom_keyword!(widget);
    syn::custom_keyword!(MAX);
}

impl syn::parse::Parse for WidgetField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<kw::widget>().is_ok()
            && input.parse::<syn::Token![::]>().is_ok()
            && input.parse::<kw::MAX>().is_ok()
        {
            Ok(WidgetField::WidgetMax)
        } else if input.peek(syn::Ident) {
            Ok(WidgetField::Pseudo(input.parse::<syn::Ident>()?))
        } else if input.parse::<syn::Token![<]>().is_ok() {
            let ty = input.parse::<syn::Type>()?;
            input.parse::<syn::Token![>]>()?;
            let name = input.parse::<syn::Ident>()?;
            Ok(WidgetField::Generic(Box::new((name, ty))))
        } else if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            Ok(WidgetField::Fields(
                content.parse_terminated::<_, syn::Token![,]>(syn::Field::parse_named)?,
            ))
        } else {
            Err(input.error("Unrecognized widget field configuration"))
        }
    }
}

pub struct WidgetImpls {
    impls: Vec<WidgetImpl>,
    name: syn::Ident,
    generics: Option<Generics>,
    vis: Option<syn::Visibility>,
    attrs: Vec<syn::Attribute>,
}

impl WidgetImpls {
    pub fn compile(mut self) -> proc_macro2::TokenStream {
        let name = self.name;
        let generic_list = self.generics.clone().map(|x| x.params).unwrap_or_default();
        let where_clause =
            self.generics.map(|x| x.where_clause).unwrap_or_else(|| quote! { where });

        if self.impls.iter().find(|x| x.tr.as_ref().map_or(false, |y| y.is_painter())).is_none() {
            self.impls.push(WidgetImpl {
                tr: None,
                meta_decl: Default::default(),
                field_decl: quote! {
                    painter: thunderclap::draw::PhantomThemed
                },
                impl_decl: quote! {
                    impl<U, G, #generic_list> thunderclap::draw::HasTheme for #name<U, G, #generic_list>
                    #where_clause
                        U: thunderclap::base::UpdateAuxiliary,
                        G: thunderclap::base::GraphicalAuxiliary,
                    {
                        #[inline]
                        fn theme(&mut self) -> &mut dyn thunderclap::draw::Themed {
                            &mut self.painter
                        }

                        fn resize_from_theme(&mut self) {}
                    }
                },
                init_field_decl: Default::default(),
                init_impl_decl: quote! {
                    painter: Default::default()
                }
            })
        }

        let metas: Vec<_> = self.impls.iter().map(|x| x.meta_decl.clone()).collect();
        let mut fields: Vec<_> = self.impls.iter().map(|x| x.field_decl.clone()).collect();
        let impls: Vec<_> = self.impls.iter().map(|x| x.impl_decl.clone()).collect();
        let mut init_fields: Vec<_> =
            self.impls.iter().map(|x| x.init_field_decl.clone()).collect();
        let mut init_impls: Vec<_> = self.impls.iter().map(|x| x.init_impl_decl.clone()).collect();

        fields.retain(|x| !x.is_empty());
        init_fields.retain(|x| !x.is_empty());
        init_impls.retain(|x| !x.is_empty());

        let vis = self.vis;
        let attrs = self.attrs;

        let builder_name = quote::format_ident!("{}Builder", name);

        quote! {
            use thunderclap::ui::core::CoreWidget;

            #vis struct #builder_name<U, G, #generic_list>
            #where_clause
                U: thunderclap::base::UpdateAuxiliary,
                G: thunderclap::base::GraphicalAuxiliary,
            {
                #(#init_fields),*
            }

            impl<U, G, #generic_list> #builder_name<U, G, #generic_list>
            #where_clause
                U: thunderclap::base::UpdateAuxiliary,
                G: thunderclap::base::GraphicalAuxiliary,
            {
                pub fn build(self) -> #name<U, G, #generic_list> {
                    #name {
                        #(#init_impls),*
                    }
                }
            }

            #(#attrs)*
            #(#metas)*
            #vis struct #name<U, G, #generic_list>
            #where_clause
                U: thunderclap::base::UpdateAuxiliary,
                G: thunderclap::base::GraphicalAuxiliary,
            {
                #(#fields),*
            }

            #(#impls)*
        }
    }
}

impl syn::parse::Parse for WidgetImpls {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = syn::Attribute::parse_outer(input)?;

        let vis = input.parse::<syn::Visibility>().ok();

        input.parse::<syn::Token![struct]>()?;
        let name = input.parse::<syn::Ident>()?;

        let generics =
            if input.peek(syn::Token![<]) { input.parse::<Generics>().ok() } else { None };

        let struct_content;
        syn::braced!(struct_content in input);
        let impls = struct_content
            .parse_terminated::<WidgetField, syn::Token![,]>(WidgetField::parse)?
            .into_iter()
            .map(|field| WidgetImpl::new(field, generics.as_ref(), &name))
            .fold(Vec::new(), |mut v, x| {
                v.extend(x.into_iter());
                v
            });

        Ok(WidgetImpls { impls, name, generics, vis, attrs })
    }
}
