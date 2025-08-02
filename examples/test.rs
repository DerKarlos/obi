//! Showcase how to use and configure FPS overlay.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // We need to spawn a camera (2d or 3d) to see the overlay
    commands.spawn(Camera2d);

    // Instruction text

    commands.spawn((
        Text::new(concat!(
            "Press 1 to toggle the overlay color.\n",
            "Press 4 to toggle the overlay visibility."
        )),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        },
    ));
}
