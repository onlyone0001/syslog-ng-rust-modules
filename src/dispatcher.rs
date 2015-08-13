use std::sync::mpsc::{Receiver, Sender};
use std::rc::Rc;

use action::ExecResult;
use super::{config, Condition, Context, Message, TimerEvent};
use reactor::{self, Event, EventDemultiplexer, EventHandler, Reactor};

#[derive(Debug)]
pub enum Request {
    Event(super::Event),
    Exit
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RequestHandler {
    Event,
    Exit
}

impl reactor::Event for Request {
    type Handler = RequestHandler;
    fn handler(&self) -> Self::Handler {
        match *self {
            Request::Event(_) => RequestHandler::Event,
            Request::Exit => RequestHandler::Exit,
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Event(ExecResult),
    Exit
}

pub struct Dispatcher {
    contexts: Vec<Context>,
    output_channel: Sender<Response>,
    exits_received: u32
}

impl Dispatcher {
    pub fn new(contexts: Vec<config::Context>, action_output_channel: Sender<Response>) -> Dispatcher {
        let contexts = contexts.into_iter().map(|ctx| Context::from(ctx)).collect::<Vec<Context>>();
        Dispatcher {
            contexts: contexts,
            output_channel: action_output_channel,
            exits_received: 0
        }
    }

    pub fn start_loop(&mut self, channel: Receiver<Request>) {
        for i in channel.iter() {
            match i {
                Request::Event(event) => self.dispatch(event),
                Request::Exit => {
                    if self.on_exit() {
                        break;
                    }
                }
            }
        }
    }

    fn on_exit(&mut self) -> bool {
        self.exits_received += 1;
        let _ = self.output_channel.send(Response::Exit);
        self.exits_received >= 2
    }

    pub fn dispatch(&mut self, event: super::Event) {
        match event {
            super::Event::Message(event) => {
                let event = Rc::new(event);
                self.on_message(event);
            },
            super::Event::Timer(ref event) => {
                self.on_timer(event);
            }
        };
    }

    fn on_message(&mut self, event: Rc<Message>) {
        for context in self.contexts.iter_mut() {
            if let Some(result) = context.on_message(event.clone()) {
                for i in result.into_iter() {
                    let _ = self.output_channel.send(i.into());
                }
            }
        }
    }

    fn on_timer(&mut self, event: &TimerEvent) {
        for context in self.contexts.iter_mut() {
            if let Some(result) = context.on_timer(event) {
                for i in result.into_iter() {
                    let _ = self.output_channel.send(i.into());
                }
            }
        }
    }
}

use std::collections::BTreeMap;

pub struct RequestReactor {
    handlers: BTreeMap<RequestHandler, Box<reactor::EventHandler<Request, Handler=RequestHandler>>>,
    demultiplexer: Demultiplexer<Request>,
    exit_condition: Condition
}

impl RequestReactor {
    fn new(demultiplexer: Demultiplexer<Request>) -> RequestReactor {
        let exit_condition = Condition::new(false);
        let exit_handler = Box::new(handlers::exit::ExitHandler::new(exit_condition.clone()));
        let event_handler = Box::new(handlers::event::EventHandler::new());

        let mut reactor = RequestReactor {
            demultiplexer: demultiplexer,
            exit_condition: exit_condition,
            handlers: BTreeMap::new()
        };

        reactor.register_handler(exit_handler);
        reactor.register_handler(event_handler);
        reactor
    }
}

impl Reactor for RequestReactor {
    type Event = Request;
    type Handler = RequestHandler;
    fn handle_events(&mut self) {
        while !self.exit_condition.is_active() {
            if let Some(request) = self.demultiplexer.select() {
                let mut handler = self.handlers.get_mut(&request.handler()).unwrap();
                handler.handle_event(request);
            } else {
                break;
            }
        }
    }
    fn register_handler(&mut self, handler: Box<reactor::EventHandler<Self::Event, Handler=RequestHandler>>) {
        self.handlers.insert(handler.handler(), handler);
    }
    fn remove_handler(&mut self, handler: &reactor::EventHandler<Self::Event, Handler=RequestHandler>){}
}

struct Demultiplexer<T>(Receiver<T>);

impl reactor::EventDemultiplexer for Demultiplexer<Request> {
    type Event = Request;
    fn select(&mut self) -> Option<Self::Event> {
        self.0.recv().ok()
    }
}

mod handlers {
    pub mod exit {
        use dispatcher::{Request, RequestHandler};
        use condition::Condition;
        use reactor::EventHandler;

        pub struct ExitHandler{
            condition: Condition,
            stops: u32
        }

        impl ExitHandler {
            pub fn new(condition: Condition) -> ExitHandler {
                ExitHandler {
                    condition: condition,
                    stops: 0
                }
            }
        }

        impl EventHandler<Request> for ExitHandler {
            type Handler = RequestHandler;
            fn handle_event(&mut self, event: Request) {
                if let Request::Exit = event {
                    self.stops += 1;

                    if self.stops >= 2 {
                        self.condition.activate();
                    }
                } else {
                    unreachable!("An ExitHandler should only receive Exit events");
                }
            }
            fn handler(&self) -> Self::Handler {
                RequestHandler::Exit
            }
        }
    }

    pub mod event {
        use dispatcher::{Request, RequestHandler};
        use reactor;

        pub struct EventHandler;

        impl EventHandler {
            pub fn new() -> EventHandler {
                EventHandler
            }
        }

        impl reactor::EventHandler<Request> for EventHandler {
            type Handler = RequestHandler;
            fn handle_event(&mut self, event: Request) {
                if let Request::Event(_) = event {
                    println!("Event recvd");
                } else {
                    unreachable!("An EventHandler should only receive Event events");
                }
            }
            fn handler(&self) -> Self::Handler {
                RequestHandler::Event
            }
        }
    }

    mod linear {
        use dispatcher::{Request, RequestHandler};
        use EventHandler;
        use Event;
        use reactor;

        pub struct LinearHandler<C, H> {
            contexts: Vec<C>,
            handler: H
        }

        impl<C, H> LinearHandler<C, H> {
            pub fn new(handler: H) -> LinearHandler<C, H> {
                LinearHandler {
                    contexts: Vec::new(),
                    handler: handler
                }
            }
        }

        /*impl<C, H, E> reactor::EventHandler<E> for LinearHandler<C, H> {
            type Handler = H;
            fn handle_event(&mut self, event: E) {

            }
            fn handler(&self) -> Self::Handler {
                self.handler
            }
        }*/
    }
}
