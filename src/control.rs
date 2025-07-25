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

/// Mouse sensitivity and movement speed
#[derive(Resource)]
pub struct ControlValues {
    pub sensitivity: f32,
    pub speed: f32,
    pub target: Vec3,
    pub distance: f32,
}

impl Default for ControlValues {
    fn default() -> Self {
        Self {
            sensitivity: 0.0005,
            speed: 50.,
            target: Vec3::ZERO,
            distance: 500.0,
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
/// A marker component used in queries when you want flycams and not other cameras
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
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    key_bindings: Res<KeyBindings>,
    mut control_values: ResMut<ControlValues>,
    mut camera: Single<&mut Transform, With<Camera>>,
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

    let speed = control_values.speed.clone();
    control_values.target += velocity * time.delta_secs() * speed;

    pitch_up = pitch_up.clamp(-1.54, 1.54);

    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw_direction, pitch_up, 0.);

    camera.translation = control_values.target - camera.forward() * control_values.distance;
}

/// Handles looking around and target shift if mouse key is pressed
fn camera_mouse(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_state: EventReader<MouseMotion>,
    mut control_values: ResMut<ControlValues>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    if let Ok(window) = primary_window.single() {
        // mouse_wheel: EventReader { reader: Local(EventCursor { last_event_count: 0, _marker: PhantomData<bevy_input::mouse::MouseWheel> }), events: Res(Events { events_a: EventSequence { events: [], start_event_count: 2 }, events_b: EventSequence { events: [], start_event_count: 2 }, event_count: 2 }) }
        for event in mouse_wheel.read() {
            //println!("mouse_wheel event: {:?} ", event);
            // mouse_wheel event: MouseWheel { unit: Pixel, x: 0.0, y: 0.0, window: 0v1#4294967296 }
            let elevation_fakt: f32 = 1. + event.y * 0.0005;
            control_values.distance /= elevation_fakt;
            camera.translation = control_values.target - camera.forward() * control_values.distance;
        }

        for event in mouse_state.read() {
            // Using smallest of height or width ensures equal vertical and horizontal sensitivity
            let window_scale = window.height().min(window.width());

            if mouse_buttons.pressed(MouseButton::Right) {
                let local_z = camera.local_z();
                let forward = -Vec3::new(local_z.x, 0., local_z.z);
                let right = Vec3::new(local_z.z, 0., -local_z.x);
                let mut velocity = Vec3::ZERO;
                let speed = control_values.speed.clone() / 500.;
                velocity +=
                    forward * (control_values.speed * event.delta.y * window_scale).to_radians();
                velocity -=
                    right * (control_values.speed * event.delta.x * window_scale).to_radians();
                control_values.target += velocity * time.delta_secs() * speed;
            }

            if mouse_buttons.pressed(MouseButton::Left) {
                let (mut yaw_direction, mut pitch_up, _roll) =
                    camera.rotation.to_euler(EulerRot::YXZ);

                {
                    pitch_up -=
                        (control_values.sensitivity * event.delta.y * window_scale).to_radians();
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
            .add_systems(Update, camera_mouse);
    }
}

/// Same as [`CameraPlugin`] but does not spawn a camera
pub struct ControlNoCamera;
impl Plugin for ControlNoCamera {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindings>()
            .add_systems(Update, camera_keys)
            .add_systems(Update, camera_mouse);
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
