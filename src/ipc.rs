use alloc::boxed::Box;
use crate::threads::Thread;
use core::mem;

pub enum Message {
    Short(u64),
    Long,
}

pub enum Rendezvous {
    Empty,
    Sending(Option<Box<Thread>>, Message),
    Receiving(Box<Thread>)
}

impl Rendezvous {
    pub fn send(&mut self, thread: Option<Box<Thread>>, message: Message) -> (Option<Box<Thread>>, Option<Box<Thread>>) {
        match &*self {
            Rendezvous::Empty => {
                *self = Rendezvous::Sending(thread, message);
                (None, None)
            }
            Rendezvous::Sending(_, _) => {
                if let Some(t) = &thread {
                    t.return_error(1);
                }
                (thread, None)
            }
            Rendezvous::Receiving(_) => {
                if let Rendezvous::Receiving(rec_thread) = mem::replace(self, Rendezvous::Empty) {
                    rec_thread.return_message(message);
                    if let Some(ref t) = thread {
                        t.return_error(0);
                    }
                    return (Some(rec_thread), thread);
                }
                (None, None) // Will never be reached
            }
        }
    }

    pub fn receive(&mut self, thread: Box<Thread>) -> (Option<Box<Thread>>, Option<Box<Thread>>) {
        match &*self {
            Rendezvous::Empty => {
                *self = Rendezvous::Receiving(thread);
                (None, None)
            }
            Rendezvous::Sending(_, _) => {
                if let Rendezvous::Sending(snd_thread, message) = mem::replace(self, Rendezvous::Empty) {
                    thread.return_message(message);
                    if let Some(ref t) = snd_thread {
                        t.return_error(0);
                    }
                    return (Some(thread), snd_thread);
                }
                (None, None) // Will never be reached
            }
            Rendezvous::Receiving(_) => {
                thread.return_error(2);
                (Some(thread), None)
            }
        }
    }
}