use std::{thread, time};

use rdev::{grab, listen, simulate, Button, Event, EventType, Key};

fn main() {
    println!("Hello, worl");
    let callback = |event: Event| -> Option<Event> {
        if let EventType::MouseMove { x, y } = event.event_type {
            println!("Consuming and cancelling CapsLock");
            None // CapsLock is now effectively disabled
        } else {
            Some(event)
        }
    };
    // send(&EventType::KeyPress(Key::KeyS));
    // send(&EventType::KeyRelease(Key::KeyS));
    //
    // send(&EventType::MouseMove { x: 0.0, y: 0.0 });
    // send(&EventType::MouseMove { x: 400.0, y: 400.0 });
    // send(&EventType::ButtonPress(Button::Left));
    // send(&EventType::ButtonRelease(Button::Right));
    // send(&EventType::Wheel {
    //     delta_x: 0,
    //     delta_y: 1,
    // });

    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error);
    }
}

fn callback(event: Event) {
    println!("My callback {:?}", event.event_type);
}
// //
// // fn send(event_type: &EventType) {
// //     let delay = time::Duration::from_millis(20);
// //     match simulate(event_type) {
//         Ok(()) => (),
//         Err(SimulateError) => {
//             println!("We could not send {:?}", event_type);
//         }
//     }
//     // Let ths OS catchup (at least MacOS)
//     thread::sleep(delay);
// }
