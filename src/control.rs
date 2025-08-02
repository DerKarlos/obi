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

    'KeyY', 'KeyH', // zoom (Y=Z at German keyboard) Mind the Compas! ???

    'PageUp', 'PageDown',
    'Backslash', 'BracketRight', // Left of "Enter"; UK or US keyboard: ] and \ German keypbard: + and #

    'OSLeft', 'OSRight',
    'metaKey',  // Chrome OSkey
    'shiftKey', // 'ShiftLeft', 'ShiftRight',

    'digit0' // reset


We start with an argumente to select one control and later switch dynamically.
All controls will have the resource type control later (now Control)
Maximal one control/plurgin/systems should run (may be none)

See also: https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html

TODO: "like F4" reaction while elevate/distance+/-!
The UI of www.F4map.com is very simple:
Arrows left/right: rotate counter-/clockwise
Arrows up/down: tile/shift forward/backward
1st mouse: tile/shilft
2nd mouse: rotate
Mouse wheel: zoom

*/

//use bevy::ecs::event::Events;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
//use winit::event::Touch;

/// Mouse sensitivity and movement speed
#[derive(Resource)]
pub struct ControlValues {
    pub use_first_mouse_key_for_orientation: bool,
    pub sensitivity: f32,
    pub speed: f32,
    pub target: Vec3,
    pub distance: f32,
    pub touch_count: i8,
    pub last_position_delta: f32,
    pub last_angle: f32,
    pub first_finger_id: u64,
}

impl Default for ControlValues {
    fn default() -> Self {
        Self {
            // default is like F4: first key for panning
            use_first_mouse_key_for_orientation: false,
            sensitivity: 0.0005,
            speed: 50.,
            target: Vec3::ZERO,
            distance: 500.0,
            touch_count: 0,
            last_position_delta: 0.,
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
            zoom_in: KeyCode::KeyH,
            zoom_out: KeyCode::KeyZ,
            zoom_out2: KeyCode::KeyY, // Z on german Keyboards
            //
            reset: KeyCode::Digit0,
        }
    }
}

/// Used in queries when you want flycams and not other cameras
/// A marker component used in queries when you want the OTB cammera and not other cameras
#[derive(Component)]
pub struct F4plusCam;

/// Spawns the `Camera3dBundle` to be controlled
fn setup(mut commands: Commands, control_values: ResMut<ControlValues>) {
    commands.spawn((
        Camera3d::default(),
        F4plusCam,
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
    let rotation_fact = time.delta_secs(); // delta rad = delta time * 1.0 (rad per second)

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

    let speed = control_values.speed;
    let height_fact = camera.translation.y.max(3.) / 20.;
    // info!("{height_fact} {}", camera.translation.y);
    control_values.target += velocity * time.delta_secs() * speed * height_fact;

    pitch_up = pitch_up.clamp(-1.54, 1.54);

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
        let window_scale = window.height().min(window.width());

        let local_z = camera.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);
        let upward = Vec3::new(0., 1., 0.);
        let speed = control_values.speed / 15.;
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
            control_values.last_position_delta = 0.;
            control_values.last_angle = 0.;
        }
        for _finger in touches.iter_just_canceled() {
            //info!("Touch with ID {} was canceled.", _finger.id());
            control_values.touch_count -= 1;
        }

        if control_values.touch_count < 0 {
            control_values.touch_count = 0
        }

        let mut delta1: Option<Vec2> = None;
        let mut position1: Vec2 = Vec2::ZERO;
        for finger in touches.iter() {
            let delta = finger.delta();

            if control_values.touch_count == 1 {
                velocity += forward * delta.y * window_scale;
                velocity -= right * delta.x * window_scale;
            }

            if control_values.touch_count == 2 {
                pitch_up -= (control_values.sensitivity * delta.y * window_scale).to_radians();
                yaw_direction -= (control_values.sensitivity * delta.x * window_scale).to_radians();

                if delta1.is_some() {
                    let positon_delta = finger.position().distance(position1).abs();
                    let angle = if finger.id() == control_values.first_finger_id {
                        finger.position().angle_to(position1)
                    } else {
                        position1.angle_to(finger.position())
                    };
                    if control_values.last_position_delta == 0. {
                        control_values.last_position_delta = positon_delta;
                    }
                    if control_values.last_angle == 0. {
                        control_values.last_angle = angle;
                    }
                    let pinch = control_values.last_position_delta - positon_delta;
                    let twist = control_values.last_angle - angle;
                    info!(
                        "angle: {} last: {} twist: {}",
                        angle.to_degrees(),
                        control_values.last_angle.to_degrees(),
                        twist.to_degrees()
                    );
                    // yaw_direction += twist * 10.0;

                    control_values.last_position_delta = positon_delta;
                    control_values.last_angle = angle;
                    let elevation_fakt: f32 = 1. + pinch * 0.002;
                    control_values.distance *= elevation_fakt;
                    //info!(
                    //    "touch count: {} delta: {:?} pinch: {pinch} delta: {positon_delta}",
                    //    control_values.touch_count, delta
                    //);
                } else {
                    delta1 = Some(delta);
                    position1 = finger.position();
                }
            }

            if control_values.touch_count == 3 {
                velocity += upward * delta.y * window_scale;
                velocity -= right * delta.x * window_scale;
            }
        }
        if control_values.touch_count == 1 || control_values.touch_count == 3 {
            velocity = velocity.normalize_or_zero();
            control_values.target += velocity * speed;
        }
        if control_values.touch_count == 2 {
            pitch_up = pitch_up.clamp(-1.54, 1.54);
            // Order is important to prevent unintended roll
            camera.rotation = Quat::from_axis_angle(Vec3::Y, yaw_direction)
                * Quat::from_axis_angle(Vec3::X, pitch_up);
        }
        if control_values.touch_count > 0 {
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
    time: Res<Time>,
) {
    if let Ok(window) = primary_window.single() {
        let mouse_key_rotate = if control_values.use_first_mouse_key_for_orientation {
            MouseButton::Left // First
        } else {
            MouseButton::Right // Second
        };
        let mouse_key_translate = if control_values.use_first_mouse_key_for_orientation {
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
                let speed = control_values.speed / 500.;
                velocity +=
                    forward * (control_values.speed * event.delta.y * window_scale).to_radians();
                velocity -=
                    right * (control_values.speed * event.delta.x * window_scale).to_radians();
                let height_fact = camera.translation.y.max(3.) / 70.;
                control_values.target += velocity * time.delta_secs() * speed * height_fact;
            }

            if mouse_buttons.pressed(mouse_key_rotate) {
                let (mut yaw_direction, mut pitch_up, _roll) =
                    camera.rotation.to_euler(EulerRot::YXZ);

                {
                    pitch_up -=
                        (control_values.sensitivity * event.delta.y * window_scale).to_radians(); // tadiants ???
                    yaw_direction -=
                        (control_values.sensitivity * event.delta.x * window_scale).to_radians();
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
            .add_systems(Startup, setup)
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

// Gets not visible ??? ?? ?
fn _instructions(mut commands: Commands) {
    commands.spawn((
        Name::new("Instructions"),
        Text::new(
            "Link? <a href=\"https://osmgo.org\">Ttest</a>\n\
            Mouse up or down: pitch\n\
            Mouse left or right: yaw\n\
            Mouse buttons: roll",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(92.),
            left: Val::Px(92.),
            ..default()
        },
    ));
}
