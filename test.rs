// Copyright 2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ipc::{self, IpcReceiver, IpcSender, IpcServer};

use libc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct PersonAndChannel {
    person: Person,
    sender: IpcSender<Person>,
}

#[test]
fn simple() {
    let person = Person {
        name: "Patrick Walton".to_owned(),
        age: 29,
    };
    let (tx, rx) = ipc::channel().unwrap();
    tx.send(person.clone()).unwrap();
    let received_person = rx.recv().unwrap();
    assert_eq!(person, received_person);
}

#[test]
fn embedded_channels() {
    let person = Person {
        name: "Patrick Walton".to_owned(),
        age: 29,
    };
    let (sub_tx, sub_rx) = ipc::channel().unwrap();
    let person_and_channel = PersonAndChannel {
        person: person.clone(),
        sender: sub_tx,
    };
    let (super_tx, super_rx) = ipc::channel().unwrap();
    super_tx.send(person_and_channel).unwrap();
    let received_person_and_channel = super_rx.recv().unwrap();
    assert_eq!(received_person_and_channel.person, person);
    received_person_and_channel.sender.send(person.clone()).unwrap();
    let received_person = sub_rx.recv().unwrap();
    assert_eq!(received_person, person);
}

#[test]
fn cross_process_embedded_channels() {
    let person = Person {
        name: "Patrick Walton".to_owned(),
        age: 29,
    };
    let (server0, server0_name) = IpcServer::new().unwrap();
    let (server2, server2_name) = IpcServer::new().unwrap();
    unsafe {
        if libc::fork() == 0 {
            let (tx1, rx1): (IpcSender<Person>, IpcReceiver<Person>) = ipc::channel().unwrap();
            let tx0 = IpcSender::connect(server0_name).unwrap();
            tx0.send(tx1).unwrap();
            rx1.recv().unwrap();
            let tx2: IpcSender<Person> = IpcSender::connect(server2_name).unwrap();
            tx2.send(person.clone()).unwrap();
            libc::exit(0);
        }
    }
    let (_, tx1): (_, IpcSender<Person>) = server0.accept().unwrap();
    tx1.send(person.clone()).unwrap();
    let (_, received_person): (_, Person) = server2.accept().unwrap();
    assert_eq!(received_person, person);
}

