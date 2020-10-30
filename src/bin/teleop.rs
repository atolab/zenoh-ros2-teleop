extern crate serde;
extern crate serde_json;

use serde::{Serialize, Deserialize};


use gilrs::{Gilrs, Button, Event, EventType,  Axis, PowerInfo};
use zteleop::*;
use std::env;

use zenoh::*;
use async_std::sync::Arc;

#[derive(Debug)]
struct Car {
    pub max_speed : f32,
    pub max_steer: f32,
    pub curr_speed : f32,
    pub curr_steer: f32

}



#[async_std::main]
async fn main() {

    let args: Vec<String> = env::args().collect();
    let router = &args[1];


    let zenoh = Zenoh::new(net::config::client(Some(router.to_string()))).await.unwrap();



    let mut car : Car = Car{
        max_speed : 0.22,
        max_steer : 2.84,
        curr_speed : 0.0,
        curr_steer : 0.0,
    };

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

            //println!("Joypad Info {:?}", &joypad_info);
            //remote.send_command(&joypad_info).await.unwrap();
            if joypad_info.r_trigger > 0.0 {
                car.curr_speed = joypad_info.r_trigger * car.max_speed;
            } else if joypad_info.l_trigger > 0.0 {
                car.curr_speed = -joypad_info.l_trigger * car.max_speed;
            } else {
                car.curr_speed = 0.0;
            }

            car.curr_steer = joypad_info.l_stick_x * car.max_steer;

            let cc = CarControl{
                control_linear_velocity : car.curr_speed,
                control_angular_velocity : car.curr_steer,
            };
            remote.send_car_control(&cc).await.unwrap();

        }

    }
}

