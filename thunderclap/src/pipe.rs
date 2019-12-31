//! More traditional closure-based event handler.

use {
    reclutch::prelude::*,
    std::{collections::HashMap, ops::Deref, rc::Rc, sync::Mutex},
};

/// Event which returns a string corresponding to the current event variant.
pub trait Event: Clone {
    fn get_key(&self) -> &'static str;
}

/// Stores a list of event handlers, not bound to any listener.
/// This can be used to modularize parts of a pipeline.
#[derive(Clone)]
pub struct UnboundTerminal<T, A, E: Event> {
    handlers: HashMap<&'static str, Rc<Mutex<dyn FnMut(&mut T, &mut A, E)>>>,
}

impl<T, A, E: Event> UnboundTerminal<T, A, E> {
    /// Creates an empty, unbound terminal.
    pub fn new() -> Self {
        UnboundTerminal { handlers: HashMap::new() }
    }

    /// Binds an event key to a handler.
    pub fn on(
        mut self,
        ev: &'static str,
        handler: impl FnMut(&mut T, &mut A, E) + 'static,
    ) -> Self {
        self.handlers.insert(ev, Rc::new(Mutex::new(handler)));
        self
    }

    /// Binds the handlers to an event queue, hence returning a `Terminal`.
    pub fn bind<D: QueueInterfaceListable<Item = E, Listener = L>, L: EventListen<Item = E>>(
        self,
        queue: &impl Deref<Target = D>,
    ) -> Terminal<T, A, E, L> {
        Terminal { handlers: self.handlers, listener: queue.listen() }
    }
}

/// Stores a list of event handlers and a single event listener.
/// Events (`Event`) with `get_key` matching to a handler's "name" will invoke the corresponding handler.
pub struct Terminal<T, A, E: Event, L: EventListen<Item = E>> {
    handlers: HashMap<&'static str, Rc<Mutex<dyn FnMut(&mut T, &mut A, E)>>>,
    listener: L,
}

impl<T, A, E: Event, L: EventListen<Item = E>> Terminal<T, A, E, L> {
    /// Creates a new terminal, connected to an event.
    pub fn new<D: QueueInterfaceListable<Item = E, Listener = L>>(
        queue: &impl Deref<Target = D>,
    ) -> Self {
        Terminal { handlers: HashMap::new(), listener: queue.listen() }
    }

    /// Binds an event key to a handler.
    pub fn on(
        mut self,
        ev: &'static str,
        handler: impl FnMut(&mut T, &mut A, E) + 'static,
    ) -> Self {
        self.handlers.insert(ev, Rc::new(Mutex::new(handler)));
        self
    }
}

/// Implemented by all bound terminals.
trait DynTerminal<T, A> {
    /// Invokes the underlying closure with the given callbacks.
    fn update(&mut self, obj: &mut T, additional: &mut A);
}

impl<T, A, E: Event, L: EventListen<Item = E>> DynTerminal<T, A> for Terminal<T, A, E, L> {
    fn update(&mut self, obj: &mut T, additional: &mut A) {
        for event in self.listener.peek() {
            if let Some(handler) = self.handlers.get_mut(event.get_key()) {
                use std::ops::DerefMut;
                let mut handler = handler.lock().unwrap();
                handler.deref_mut()(obj, additional, event.clone());
            }
        }
    }
}

/// Stores a list of terminals and updates each one when invoked.
pub struct Pipeline<T: 'static, A: 'static> {
    terminals: Vec<Box<dyn DynTerminal<T, A>>>,
}

impl<T: 'static, A: 'static> Default for Pipeline<T, A> {
    fn default() -> Self {
        Pipeline { terminals: Default::default() }
    }
}

impl<T: 'static, A: 'static> Pipeline<T, A> {
    /// Creates a pipeline with no terminals.
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a terminal to the pipeline.
    pub fn add<E: Event + 'static, L: EventListen<Item = E> + 'static>(
        mut self,
        terminal: Terminal<T, A, E, L>,
    ) -> Self {
        self.terminals.push(Box::new(terminal));
        self
    }

    /// Invokes update on all the terminals with the given parameters.
    pub fn update(&mut self, obj: &mut T, additional: &mut A) {
        for terminal in &mut self.terminals {
            terminal.update(obj, additional);
        }
    }
}

/// Simplifies creation of pipelines.
///
/// # Example
/// ```ignore
/// pipeline! {
///     Counter as obj,
///     UpdateAux as aux,
///     _event in &count_up.event_queue => {
///         click { obj.count += 1 }
///     }
///     _event in &count_down.event_queue => {
///         click { obj.count -= 1 }
///     }
/// }
/// ```
/// Which expands to
/// ```ignore
/// Pipeline::new()
/// .add(Terminal::new(&count_up.event_queue).on(
///     "click",
///     |obj: &mut Counter, _aux: &mut UpdateAux, _event| {
///         let _event = _event.unwrap_as_click().unwrap();
///         { obj.count += 1; }
///     },
/// ))
/// .add(Terminal::new(&count_down.event_queue).on(
///     "click",
///     |obj: &mut Counter, _aux: &mut UpdateAux, _event| {
///         let _event = _event.unwrap_as_click().unwrap();
///         { obj.count -= 1; }
///     },
/// ));
/// ```
#[macro_export]
macro_rules! pipeline {
    ($ot:ty as $obj:ident,$at:ty as $add:ident,$($eo:ident in $eq:expr=>{$($ev:tt $body:block)*})*) => {{
        let mut pipe = $crate::pipe::Pipeline::new();
        $(
            let mut terminal = $crate::pipe::Terminal::new($eq);
            $(
                terminal = terminal.on(
                    std::stringify!($ev),
                    |$obj: &mut $ot, $add: &mut $at, #[allow(unused_variables)] $eo| {
                        #[allow(unused_variables)]
                        $crate::paste::expr!{
                            let $eo = $eo.[<unwrap_as_ $ev>]().unwrap();
                            $body
                        }
                    });
            )*
            pipe = pipe.add(terminal);
        )*
        pipe
    }};
}

#[macro_export]
macro_rules! unbound_terminal {
    ($ot:ty as $obj:ident,$at:ty as $add:ident,$et:ty as $eo:ident,$($ev:tt $body:block)*) => {{
        let mut terminal = $crate::pipe::UnboundTerminal::new();
        $(
            terminal = terminal.on(
                std::stringify!($ev),
                |$obj: &mut $ot, #[allow(unused_variables)] $add: &mut $at, $eo: $et| {
                    #[allow(unused_variables)]
                    $crate::paste::expr!{
                        let $eo = $eo.[<unwrap_as_ $ev>]().unwrap();
                        $body
                    }
                });
        )*
        terminal
    }}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipelines() {
        use reclutch::event::RcEventQueue;

        #[derive(PipelineEvent, Clone)]
        #[thunderclap_crate(crate)]
        enum TestEvent {
            #[event_key(count_up)]
            CountUp,
            #[event_key(count_down)]
            CountDown,
        }

        let event_queue: RcEventQueue<TestEvent> = Default::default();

        struct Test {
            pipe: Option<Pipeline<Test, ()>>,
            count: u32,
        }

        impl Test {
            fn update(&mut self) {
                let mut pipe = self.pipe.take().unwrap();
                pipe.update(self, &mut ());
                self.pipe = Some(pipe);
            }
        }

        let mut test = Test {
            pipe: pipeline! {
                Test as obj,
                () as _aux,
                _ev in &event_queue => {
                    count_up { obj.count += 1; }
                    count_down { obj.count -= 1; }
                }
            }
            .into(),
            count: 0,
        };

        test.update(); // 0

        event_queue.emit_owned(TestEvent::CountUp); // 1
        event_queue.emit_owned(TestEvent::CountUp); // 2
        event_queue.emit_owned(TestEvent::CountUp); // 3
        event_queue.emit_owned(TestEvent::CountDown); // 2 - final value

        test.update();

        assert_eq!(test.count, 2);
    }
}
