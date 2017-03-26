#[macro_use]
extern crate log;
extern crate env_logger;

mod broker;

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender};
use broker::{SendBroker, Registrar, Finder};

struct Call {
    value: u16,
    listener: Sender<Answer>,
}

struct Answer {
    value: u16,
}

// Counters
const GLOBAL: u16 = 1000;
const LOCAL: u16 = 3;
const REG_ID: u16 = 1;

fn main() {
    env_logger::init().unwrap();
    let broker: SendBroker<u16, Call> = SendBroker::new();
    for a in 0..GLOBAL {
        let finder = broker.clone();
        let rbroker = broker.clone();
        debug!("- - - - - - - - - - - - - - - - - - - -");
        debug!(">>>>> {} >>>>>", a);
        let shandler = thread::spawn(move || {
            let registrar = rbroker.clone();
            let rhandler = thread::spawn(move || {
                let (tx, rx) = channel();
                registrar.reg_sender(REG_ID, tx).expect("register to broker");
                let mut optrx = Some(rx);
                for x in 0..LOCAL {
                    let rx = optrx.take().expect("rx unwrap");
                    match rx.recv_timeout(Duration::from_millis(1)) {
                        Ok(Call { listener, value }) => {
                            debug!("{} - request -> {}", a, value);
                            assert_eq!(value, x);
                            listener.send(Answer { value }).expect("sending response");
                        },
                        Err(_) => {
                        },
                    }
                    optrx = Some(rx);
                }
                registrar.unreg_sender(REG_ID).expect("remove from broker");
            });
            for x in 0..LOCAL {
                loop {
                    if let Some(service) = finder.find_sender(REG_ID).expect("finding sender") {
                        debug!("{} - found : {}", a, x);
                        let (tx, rx) = channel();
                        service.send(Call { listener: tx, value: x }).expect("sending request");
                        let Answer { value } = rx.recv().expect("receive result");
                        debug!("{} - response <- {}", a, value);
                        assert_eq!(value, x);
                        break;
                    }
                }
            }
            rhandler.join().unwrap();
        });
        shandler.join().unwrap();
        debug!("<<<<< {} <<<<<", a);
        trace!("Iter: {}", a);
    }
    println!("Done!");
}
