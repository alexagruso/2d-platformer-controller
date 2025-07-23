use avian2d::{math::*, prelude::*};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

const CONTROLLER_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
const OBSTACLE_COLOR: Color = Color::srgb(0.2, 0.7, 0.9);

const CONTROLLER_SIZE: Vector = Vector::new(30.0, 60.0); // Total width and height of the controller's capsule collider
const CONTROLLER_INITIAL_POSITION: Vector = Vector::new(0.0, 100.0);
// const CONTROLLER_SKIN_WIDTH: f32 = 4.0;

const HORIZONTAL_PLAYER_SPEED: f32 = 100.0;
const GRAVITY: f32 = 100.0;
const JUMP_SPEED: f32 = 50.0;
// const MINIMUM_MOVEMENT_DISTANCE: f32 = 0.0001;

fn platform_from_position(position: Vector, size: Vector, rotation: Scalar) -> impl Bundle {
    (
        Sprite {
            color: OBSTACLE_COLOR,
            custom_size: Some(Vec2::new(size.x, size.y)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 0.0)
            .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
    )
}

fn capsule_from_size(size: Vector) -> Capsule2d {
    Capsule2d::new(size.x / 2.0, size.y - size.x)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(capsule_from_size(CONTROLLER_SIZE))),
        MeshMaterial2d(materials.add(CONTROLLER_COLOR)),
        ControllerBundle::new(CONTROLLER_SIZE, CONTROLLER_INITIAL_POSITION),
        // This allows the camera to follow the player's position
        children![(
            Camera2d,
            Projection::Orthographic(OrthographicProjection::default_2d())
        )],
    ));

    commands.spawn(platform_from_position(
        Vector::new(0.0, 0.0),
        Vector::new(100.0, 10.0),
        0.0,
    ));

    commands.spawn(platform_from_position(
        Vector::new(0.0, 0.0),
        Vector::new(100.0, 10.0),
        0.0,
    ));

    commands.spawn(platform_from_position(
        Vector::new(0.0, 0.0),
        Vector::new(100.0, 10.0),
        0.0,
    ));
}

fn close_on_esc(mut exit: ResMut<Events<AppExit>>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

#[derive(Component)]
struct Camera;

fn zoom_camera(
    mut mouse_scroll: EventReader<MouseWheel>,
    mut camera: Query<&mut Projection, With<Camera2d>>,
) {
    let mut camera_projection = match camera.single_mut() {
        Ok(projection) => projection,
        Err(_) => return,
    };

    match &mut *camera_projection {
        Projection::Orthographic(projection) => {
            for scroll in mouse_scroll.read() {
                match scroll.unit {
                    MouseScrollUnit::Line => {
                        let scale = projection.scale;
                        projection.scale = (scale - scroll.y * 0.1).max(0.1);
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default().with_length_unit(20.0),
            PhysicsDebugPlugin::default(),
            ControllerPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (close_on_esc, zoom_camera))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .run();
}

#[derive(Component)]
struct Controller;

#[derive(Bundle)]
struct ControllerBundle {
    transform: Transform,
    velocity: LinearVelocity,
    collider: Collider,
    rigidbody: RigidBody,
}

impl ControllerBundle {
    fn new(size: Vector, starting_position: Vector) -> impl Bundle {
        (
            ControllerBundle {
                transform: Transform::from_translation(starting_position.extend(0.0)),
                velocity: LinearVelocity(Vector::ZERO),
                collider: capsule_from_size(size).into(),
                rigidbody: RigidBody::Kinematic,
            },
            Controller,
        )
    }
}

#[derive(Event)]
enum ControllerMovement {
    HorizontalMovement(f32),
    SetPosition(Vector),
    Jump,
}

struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ControllerMovement>()
            .add_systems(
                PhysicsSchedule,
                // TODO: explain why we put `collision_response` in the narrow phase
                collision_response.in_set(NarrowPhaseSet::Last),
            )
            .add_systems(Update, controller_input)
            .add_systems(FixedUpdate, controller_movement);
    }
}

fn controller_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut controller_movement_events: EventWriter<ControllerMovement>,
) {
    let mut horizontal_velocity = 0.0;
    if keyboard_input.pressed(KeyCode::KeyD) {
        horizontal_velocity += 1.0;
    } else if keyboard_input.pressed(KeyCode::KeyA) {
        horizontal_velocity -= 1.0;
    }

    use ControllerMovement as Event;
    controller_movement_events.write(Event::HorizontalMovement(horizontal_velocity));

    if keyboard_input.just_pressed(KeyCode::Space) {
        controller_movement_events.write(Event::Jump);
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        controller_movement_events.write(Event::SetPosition(CONTROLLER_INITIAL_POSITION));
    }
}

fn controller_movement(
    time: Res<Time<Fixed>>,
    mut controller_movement_events: EventReader<ControllerMovement>,
    mut controllers: Query<(&mut LinearVelocity, &mut Transform), With<Controller>>,
) {
    for event in controller_movement_events.read() {
        for (mut controller_velocity, mut controller_transform) in &mut controllers {
            use ControllerMovement as Event;
            match event {
                Event::HorizontalMovement(magnitude) => {
                    controller_velocity.x = magnitude * HORIZONTAL_PLAYER_SPEED
                }
                Event::Jump => controller_velocity.y = JUMP_SPEED,
                Event::SetPosition(position) => {
                    controller_transform.translation = position.extend(0.0);
                }
            }

            controller_velocity.y -= GRAVITY * time.delta_secs();
        }
    }
}

// struct CollideAndSlideConfig {
//     bounces: usize,
//     rotation: Scalar,
//     skin_width: Scalar,
//     max_slope_angle: Scalar,
//     filter: SpatialQueryFilter,
// }
//
// impl Default for CollideAndSlideConfig {
//     fn default() -> Self {
//         CollideAndSlideConfig {
//             bounces: 10,
//             rotation: 0.0,
//             skin_width: 0.015,
//             max_slope_angle: 60.0,
//             // this filter collides with everything
//             filter: SpatialQueryFilter::from_excluded_entities([]),
//         }
//     }
// }
//

fn collision_response(
    time: Res<Time<Fixed>>,
    spatial_query: Res<SpatialQueryPipeline>,
    mut controllers: Query<(&mut LinearVelocity, &Transform, &Collider, Entity), With<Controller>>,
) {
    for (mut velocity, transform, collider, entity) in &mut controllers {
        let cast_direction = match velocity.y.signum() {
            1.0 => Dir2::Y,
            -1.0 => Dir2::NEG_Y,
            0.0 => continue, // If the controller is still, we don't compute collisions for it
            _ => {
                panic!("Velocity should never be NaN");
            }
        };
        let cast_origin = transform.translation.xy();
        // Excluding the controller entity prevents controllers from colliding with themselves
        let cast_filter = SpatialQueryFilter::from_excluded_entities([entity]);

        let delta_secs = time.delta_secs();
        if let Some(hit) = spatial_query.cast_shape(
            &collider,
            cast_origin,
            0.0, // TODO: support rotations
            cast_direction,
            &ShapeCastConfig {
                max_distance: velocity.y.abs() * delta_secs,
                ..default()
            },
            &cast_filter,
        ) {
            let snap_to_surface = hit.distance * velocity.y.signum() / delta_secs;

            velocity.y = snap_to_surface;
        }
    }
}
