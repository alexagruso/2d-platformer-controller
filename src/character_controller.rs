use avian2d::{math::*, prelude::*};
use bevy::prelude::*;

const MAX_BOUNCES: usize = 5;
const NORMAL_COUNT: usize = 10;
const SKIN_WIDTH: f32 = 0.015;

#[derive(Component)]
pub struct CharacterController;

#[derive(Component)]
#[component(storage = "SparseSet")]
struct Grounded;

#[derive(Component)]
struct JumpImpulse(Scalar);

#[derive(Component)]
struct Gravity(Vector);

#[derive(Component)]
struct MaxSlopeAngle(Scalar);

#[derive(Component)]
struct MovementAcceleration(Scalar);

#[derive(Component)]
struct MovementDamping(Scalar);

#[derive(Bundle)]
struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDamping,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub const fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDamping(damping),
            jump_impulse: JumpImpulse(jump_impulse),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 7.0, PI * 0.45)
    }
}

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    gravity: Gravity,
    movement: MovementBundle,
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider, gravity: Vector) -> Self {
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Kinematic,
            collider,
            ground_caster: ShapeCaster::new(caster_shape, Vector::ZERO, 0.0, Dir2::NEG_Y)
                .with_max_distance(10.0),
            gravity: Gravity(gravity),
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, damping, jump_impulse, max_slope_angle);
        self
    }
}

#[derive(Event)]
enum MovementAction {
    Walk(Scalar),
    Sprint(Scalar),
    Jump,
}

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    update_grounded,
                    apply_gravity,
                    movement,
                    apply_damping,
                )
                    .chain(),
            )
            .add_systems(
                PhysicsSchedule,
                kinematic_collision_response.in_set(NarrowPhaseSet::Last),
            );
    }
}

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut movement_events: EventWriter<MovementAction>,
) {
    let left = keyboard_input.pressed(KeyCode::KeyA);
    let right = keyboard_input.pressed(KeyCode::KeyD);

    let horizontal_direction = (right as i8 - left as i8) as Scalar;

    if horizontal_direction != 0.0 {
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            movement_events.write(MovementAction::Sprint(horizontal_direction));
        } else {
            movement_events.write(MovementAction::Walk(horizontal_direction));
        }
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_events.write(MovementAction::Jump);
    }
}

fn update_grounded(
    mut commands: Commands,
    mut controllers: Query<(Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>)>,
) {
    for (entity, hits, rotation, max_slope_angle) in &mut controllers {
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                (rotation * -hit.normal2).angle_to(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn movement(
    mut movement_events: EventReader<MovementAction>,
    mut controllers: Query<(
        &MovementAcceleration,
        &JumpImpulse,
        &mut LinearVelocity,
        Has<Grounded>,
    )>,
) {
    for event in movement_events.read() {
        for (acceleration, jump_impulse, mut velocity, is_grounded) in &mut controllers {
            match event {
                MovementAction::Walk(direction) => {
                    velocity.x = direction * acceleration.0;
                }
                MovementAction::Sprint(direction) => {
                    velocity.x = direction * acceleration.0;
                }
                MovementAction::Jump => {
                    if is_grounded || !is_grounded {
                        velocity.y = jump_impulse.0;
                    }
                }
            }
        }
    }
}

fn apply_gravity(mut controllers: Query<(&Gravity, &mut LinearVelocity)>) {
    for (gravity, mut velocity) in &mut controllers {
        **velocity += gravity.0;
    }
}

fn apply_damping(mut controllers: Query<(&MovementDamping, &mut LinearVelocity)>) {
    for (damping, mut velocity) in &mut controllers {
        velocity.x *= damping.0;
    }
}

fn collide_and_slide(
    position: Vector,
    velocity: Vector,
    collider: &Collider,
    spatial_query: &Res<SpatialQueryPipeline>,
    cast_filter: &SpatialQueryFilter,
) -> Vector {
    let mut cast_direction = match Dir2::new(velocity) {
        Ok(direction) => direction,
        Err(_) => return Vector::ZERO,
    };
    let mut cast_distance = velocity.length();

    let mut result_velocity = Vector::ZERO;
    'bounces: for _ in 0..MAX_BOUNCES {
        if let Some(hit) = spatial_query.cast_shape(
            collider,
            position,
            0.0, // TODO: support custom rotations
            cast_direction,
            &ShapeCastConfig {
                max_distance: cast_distance + SKIN_WIDTH,
                ..default()
            },
            cast_filter,
        ) {}
    }

    result_velocity
}

fn kinematic_collision_response(
    time: Res<Time>,
    spatial_query: Res<SpatialQueryPipeline>,
    mut controllers: Query<
        (
            &mut Transform,
            &Position,
            &LinearVelocity,
            &Collider,
            Entity,
        ),
        With<CharacterController>,
    >,
) {
    let (mut transform, position, velocity, collider, player_entity) = controllers
        .single_mut()
        .expect("There should always be exactly 1 player entity");

    let cast_filter = SpatialQueryFilter::from_excluded_entities([player_entity]);

    let mut new_motion = collide_and_slide(
        **position,
        velocity.with_y(0.0) * time.delta_secs(),
        collider,
        &spatial_query,
        &cast_filter,
    );

    new_motion += collide_and_slide(
        **position,
        velocity.with_x(0.0) * time.delta_secs(),
        collider,
        &spatial_query,
        &cast_filter,
    );

    transform.translation += Vec3::new(new_motion.x, new_motion.y, 0.0);
}
