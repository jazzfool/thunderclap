//! More traditional closure-based event handler on top `RcEventQueue`.

// TODO(jazzfool): support more event queues.

use {
    reclutch::{
        event::{RcEventListener, RcEventQueue},
        prelude::*,
    },
    std::collections::HashMap,
};

/// Event which returns a string corresponding to the current event variant.
pub trait Event: Clone {
    fn get_key(&self) -> &'static str;
}

/// Stores a list of event handlers and a single event listener.
/// Events (`Event`) with `get_key` matching to a handler's "name" will invoke the corresponding handler.
pub struct Terminal<T, A, E: Event> {
    handlers: HashMap<&'static str, Box<dyn FnMut(&mut T, &mut A, E)>>,
    listener: RcEventListener<E>,
    phantom: std::marker::PhantomData<T>,
}

impl<T, A, E: Event> Terminal<T, A, E> {
    /// Creates a new terminal, connected to an event.
    pub fn new(queue: &RcEventQueue<E>) -> Self {
        Terminal {
            handlers: HashMap::new(),
            listener: queue.listen(),
            phantom: Default::default(),
        }
    }

    /// Binds an event key to a handler.
    pub fn on(
        mut self,
        ev: &'static str,
        handler: impl FnMut(&mut T, &mut A, E) + 'static,
    ) -> Self {
        self.handlers.insert(ev, Box::new(handler));
        self
    }
}

/// Implemented by all terminals.
trait DynTerminal<T, A> {
    /// Invokes the underlying closure with the given callbacks.
    fn update(&mut self, obj: &mut T, additional: &mut A);
}

impl<T, A, E: Event> DynTerminal<T, A> for Terminal<T, A, E> {
    fn update(&mut self, obj: &mut T, additional: &mut A) {
        for event in self.listener.peek() {
            if let Some(handler) = self.handlers.get_mut(event.get_key()) {
                (**handler)(obj, additional, event.clone());
            }
        }
    }
}

/// Stores a list of terminals and updates each one when invoked.
pub struct Pipeline<T: 'static, A: 'static> {
    terminals: Vec<Box<dyn DynTerminal<T, A>>>,
}

impl<T: 'static, A: 'static> Pipeline<T, A> {
    /// Creates a pipeline with no terminals.
    pub fn new() -> Self {
        Pipeline {
            terminals: Vec::new(),
        }
    }

    /// Adds a terminal to the pipeline.
    pub fn add<E: Event + 'static>(mut self, terminal: Terminal<T, A, E>) -> Self {
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
///         obj.count += 1;
///     },
/// ))
/// .add(Terminal::new(&count_down.event_queue).on(
///     "click",
///     |obj: &mut Counter, _aux: &mut UpdateAux, _event| {
///         obj.count -= 1;
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
                terminal = terminal.on(std::stringify!($ev), |$obj: &mut $ot, $add: &mut $at, #[allow(unused_variables)] $eo| $body);
            )*
            pipe = pipe.add(terminal);
        )*
        pipe
    }};
}

#[macro_export]
macro_rules! force_event {
    ($ev:ident,$evt:path) => {
        let $ev = if let $evt(x) = $ev { x } else { panic!() };
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipelines() {
        #[derive(Clone)]
        enum TestEvent {
            CountUp,
            CountDown,
        }

        impl Event for TestEvent {
            fn get_key(&self) -> &'static str {
                match self {
                    TestEvent::CountUp => "count_up",
                    TestEvent::CountDown => "count_down",
                }
            }
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
