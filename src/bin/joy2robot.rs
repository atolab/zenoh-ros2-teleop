extern crate serde;
extern crate serde_json;

use serde::{Serialize, Deserialize};


use zteleop::*;
use std::env;
use futures::prelude::*;
use futures::select;
use std::convert::TryInto;

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
    //let ws = Arc::new(zenoh.workspace(None).await.unwrap());

    //let remote = RemoteControl::new(router).await.unwrap();

    let mut remote = RemoteControl::new(&zenoh).await.unwrap();
    remote.initialize().await.unwrap();


    let mut car : Car = Car{
        max_speed : 0.22,
        max_steer : 2.84,
        curr_speed : 0.0,
        curr_steer : 0.0,
    };

    let mut data_stream = remote.register_stream_listener().await.unwrap();

    loop {
        select!(
            change = data_stream.next().fuse() => {
                let change = change.unwrap();
                match change.value {
                    Some(zv) =>
                    match zv {
                        zenoh::Value::Json(jv) => {
                            let ji = JoypadData::deserialize(&jv);
                            println!("Data from Zenoh Joypad: {:?}", ji);
                            car.curr_speed = ji.r_trigger * car.max_speed;
                            car.curr_steer = ji.l_stick_x * car.max_steer;

                            let cc = CarControl{
                                control_linear_velocity : car.curr_speed,
                                control_angular_velocity : car.curr_steer,
                            };

                            remote.send_car_control(&cc).await.unwrap();
                        }
                        _ => ()
                    }
                    None => ()
                }
            }
        );
    }

    data_stream.close().await.unwrap();
    zenoh.close().await.unwrap();
}
