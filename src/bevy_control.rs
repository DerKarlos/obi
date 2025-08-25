/*
 * ToDo: touch?
 *
    This code was done, starting with https://github.com/sburris0/bevy_flycam:
    To welcome our users, we offer a camera control as it is used with OpenStreetMap 3D rendering
    The first was www.f4map.com . We offer the same key, mouse and wheel
    But we extend it with more keys for all mouse moves too

    Overview:

    'KeyA', 'ArrowLeft', // cursor keys
    'KeyD', 'ArrowRight',
    'KeyW', 'ArrowUp',
    'KeyS', 'ArrowDown',

    'KeyQ', 'KeyE', // rotate
    'KeyR', 'KeyF', // nick
    'KeyG', 'KeyT', // elevate

    'KeyY', 'KeyH', // zoom (Y=Z at German keyboard)

    'PageUp', 'PageDown',
    'Backslash', 'BracketRight', // Left of "Enter"; UK or US keyboard: ] and \ German keypbard: + and #

    'OSLeft', 'OSRight',
    'metaKey',  // Chrome OSkey
    'shiftKey', // 'ShiftLeft', 'ShiftRight',

    'digit0' // reset building positin


See also: https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html

TODO: "like F4" reaction while elevate/distance+/-!
The UI of www.F4map.com is very simple:
Arrows left/right: rotate counter-/clockwise
Arrows up/down: tile/shift forward/backward
1st mouse: tile/shilft
2nd mouse: rotate
Mouse wheel: zoom
Touch: to check

*/

//use bevy::ecs::event::Events;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Key/Mouse/Touch sensitivity - and system runtime data
#[derive(Resource)]
pub struct ControlValues {
    pub use_first_point_for_rotation: bool,
    pub key_rad_per_sec: f32,
    pub mouse_and_touch_rad_per_pixel: f32,
    pub key_meter_per_sec: f32,
    pub mouse_meter_per_pixel: f32,
    pub touch_meter_per_pixel: f32,
    pub touch_pinch_fact_per_pixel: f32,
    pub touch_twist_rad_per_pixel: f32,
    pub target: Vec3,
    pub distance: f32,
    pub touch_count: i8,
    pub last_position_distance: f32,
    pub last_angle: f32,
    pub first_finger_id: u64,
}

impl Default for ControlValues {
    fn default() -> Self {
        Self {
            // default is like F4: first key for panning
            use_first_point_for_rotation: false,

            // Rad per second or pixel
            key_rad_per_sec: 1.0,
            #[cfg(target_arch = "wasm32")]
            mouse_and_touch_rad_per_pixel: 0.0002,
            #[cfg(not(target_arch = "wasm32"))]
            mouse_and_touch_rad_per_pixel: 0.0005,

            // Meter per second or pixel on ground (gets faster if elevated)
            key_meter_per_sec: 50.,
            mouse_meter_per_pixel: 0.003,
            touch_meter_per_pixel: 1.5,
            touch_pinch_fact_per_pixel: 0.005,
            touch_twist_rad_per_pixel: 10.,

            // System runtime data
            target: Vec3::ZERO,
            distance: 500.0,
            touch_count: 0,
            last_position_distance: 0.,
            last_angle: 0.,
            first_finger_id: 0,
        }
    }
}

/// Key configuration
#[derive(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_forward2: KeyCode,
    pub move_backward: KeyCode,
    pub move_backward2: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_ascend: KeyCode,
    pub move_ascend2: KeyCode,
    pub move_descend: KeyCode,
    pub move_descend2: KeyCode,
    //
    pub rotate_up: KeyCode,
    pub rotate_down: KeyCode,
    pub rotate_left: KeyCode,
    pub rotate_left2: KeyCode,
    pub rotate_right: KeyCode,
    pub rotate_right2: KeyCode,
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub zoom_out2: KeyCode,
    //
    pub reset: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::KeyW, // F4
            move_forward2: KeyCode::ArrowUp,
            move_backward: KeyCode::KeyS, // F4
            move_backward2: KeyCode::ArrowDown,
            move_left: KeyCode::KeyA,  // F4
            move_right: KeyCode::KeyD, // F4
            move_ascend: KeyCode::KeyT,
            move_ascend2: KeyCode::BracketRight, // # on German Mac
            move_descend2: KeyCode::Backslash,   // + on German Mac
            move_descend: KeyCode::KeyG,
            //
            rotate_down: KeyCode::KeyF,
            rotate_up: KeyCode::KeyR,
            rotate_left: KeyCode::KeyQ,
            rotate_left2: KeyCode::ArrowLeft,
            rotate_right: KeyCode::KeyE,
            rotate_right2: KeyCode::ArrowRight,
            //
            zoom_in: KeyCode::KeyH,
            zoom_out: KeyCode::KeyZ,
            zoom_out2: KeyCode::KeyY, // Z on German keyboards
            //
            reset: KeyCode::Digit0,
        }
    }
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_camera(mut commands: Commands, control_values: ResMut<ControlValues>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(
            -0.2 * control_values.distance,
            0.3 * control_values.distance,
            1.0 * control_values.distance,
        )
        .looking_at(Vec3::new(0., 0.17 * control_values.distance, 0.), Vec3::Y),
    ));
}

/// Handles keyboard input and movement
fn camera_keys(
    mut camera: Single<&mut Transform, With<Camera>>,
    mut control_values: ResMut<ControlValues>,
    key_bindings: Res<KeyBindings>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut velocity = Vec3::ZERO;
    let elevation_fakt: f32 = 1. + time.delta_secs() * 2.0;
    let rotation_fact = time.delta_secs() * control_values.key_rad_per_sec; // delta rad = delta time * 1.0 (rad per second)

    let local_z = camera.local_z();
    let forward = -Vec3::new(local_z.x, 0., local_z.z);
    let right = Vec3::new(local_z.z, 0., -local_z.x);
    let upward = Vec3::new(0., 1., 0.);
    let (mut yaw_direction, mut pitch_up, _roll) = camera.rotation.to_euler(EulerRot::YXZ);

    for key in keys.get_pressed() {
        let key = *key;
        //
        // forward/backward
        if key == key_bindings.move_forward || key == key_bindings.move_forward2 {
            velocity += forward;
        } else if key == key_bindings.move_backward || key == key_bindings.move_backward2 {
            velocity -= forward;
        //
        // sidewise
        } else if key == key_bindings.move_right {
            velocity += right;
        } else if key == key_bindings.move_left {
            velocity -= right;
        //
        // elevate
        } else if key == key_bindings.move_ascend || key == key_bindings.move_ascend2 {
            velocity += upward;
            //control_values.view.elevation *= elevation_fakt;
        } else if key == key_bindings.move_descend || key == key_bindings.move_descend2 {
            velocity -= upward;

        //
        // rotate
        } else if key == key_bindings.rotate_right || key == key_bindings.rotate_right2 {
            yaw_direction -= rotation_fact;
        } else if key == key_bindings.rotate_left || key == key_bindings.rotate_left2 {
            yaw_direction += rotation_fact;
        } else if key == key_bindings.rotate_up {
            pitch_up += rotation_fact;
        } else if key == key_bindings.rotate_down {
            pitch_up -= rotation_fact;
        //
        // zoom
        } else if key == key_bindings.zoom_out || key == key_bindings.zoom_out2 {
            control_values.distance /= elevation_fakt;
        } else if key == key_bindings.zoom_in {
            control_values.distance *= elevation_fakt;
        } else if key == key_bindings.reset {
            control_values.target = Vec3::ZERO;
        }
    }

    velocity = velocity.normalize_or_zero();

    let key_meter_per_sec = control_values.key_meter_per_sec;
    let height_fact = camera.translation.y.max(3.) / 20.;
    // info!("{height_fact} {}", camera.translation.y);
    control_values.target += velocity * time.delta_secs() * key_meter_per_sec * height_fact;

    const CLAMP_LIMIT: f32 = 88.0_f32.to_radians();
    pitch_up = pitch_up.clamp(-CLAMP_LIMIT, CLAMP_LIMIT);

    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw_direction, pitch_up, 0.);

    camera.translation = control_values.target - camera.forward() * control_values.distance;
}

/// Handles looking around and target shift if screeb us touched
/// https://bevy-cheatbook.github.io/input/gesture.html
fn camera_touch(
    mut camera: Single<&mut Transform, With<Camera>>,
    mut control_values: ResMut<ControlValues>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    touches: Res<Touches>,
    mut _touch_events: EventReader<TouchInput>,
) {
    if let Ok(window) = primary_window.single() {
        let (touch_count_rotate, touch_count_translate) =
            if control_values.use_first_point_for_rotation {
                (
                    1, // rotate: One touch
                    2, // translate: Two touch
                )
            } else {
                (
                    2, // rotate: Two touch
                    1, // translate: One touch
                )
            };

        let window_scale = window.height().min(window.width());

        let local_z = camera.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);
        let upward = Vec3::new(0., 1., 0.);
        let touch_meter_per_pixel = control_values.touch_meter_per_pixel;
        let mut velocity = Vec3::ZERO;

        let (mut yaw_direction, mut pitch_up, _roll) = camera.rotation.to_euler(EulerRot::YXZ);

        for finger in touches.iter_just_pressed() {
            //info!("A new touch with ID {} just began.", finger.id());
            control_values.touch_count += 1;
            if control_values.touch_count == 1 {
                control_values.first_finger_id = finger.id();
            }
        }
        for _finger in touches.iter_just_released() {
            //info!("Touch with ID {} just ended.", _finger.id());
            control_values.touch_count -= 1;
            control_values.last_position_distance = 0.;
            control_values.last_angle = 0.;
        }
        for _finger in touches.iter_just_canceled() {
            //info!("Touch with ID {} was canceled.", _finger.id());
            control_values.touch_count -= 1;
        }

        if control_values.touch_count < 0 {
            control_values.touch_count = 0
        }

        let mut delta_first_finger: Option<Vec2> = None;
        let mut position_first_finger: Vec2 = Vec2::ZERO;
        for finger in touches.iter() {
            let mut finger_delta = finger.delta();
            // iPad finger produces this 1. as diter
            if finger_delta.x.abs() <= 1. {
                finger_delta.x = 0.;
            };
            if finger_delta.y.abs() <= 1. {
                finger_delta.y = 0.;
            };

            // shift-slide-move
            if control_values.touch_count == touch_count_translate {
                velocity += forward * finger_delta.y * window_scale;
                velocity -= right * finger_delta.x * window_scale;
            }

            // rotate
            if control_values.touch_count == touch_count_rotate {
                pitch_up -=
                    (control_values.mouse_and_touch_rad_per_pixel * finger_delta.y * window_scale)
                        .to_radians();
                yaw_direction -=
                    (control_values.mouse_and_touch_rad_per_pixel * finger_delta.x * window_scale)
                        .to_radians();
            }

            // pinch-zoom and twist (always with 2 fingers)
            if control_values.touch_count == 2 {
                if delta_first_finger.is_none() {
                    delta_first_finger = Some(finger_delta);
                    position_first_finger = finger.position();
                } else {
                    // 2nd finger touch event
                    let positon_distance = finger.position().distance(position_first_finger).abs();
                    let angle = if finger.id() == control_values.first_finger_id {
                        finger.position().angle_to(position_first_finger)
                    } else {
                        position_first_finger.angle_to(finger.position())
                    };
                    if control_values.last_position_distance == 0. {
                        control_values.last_position_distance = positon_distance;
                    }
                    if control_values.last_angle == 0. {
                        control_values.last_angle = angle;
                    }
                    let pinch = control_values.last_position_distance - positon_distance;

                    // Only in "F4 mode" ttt test off
                    if touch_count_rotate == -2 {
                        let twist = control_values.last_angle - angle;
                        info!(
                            "angle: {} last: {} twist: {}",
                            angle.to_degrees(),
                            control_values.last_angle.to_degrees(),
                            twist.to_degrees()
                        );
                        yaw_direction += twist * control_values.touch_twist_rad_per_pixel;
                    }

                    control_values.last_position_distance = positon_distance;
                    control_values.last_angle = angle;
                    let elevation_fakt: f32 =
                        1. + pinch * control_values.touch_pinch_fact_per_pixel;
                    control_values.distance *= elevation_fakt;
                    //info!(
                    //    "touches: {} finger_delta: {:?} pinch: {pinch} distance: {positon_distance}",
                    //    control_values.touch_count, finger_delta
                    //);
                }
            }

            if control_values.touch_count == 3 {
                // elevate
                velocity += upward * finger_delta.y * window_scale;
                velocity -= right * finger_delta.x * window_scale;
            }
        }

        if control_values.touch_count > 0 {
            pitch_up = pitch_up.clamp(-1.54, 1.54);
            // Order is important to prevent unintended roll (but there is a better function)
            camera.rotation = Quat::from_axis_angle(Vec3::Y, yaw_direction)
                * Quat::from_axis_angle(Vec3::X, pitch_up);

            velocity = velocity.normalize_or_zero();
            let height_fact = camera.translation.y.max(3.) / 70.;
            control_values.target += velocity * touch_meter_per_pixel * height_fact;
            camera.translation = control_values.target - camera.forward() * control_values.distance;
        }
    }
}

/// Handles looking around and target shift if mouse key is pressed
fn camera_mouse(
    mut camera: Single<&mut Transform, With<Camera>>,
    mut control_values: ResMut<ControlValues>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_state: EventReader<MouseMotion>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = primary_window.single() {
        let mouse_key_rotate = if control_values.use_first_point_for_rotation {
            MouseButton::Left // First
        } else {
            MouseButton::Right // Second
        };
        let mouse_key_translate = if control_values.use_first_point_for_rotation {
            MouseButton::Right // Second
        } else {
            MouseButton::Left // First
        };

        // mouse_wheel: EventReader { reader: Local(EventCursor { last_event_count: 0, _marker: PhantomData<bevy_input::mouse::MouseWheel> }), events: Res(Events { events_a: EventSequence { events: [], start_event_count: 2 }, events_b: EventSequence { events: [], start_event_count: 2 }, event_count: 2 }) }
        for event in mouse_wheel.read() {
            //info!("mouse_wheel event: {:?} ", event);
            // mouse_wheel event: MouseWheel { unit: Pixel, x: 0.0, y: 0.0, window: 0v1#4294967296 }
            let elevation_fakt: f32 = 1. + event.y * 0.0005;
            control_values.distance /= elevation_fakt;
            control_values.distance = control_values.distance.max(0.3);
            camera.translation = control_values.target - camera.forward() * control_values.distance;
        }

        for event in mouse_state.read() {
            // Using smallest of height or width ensures equal vertical and horizontal sensitivity
            let window_scale = window.height().min(window.width());

            if mouse_buttons.pressed(mouse_key_translate) {
                let local_z = camera.local_z();
                let forward = -Vec3::new(local_z.x, 0., local_z.z);
                let right = Vec3::new(local_z.z, 0., -local_z.x);
                let mut velocity = Vec3::ZERO;
                let mouse_meter_per_pixel = control_values.mouse_meter_per_pixel;
                velocity += forward * event.delta.y;
                velocity -= right * event.delta.x;
                let height_fact = camera.translation.y.max(3.) / 70.;
                control_values.target +=
                    velocity * mouse_meter_per_pixel * height_fact * window_scale;
            }

            if mouse_buttons.pressed(mouse_key_rotate) {
                let (mut yaw_direction, mut pitch_up, _roll) =
                    camera.rotation.to_euler(EulerRot::YXZ);

                {
                    pitch_up -= (control_values.mouse_and_touch_rad_per_pixel
                        * event.delta.y
                        * window_scale)
                        .to_radians();
                    yaw_direction -= (control_values.mouse_and_touch_rad_per_pixel
                        * event.delta.x
                        * window_scale)
                        .to_radians();
                }

                pitch_up = pitch_up.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                camera.rotation = Quat::from_axis_angle(Vec3::Y, yaw_direction)
                    * Quat::from_axis_angle(Vec3::X, pitch_up);

                camera.translation =
                    control_values.target - camera.forward() * control_values.distance;

                // https://bevy.org/examples/camera/camera-orbit/
            }
        }
    }
}

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct ControlWithCamera;
impl Plugin for ControlWithCamera {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindings>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, camera_keys)
            .add_systems(Update, camera_mouse)
            .add_systems(Update, camera_touch);
    }
}

/// Same as [`CameraPlugin`] but does not spawn a camera
pub struct ControlNoCamera;
impl Plugin for ControlNoCamera {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindings>()
            .add_systems(Update, camera_keys)
            .add_systems(Update, camera_mouse)
            .add_systems(Update, camera_touch);
    }
}
