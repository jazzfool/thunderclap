use {proc_macro::TokenStream, quote::quote};

#[derive(Debug)]
struct DataField {
    name: syn::Ident,
    default: syn::Expr,
    field_type: syn::Type,
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

#[derive(Debug)]
struct DataFieldList {
    list: Vec<DataField>,
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
    input: syn::parse::ParseStream,
) -> syn::Result<(syn::Ident, Option<DataFieldList>, FunctionBody, bool)> {
    input.parse::<syn::Token![fn]>()?;

    let fn_name = input.parse::<syn::Ident>()?;
    let parameters;
    syn::parenthesized!(parameters in input);

    let dfl =
        if parameters.is_empty() { None } else { parameters.parse::<DataFieldList>()?.into() };

    let fn_body = if fn_name.to_string() == "build" {
        let body;
        syn::braced!(body in input);
        FunctionBody::View(body)
    } else {
        let body = input.parse::<syn::Block>()?;
        FunctionBody::Other(body)
    };

    let next_fn = input.peek(syn::Token![fn]);

    Ok((fn_name, dfl, fn_body, next_fn))
}

#[derive(Debug, Clone)]
struct WidgetNode {
    type_name: syn::Ident,
    var_name: syn::Ident,
    data_assignments: Vec<DataAssignment>,
    children: Vec<WidgetNode>,
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

fn parse_view(
    input: syn::parse::ParseStream,
    bindings: &mut Vec<proc_macro2::TokenStream>,
    terminals: &mut Vec<proc_macro2::TokenStream>,
    count: &mut u64,
) -> syn::Result<(WidgetNode, bool)> {
    let type_name = input.parse::<syn::Ident>()?;
    let assignments;
    syn::parenthesized!(assignments in input);
    let data_assignments: syn::punctuated::Punctuated<_, syn::Token![,]> =
        assignments.parse_terminated(DataAssignment::parse)?;
    let mut data_assignments: Vec<_> = data_assignments.into_iter().collect();
    let var_name = if input.parse::<syn::Token![as]>().is_ok() {
        input.parse::<syn::Ident>()?
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
        }
    }

    data_assignments.retain(|assignment| !assignment.binding);

    let mut parse_terminals = true;
    let mut events = Vec::new();
    while parse_terminals {
        if input.parse::<syn::token::At>().is_ok() {
            let event_name = input.parse::<syn::Ident>()?;
            let handler_body = input.parse::<syn::Block>()?;
            events.push(quote! {
                #event_name => {
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
                    std::stringify!(#var_name) => event in #var_name.default_event_queue() => {
                        #(#events)*
                    }
                }
            }
            .into(),
        );
    }

    let mut children = Vec::new();
    if input.peek(syn::token::Brace) {
        let children_parse;
        syn::braced!(children_parse in input);

        let mut parse_child = true;
        while parse_child {
            if children_parse.is_empty() {
                parse_child = false;
            } else {
                let (node, found_comma) = parse_view(&children_parse, bindings, terminals, count)?;
                children.push(node);
                parse_child = found_comma;
            }
        }
    }

    let found_comma = input.parse::<syn::Token![,]>().is_ok();

    Ok((WidgetNode { type_name, var_name, data_assignments, children }, found_comma))
}

fn flatten_widget_node_tree(root: &WidgetNode, output: &mut Vec<WidgetNode>) {
    output.push(root.clone());
    for child in &root.children {
        flatten_widget_node_tree(child, output);
    }
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

#[derive(Debug)]
pub(crate) struct RooftopData {
    struct_name: syn::Ident,
    output_event: syn::Type,
    data_fields: DataFieldList,
    widget_tree_root: WidgetNode,
    bindings: Vec<proc_macro2::TokenStream>,
    terminals: Vec<proc_macro2::TokenStream>,
    functions: Vec<(syn::Ident, syn::Block)>,
    vis: Option<syn::Visibility>,
}

impl syn::parse::Parse for RooftopData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse::<syn::Visibility>().ok();

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
        let mut count = 0;
        let widget_tree_root = parse_view(&view_body, &mut bindings, &mut terminals, &mut count)?.0;

        Ok(RooftopData {
            struct_name,
            output_event,
            data_fields: data_fields
                .expect("failed to find data fields (parameters of build() pseudo-function)"),
            widget_tree_root,
            bindings,
            terminals,
            functions: other_functions,
            vis,
        })
    }
}

impl RooftopData {
    pub(crate) fn compile(self) -> TokenStream {
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

        let crate_name = quote::format_ident!("thunderclap");

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
                    let mut #name = #crate_name::ui::WidgetConstructor::<U, G>::construct(#type_name {
                        #(#assignments)*
                        ..<#type_name as #crate_name::ui::WidgetConstructor<U, G>>::from_theme(theme)
                    }, theme, u_aux, g_aux);
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
                    #name: <#type_name as #crate_name::ui::WidgetDataTarget<U, G>>::Target,
                }
            })
            .collect();

        let bindings = &self.bindings;
        let terminals = &self.terminals;

        let build_graph =
            find_pseudo_function("build_graph", &self.functions).unwrap_or(quote! { { graph } });
        let before_graph =
            find_pseudo_function("before_graph", &self.functions).unwrap_or_default();
        let after_graph = find_pseudo_function("after_graph", &self.functions).unwrap_or_default();
        let draw = find_pseudo_function("draw", &self.functions).unwrap_or(quote! { vec![] });
        let setup = find_pseudo_function("setup", &self.functions).unwrap_or_default();

        let define_layout = self.widget_tree_root.compile_layout();

        let root_name = &self.widget_tree_root.var_name;
        let vis = self.vis;

        {
            quote! {
                #vis struct #struct_name {
                    #(#data_fields)*
                }

                impl #struct_name {
                    pub fn from_theme(theme: &dyn #crate_name::draw::Theme) -> Self {
                        #struct_name {
                            #(#data_field_init)*
                        }
                    }

                    pub fn construct<U, G>(self, theme: &dyn #crate_name::draw::Theme, u_aux: &mut U, g_aux: &mut G) -> #widget_name<U, G>
                    where
                        U: #crate_name::base::UpdateAuxiliary,
                        G: #crate_name::base::GraphicalAuxiliary,
                    {
                        let mut data = #crate_name::base::Observed::new(self);
                        #(#widget_declarations)*
                        #define_layout;

                        use #crate_name::ui::DefaultEventQueue;
                        let mut graph = #crate_name::reclutch::verbgraph::verbgraph! {
                            #widget_name<U, G> as widget,
                            U as aux,
                            "bind" => event in &data.on_change => {
                                change => {
                                    use #crate_name::{ui::DefaultWidgetData, base::WidgetChildren};
                                    let bind = &mut widget.data;
                                    #(#bindings)*
                                    for child in &mut widget.children_mut() {
                                        child.require_update(aux, "bind");
                                    }
                                }
                            }
                            #(#terminals)*
                        };

                        graph = #build_graph;

                        // emits false positive event to apply bindings
                        data.get_mut();

                        let mut output_widget = #widget_name {
                            event_queue: Default::default(),
                            data,
                            graph: graph.into(),
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
                            use #crate_name::reclutch::widget::Widget;
                            output_widget.update(u_aux);
                        }

                        output_widget.widget_setup(theme, u_aux, g_aux);

                        output_widget

                        // Phew, all that parsing just to generate this
                    }
                }

                impl<U, G> #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    #[doc = "Auto-generated function by `rooftop!`, called automatically."]
                    fn widget_setup(&mut self, theme: &dyn #crate_name::draw::Theme, u_aux: &mut U, g_aux: &mut G) {
                        #setup
                    }
                }

                impl<U, G> #crate_name::ui::WidgetDataTarget<U, G> for #struct_name
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    type Target = #widget_name<U, G>;
                }

                #[derive(
                    WidgetChildren,
                    LayableWidget,
                    DropNotifier,
                    HasVisibility,
                    Repaintable,
                    OperatesVerbGraph,
                )]
                #[widget_children_trait(base::WidgetChildren)]
                #[thunderclap_crate(#crate_name)]
                #vis struct #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    pub event_queue: #crate_name::reclutch::event::RcEventQueue<#output_event>,
                    pub data: #crate_name::base::Observed<#struct_name>,
                    graph: #crate_name::reclutch::verbgraph::OptionVerbGraph<Self, U>,
                    parent_position: #crate_name::geom::AbsolutePoint,

                    #[widget_visibility]
                    visibility: #crate_name::base::Visibility,
                    #[repaint_target]
                    command_group: #crate_name::reclutch::display::CommandGroup,
                    #[widget_layout]
                    layout: #crate_name::base::WidgetLayoutEvents,
                    #[widget_drop_event]
                    drop_event: #crate_name::reclutch::event::RcEventQueue<#crate_name::base::DropEvent>,

                    #(#widgets_as_fields)*

                    phantom_themed: #crate_name::draw::PhantomThemed,
                    phantom_g: std::marker::PhantomData<G>,
                }

                impl<U, G> #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    fn on_transform(&mut self) {
                        use #crate_name::{base::{Repaintable}, geom::ContextuallyRectangular};
                        self.repaint();
                        self.layout.notify(self.#root_name.abs_rect());
                    }
                }

                impl<U, G> #crate_name::reclutch::verbgraph::HasVerbGraph for #widget_name<U, G>
                where
                    U: base::UpdateAuxiliary,
                    G: base::GraphicalAuxiliary,
                {
                    fn verb_graph(&mut self) -> &mut #crate_name::reclutch::verbgraph::OptionVerbGraph<Self, U> {
                        &mut self.graph
                    }
                }

                impl<U, G> #crate_name::reclutch::widget::Widget for #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    type UpdateAux = U;
                    type GraphicalAux = G;
                    type DisplayObject = #crate_name::reclutch::display::DisplayCommand;

                    #[inline]
                    fn bounds(&self) -> #crate_name::reclutch::display::Rect {
                        self.#root_name.bounds().cast_unit()
                    }

                    fn update(&mut self, aux: &mut U) {
                        #crate_name::base::invoke_update(self, aux);

                        #before_graph
                        let mut graph = self.graph.take().unwrap();
                        graph.update_all(self, aux);
                        self.graph = Some(graph);
                        #after_graph
                        if let Some(rect) = self.layout.receive() {
                            use #crate_name::geom::ContextuallyRectangular;
                            self.#root_name.set_ctxt_rect(rect);
                            self.command_group.repaint();
                        }
                    }

                    fn draw(&mut self, display: &mut dyn #crate_name::reclutch::display::GraphicsDisplay, aux: &mut G) {
                        self.command_group.push(display, &{ #draw }, Default::default(), None, None);
                    }
                }

                impl<U, G> #crate_name::base::Movable for #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn set_position(&mut self, position: #crate_name::geom::RelativePoint) {
                        self.#root_name.set_position(position);
                    }

                    #[inline]
                    fn position(&self) -> #crate_name::geom::RelativePoint {
                        self.#root_name.position()
                    }
                }

                impl<U, G> #crate_name::base::Resizable for #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn set_size(&mut self, size: #crate_name::reclutch::display::Size) {
                        self.#root_name.set_size(size);
                    }

                    #[inline]
                    fn size(&self) -> #crate_name::reclutch::display::Size {
                        self.#root_name.size()
                    }
                }

                impl<U, G> #crate_name::geom::StoresParentPosition for #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    fn set_parent_position(&mut self, parent_pos: #crate_name::geom::AbsolutePoint) {
                        self.parent_position = parent_pos;
                        self.on_transform();
                    }

                    #[inline(always)]
                    fn parent_position(&self) -> #crate_name::geom::AbsolutePoint {
                        self.parent_position
                    }
                }

                impl<U, G> #crate_name::draw::HasTheme for #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn theme(&mut self) -> &mut dyn #crate_name::draw::Themed {
                        &mut self.phantom_themed
                    }

                    fn resize_from_theme(&mut self) {}
                }

                impl<U, G> #crate_name::ui::DefaultEventQueue<#output_event> for #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn default_event_queue(&self) -> &#crate_name::reclutch::event::RcEventQueue<#output_event> {
                        &self.event_queue
                    }
                }

                impl<U, G> #crate_name::ui::DefaultWidgetData<#struct_name> for #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    #[inline]
                    fn default_data(&mut self) -> &mut #crate_name::base::Observed<#struct_name> {
                        &mut self.data
                    }
                }

                impl<U, G> Drop for #widget_name<U, G>
                where
                    U: #crate_name::base::UpdateAuxiliary,
                    G: #crate_name::base::GraphicalAuxiliary,
                {
                    fn drop(&mut self) {
                        use #crate_name::reclutch::prelude::*;
                        self.drop_event.emit_owned(#crate_name::base::DropEvent);
                    }
                }
            }
        }
            .into()
    }
}

fn find_pseudo_function(
    name: &'static str,
    functions: &[(syn::Ident, syn::Block)],
) -> Option<proc_macro2::TokenStream> {
    use quote::ToTokens;
    let block = functions.iter().find(|func| func.0 == name)?.1.clone();
    let mut tokens = proc_macro2::TokenStream::new();
    block.to_tokens(&mut tokens);
    tokens.into()
}
