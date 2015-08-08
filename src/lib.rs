
#[macro_use]
extern crate maplit;
#[macro_use]
mod macros;

pub use action::Action;
pub use conditions::Conditions;
pub use context::Context;
pub use correlator::Correlator;
pub use dispatcher::Dispatcher;
pub use message::Message;
pub use timer::{Timer,
                TimerEvent};

pub mod conditions;
pub mod config;
pub mod action;
mod context;
mod correlator;
mod dispatcher;
mod message;
mod state;
mod timer;

pub type MiliSec = u32;

#[derive(Debug)]
pub enum Event {
    Timer(TimerEvent),
    Message(Message)
}

#[derive(Debug)]
pub enum Command {
    Dispatch(Event),
    Exit
}
