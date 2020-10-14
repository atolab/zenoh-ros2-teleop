use gilrs::{Gilrs, Button, Event, EventType,  Axis, PowerInfo};

use zteleop::*;
use std::env;

use zenoh::*;
use async_std::sync::Arc;

#[async_std::main]
async fn main() {

    let args: Vec<String> = env::args().collect();
    let router = &args[1];


    let zenoh = Zenoh::new(config::client(Some(router.to_string()))).await.unwrap();
    //let zenoh = Arc::new(Zenoh::new(config::client(Some(router.to_string()))).await.unwrap());
    //let ws = Arc::new(zenoh.workspace(None).await.unwrap());


    // let mut remote = RemoteControl::new(router).await.unwrap();

    let mut remote = RemoteControl::new(&zenoh).await.unwrap();
    remote.initialize().await.unwrap();


    let mut gilrs = Gilrs::new().unwrap();

    let mut joypad_info = JoypadData::new();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;

    loop {
        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {

            active_gamepad = Some(id);
            match event {

                EventType::ButtonChanged(button, value, code) => {
                    match button {
                        Button::RightTrigger2 =>  joypad_info.r_trigger = value,
                        Button::LeftTrigger2 =>  joypad_info.l_trigger = value,
                        _ => (),
                    }
                },
                EventType::AxisChanged(axis, value, code) => {
                    match axis {
                        Axis::LeftStickX => joypad_info.l_stick_x = value,
                        Axis::LeftStickY => joypad_info.l_stick_y = value,
                        _ => (),
                    }
                },
                _ => (),
            }

            if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
                match gamepad.power_info() {
                    PowerInfo::Charged => joypad_info.battery = 100,
                    PowerInfo::Charging(l) => joypad_info.battery = l,
                    PowerInfo::Discharging(l) => joypad_info.battery = l,
                    PowerInfo::Wired => joypad_info.battery = 100,
                    PowerInfo::Unknown => joypad_info.battery = 0,
                }
            }

            // println!("Joypad Info {:?}", &joypad_info);
            remote.send_command(&joypad_info).await.unwrap();

        }

    }
}

