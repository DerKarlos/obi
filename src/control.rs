/*
 * ToDo: rename "player..." to ???
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


We start with an argumente to select one control and later switch dynamically.
All controls will have the resource type control later (now Control)
Maximal one control/plurgin/systems should run (may be none)

What about the PlayerQuery??? Is it for Fly-Cam or for all controls

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
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::StartingValues;

/// Mouse sensitivity and movement speed
#[derive(Resource)]
pub struct ControlValues {
    pub sensitivity: f32,
    pub speed: f32,
    //pub view: GeoView,   NO roll
    pub _north: f32,
    pub _east: f32,
    pub _elevation: f32,
    pub yaw_direction: f32,
    pub pitch_up: f32,
    pub distance: f32,
}

impl Default for ControlValues {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 50.,
            // view: GeoView::default(),
            _north: 0.0,
            _east: 0.0,
            _elevation: 1.4,
            yaw_direction: 0.0,
            pitch_up: 0.0,
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
        }
    }
}

/// Used in queries when you want flycams and not other cameras
/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands, starting_values: Res<StartingValues>) {
    commands.spawn((
        Camera3d::default(),
        FlyCam,
        Transform::from_xyz(
            -0.2 * starting_values.range,
            0.3 * starting_values.range,
            1.0 * starting_values.range,
        )
        .looking_at(Vec3::new(0., 0.17 * starting_values.range, 0.), Vec3::Y),
    ));
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    key_bindings: Res<KeyBindings>,
    mut control_values: ResMut<ControlValues>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    let mut velocity = Vec3::ZERO;
    let elevation_fakt: f32 = 1. + time.delta_secs() / 1.0;
    let rotation_fact = time.delta_secs(); // delta rad = delta time * 1.0 (rad per second)

    let local_z = camera.local_z();
    let forward = -Vec3::new(local_z.x, 0., local_z.z);
    let right = Vec3::new(local_z.z, 0., -local_z.x);
    let upward = Vec3::new(0., 1., 0.);

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
            //control_values.view.elevation /= elevation_fakt;
            //
            // rotate
        } else if key == key_bindings.rotate_right || key == key_bindings.rotate_right2 {
            control_values.yaw_direction -= rotation_fact;
        } else if key == key_bindings.rotate_left || key == key_bindings.rotate_left2 {
            control_values.yaw_direction += rotation_fact;
        } else if key == key_bindings.rotate_up {
            control_values.pitch_up += rotation_fact;
        } else if key == key_bindings.rotate_down {
            control_values.pitch_up -= rotation_fact;
        //
        // zoom
        } else if key == key_bindings.zoom_out || key == key_bindings.zoom_out2 {
            control_values.distance *= elevation_fakt;
        } else if key == key_bindings.zoom_in {
            control_values.distance /= elevation_fakt;
        }
    }

    velocity = velocity.normalize_or_zero();
    let mut target = Vec3::new(
        control_values._east,
        control_values._elevation,
        control_values._north,
    );
    target += velocity * time.delta_secs() * control_values.speed;
    control_values._east = target.x;
    control_values._elevation = target.y;
    control_values._north = target.z;

    control_values.pitch_up = control_values.pitch_up.clamp(-1.54, 1.54);

    camera.rotation = Quat::from_euler(
        EulerRot::YXZ,
        control_values.yaw_direction,
        control_values.pitch_up,
        0.,
    );

    camera.translation = target - camera.forward() * control_values.distance;
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<ControlValues>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut state: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if mouse_buttons.pressed(MouseButton::Left) {
        if let Ok(window) = primary_window.single() {
            for mut transform in query.iter_mut() {
                for ev in state.read() {
                    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                    {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }

                    pitch = pitch.clamp(-1.54, 1.54);

                    // Order is important to prevent unintended roll
                    transform.rotation =
                        Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);

                    // https://bevy.org/examples/camera/camera-orbit/
                }
            }
        } else {
            warn!("Primary window not found for `player_look`!");
        }
    }
}

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlValues>()
            .init_resource::<KeyBindings>()
            .add_systems(Startup, setup_player)
            .add_systems(Update, player_move)
            .add_systems(Update, player_look);
    }
}

/// Same as [`PlayerPlugin`] but does not spawn a camera
pub struct NoCameraPlayerPlugin;
impl Plugin for NoCameraPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlValues>()
            .init_resource::<KeyBindings>()
            .add_systems(Update, player_move)
            .add_systems(Update, player_look);
    }
}
