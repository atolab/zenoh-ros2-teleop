extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use zenoh::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JoypadData {
    pub r_trigger: f32,
    pub l_trigger: f32,
    pub l_stick_x: f32,
    pub l_stick_y: f32,
    pub battery: u8,
}

impl JoypadData {
    pub fn new() -> JoypadData {
        JoypadData {
            r_trigger: 0.0,
            l_trigger: 0.0,
            l_stick_x: 0.0,
            l_stick_y: 0.0,
            battery: 100,
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn deserialize(data: &String) -> JoypadData {
        serde_json::from_str::<JoypadData>(data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CarControl {
    pub control_linear_velocity: f32,
    pub control_angular_velocity: f32,
}

impl CarControl {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn deserialize(data: &String) -> CarControl {
        serde_json::from_str::<CarControl>(data).unwrap()
    }
}

// pub struct RemoteControl<'a> {
//     zenoh : Arc<zenoh::Zenoh>,
//     workspace : Arc<zenoh::Workspace<'a>>,
// }

pub struct RemoteControl<'a> {
    // zenoh : Arc<zenoh::Zenoh>,
    //workspace : Arc<zenoh::Workspace<'a>>,
    zenoh: &'a zenoh::Zenoh,
    workspace: Option<zenoh::Workspace<'a>>,
}

impl<'a> RemoteControl<'a> {
    // pub async fn new(locator : &String) -> Result<RemoteControl<'a>, ZError> {
    //     let zconfig = config::client(Some(locator.to_string()));
    //     let z = Arc::new(Zenoh::new(zconfig).await?);
    //     Ok(RemoteControl{
    //         zenoh : z,
    //         //inner : RwLock::new(None),
    //         inner : None,
    //     })
    // }

    pub async fn new(zenoh: &'a zenoh::Zenoh) -> Result<RemoteControl<'a>, ZError> {
        Ok(RemoteControl {
            zenoh: zenoh,
            //inner : RwLock::new(None),
            workspace: None,
        })
    }

    pub async fn initialize(&mut self) -> Result<(), ZError> {
        let ws = self.zenoh.workspace(None).await?;
        self.workspace = Some(ws);
        Ok(())
    }

    // pub async fn new(ws: Arc<zenoh::Workspace<'a>>) -> Result<RemoteControl<'a>, ZError> {
    //     Ok(RemoteControl{
    //         workspace : ws,
    //     })
    // }

    pub async fn send_command(&'a self, info: &JoypadData) -> Result<(), ZError> {
        let path = zenoh::Path::try_from(format!("/robot/command"))?;
        let v = zenoh::Value::Json(info.serialize());
        // Ok(self.inner.read().await.as_ref().unwrap().ws.put(&path, v).await?)
        Ok(self.workspace.as_ref().unwrap().put(&path, v).await?)
    }

    pub async fn send_car_control(&'a self, info: &CarControl) -> Result<(), ZError> {
        let path = zenoh::Path::try_from(format!("/turtlebot/move"))?;
        let v = zenoh::Value::Json(info.serialize());
        // Ok(self.inner.read().await.as_ref().unwrap().ws.put(&path, v).await?)
        Ok(self.workspace.as_ref().unwrap().put(&path, v).await?)
    }

    pub async fn register_listener<RemoteControlListener>(
        &'a self,
        mut cb: RemoteControlListener,
    ) -> Result<(), ZError>
    where
        RemoteControlListener: FnMut(JoypadData) + Send + Sync + 'static,
    {
        let f = move |r: Change| match r.value {
            Some(zv) => match zv {
                zenoh::Value::Json(jv) => {
                    let ji = JoypadData::deserialize(&jv);
                    cb(ji);
                }
                _ => (),
            },
            None => (),
        };
        let s = zenoh::Selector::try_from(format!("/robot/command"))?;
        // self.inner.read().await.as_ref().unwrap().ws.subscribe_with_callback(&s, f).await?;
        self.workspace
            .as_ref()
            .unwrap()
            .subscribe_with_callback(&s, f)
            .await?;

        Ok(())
    }

    pub async fn register_stream_listener(&'a self) -> Result<zenoh::ChangeStream<'a>, ZError> {
        let s = zenoh::Selector::try_from(format!("/robot/command"))?;
        //let inner  = self.inner.read().await;
        //inner.stream_subscribe(s)
        let sub = self.workspace.as_ref().unwrap().subscribe(&s).await?;

        Ok(sub)
    }
}
