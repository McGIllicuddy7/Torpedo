use raylib::{
    RaylibHandle, RaylibThread,
    ffi::KeyboardKey::{self, KEY_W},
    math::{Quaternion, Vector3},
};

use crate::{
    engine::{
        CameraData, GObject, GameObject, GameObjectData, make_object, random_vector, set_player,
    },
    mesh::GameMesh,
};

pub struct Ship {
    pub data: GameObjectData,
    pub is_ai: bool,
}
impl GameObject for Ship {
    fn on_update(
        &mut self,
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
    ) {
        self.update(handle, thread);
    }
    fn on_event(
        &mut self,
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
        ev: crate::engine::Event,
    ) {
    }
    fn get_data(&self) -> &GameObjectData {
        &self.data
    }
    fn get_data_mut(&mut self) -> &mut GameObjectData {
        &mut self.data
    }
}

impl Ship {
    pub fn update(
        &mut self,
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
    ) {
        let lin_acc_amount = 10.;
        let rot_acc_amount = 20.;
        let input = if self.is_ai {
            self.ai_input(handle, thread)
        } else {
            self.player_input(handle, thread)
        };
        self.data.angular_velocity += input.rotational_acc * rot_acc_amount * 1. / 60.;
        if input.rotational_acc.length() == 0.0 {
            if self.data.angular_velocity.length() < rot_acc_amount / 60. {
                self.data.angular_velocity = Vector3::zero();
            } else {
                self.data.angular_velocity -=
                    self.data.angular_velocity.normalized() * rot_acc_amount / 60.;
            }
        }
        self.data.velocity +=
            input.lin_acc.rotate_by(self.data.rotation.inverted()) * lin_acc_amount * 1. / 60.;
        if self.data.angular_velocity.length() > 1. {
            let n = self.data.angular_velocity.normalized();
            self.data.angular_velocity = n * 1.;
        }
        if self.data.velocity.length() > 3. {
            let n = self.data.velocity.normalized();
            self.data.velocity = n * 3.;
        }
    }

    pub fn ai_input(&self, _handle: &mut RaylibHandle, _thread: &RaylibThread) -> Input {
        Input {
            rotational_acc: random_vector() / 20.0,
            lin_acc: random_vector(),
        }
    }

    pub fn player_input(&self, handle: &mut RaylibHandle, _thread: &RaylibThread) -> Input {
        let mut lin_acc = Vector3::zero();
        let mut racc = Vector3::zero();
        if handle.is_key_down(KeyboardKey::KEY_W) {
            lin_acc.x += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_S) {
            lin_acc.x -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_A) {
            lin_acc.y += 1.
        }
        if handle.is_key_down(KeyboardKey::KEY_D) {
            lin_acc.y -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_Z) {
            lin_acc.z -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_SPACE) {
            lin_acc.z += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_Q) {
            racc.z -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_E) {
            racc.z += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_R) {
            racc.y += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_F) {
            racc.y -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_T) {
            racc.x += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_G) {
            racc.x -= 1.;
        }
        let v = handle.get_mouse_delta() * 5.;
        racc.z += v.x;
        racc.y -= v.y;
        if racc.length() > 0.0 {
            if racc.z > 1. {
                racc.z = 1.;
            }
            if racc.z < -1. {
                racc.z = -1.;
            }
            if racc.y > 1. {
                racc.y = 1.;
            }
            if racc.y < -1. {
                racc.y = -1.;
            }
        }
        Input {
            rotational_acc: racc,
            lin_acc,
        }
    }
}

pub struct Input {
    pub rotational_acc: Vector3,
    pub lin_acc: Vector3,
}

pub fn create_player_ufo(pos: Vector3, rotation: Quaternion) -> GObject {
    let size = 1.;
    _ = rotation;
    let mut msh = GameMesh {
        points: Vec::new(),
        lines: Vec::new(),
    };
    msh.add_cylinder(
        1. * size,
        0.8 * size,
        0.5 * size,
        6,
        Vector3::new(0., 0., -0.25 * size),
        Vector3::new(0.0, 0.0, 1.),
        Vector3::new(1., 0.0, 0.0),
    );
    let v = make_object(Ship {
        is_ai: false,
        data: GameObjectData {
            model: Some(msh),
            location: pos,
            rotation: Quaternion::identity(),
            width: 2. * size,
            depth: 2. * size,
            height: 0.5 * size,
            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            camera_data: Some(CameraData {
                position: Vector3::zero(),
                rotation: Quaternion::identity(),
            }),
            is_projectile: false,
            is_static: false,
        },
    });
    set_player(v);
    v
}
