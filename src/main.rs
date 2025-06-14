use avian2d::{math::*, prelude::*};
use bevy::prelude::*;
use character_controller::{CharacterControllerBundle, CharacterControllerPlugin};

const PLAYER_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const OBSTACLE_COLOR: Color = Color::srgb(0.2, 0.7, 0.9);

mod character_controller;

fn platform(position: Vector, size: Vector, rotation: Scalar) -> impl Bundle {
    (
        Sprite {
            color: Color::srgb(0.7, 0.7, 0.8),
            custom_size: Some(Vec2::new(size.x, size.y)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 0.0)
            .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
    )
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Capsule2d::new(15.0, 20.0))),
        MeshMaterial2d(materials.add(PLAYER_COLOR)),
        Transform::from_xyz(0.0, 45.0, 0.0),
        CharacterControllerBundle::new(Collider::capsule(15.0, 20.0), Vector::NEG_Y * 10.0)
            .with_movement(100.0, 0.92, 400.0, (46.0 as Scalar).to_radians()),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(15.0))),
        MeshMaterial2d(materials.add(OBSTACLE_COLOR)),
        Transform::from_xyz(0.0, 100.0, 0.0),
        RigidBody::Static,
        Collider::circle(15.0),
    ));

    commands.spawn(platform(
        Vector::new(0.0, 0.0),
        Vector::new(100.0, 10.0),
        0.0,
    ));

    commands.spawn(platform(
        Vector::new(75.0, 0.0),
        Vector::new(200.0, 10.0),
        45.0,
    ));

    commands.spawn(platform(
        Vector::new(-50.0, 0.0),
        Vector::new(200.0, 20.0),
        -70.0,
    ));
}

fn close_on_esc(mut exit: ResMut<Events<AppExit>>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default().with_length_unit(20.0),
            CharacterControllerPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .run();
}
