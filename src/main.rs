#[macro_use]
extern crate log;
extern crate env_logger;

use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender};


struct SendBroker {
    map: Arc<Mutex<HashMap<u16, Sender<Call>>>>,
}

impl Clone for SendBroker {
    fn clone(&self) -> Self {
        SendBroker {
            map: self.map.clone(),
        }
    }
}

impl SendBroker {
    fn new() -> Self {
        SendBroker {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn reg_sender(&self, id: u16, sender: Sender<Call>) -> Option<Sender<Call>> {
        let mut map = self.map.lock().expect("need lock to register sender");
        (*map).insert(id, sender)
    }

    fn unreg_sender(&self, id: u16) -> Option<Sender<Call>> {
        let mut map = self.map.lock().expect("need lock to unregister sender");
        (*map).remove(&id)
    }

    fn find_sender(&self, id: u16) -> Option<Sender<Call>> {
        let map = self.map.lock().expect("need lock to find sender");
        (*map).get(&id).cloned()
    }
}


struct Call {
    value: u16,
    listener: Sender<Answer>,
}

struct Answer {
    value: u16,
}

// Counters
const LOCAL: u16 = 3;
const REG_ID: u16 = 1;

fn main() {
    env_logger::init().unwrap();
    let broker = SendBroker::new();
    for a in 0.. {
        let finder = broker.clone();
        let rbroker = broker.clone();
        debug!("- - - - - - - - - - - - - - - - - - - -");
        debug!(">>>>> {} >>>>>", a);
        let shandler = thread::spawn(move || {
            let registrar = rbroker.clone();
            let rhandler = thread::spawn(move || {
                let (tx, rx) = channel();
                registrar.reg_sender(REG_ID, tx);
                let mut optrx = Some(rx);
                let mut counter = 0;
                loop {
                    let rx = optrx.take().expect("rx unwrap");
                    match rx.recv_timeout(Duration::from_millis(1)) {
                        Ok(Call { listener, value }) => {
                            debug!("{} - request -> {}", a, value);
                            assert_eq!(value, counter);
                            listener.send(Answer { value }).expect("sending response");
                            counter += 1;
                            if counter >= LOCAL {
                                break;
                            }
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
                    if let Some(service) = finder.find_sender(REG_ID) {
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
