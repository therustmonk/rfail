#[macro_use]
extern crate log;

mod broker;

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender};
use broker::{SendBroker, Registrar, Finder};

struct Call {
    value: u8,
    listener: Sender<Answer>,
}

struct Answer {
    value: u8,
}

fn main() {
    let broker: SendBroker<u8, Call> = SendBroker::new();
    for a in 1..10000 {
        let finder = broker.clone();
        let rbroker = broker.clone();
        let shandler = thread::spawn(move || {
            let registrar = rbroker.clone();
            let rhandler = thread::spawn(move || {
                let (tx, rx) = channel();
                registrar.reg_sender(1, tx).expect("register to broker");
                let mut optrx = Some(rx);
                for x in 1..3 {
                    let rx = optrx.take().expect("rx unwrap");
                    match rx.recv_timeout(Duration::from_millis(1)) {
                        Ok(Call { listener, value }) => {
                            assert_eq!(value, x);
                            listener.send(Answer { value }).expect("sending response");
                        },
                        Err(_) => {
                        },
                    }
                    optrx = Some(rx);
                }
                registrar.unreg_sender(1).expect("remove from broker");
            });
            for x in 1..3 {
                loop {
                    if let Some(service) = finder.find_sender(1).expect("finding sender") {
                        let (tx, rx) = channel();
                        service.send(Call { listener: tx, value: x }).expect("sending request");
                        let Answer { value } = rx.recv().expect("receive result");
                        assert_eq!(value, x);
                        break;
                    }
                }
            }
            rhandler.join().unwrap();
        });
        shandler.join().unwrap();
        trace!("Iter: {}", a);
        println!("Done!");
    }
}
