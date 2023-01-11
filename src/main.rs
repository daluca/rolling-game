use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use std::f32::consts::PI;

const MOVE_FORCE: f32 = 1500.0;

#[derive(Component)]
struct Player {
    id: usize,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
}

#[derive(Component)]
struct Goal;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 2D Camera
    commands.spawn(Camera2dBundle::default());

    // Spawn Players
    spawn_players(0, Vec2::new(-100.0, 0.0), &mut commands, &asset_server);
    spawn_players(1, Vec2::new(100.0, 0.0), &mut commands, &asset_server);

    // Spawn Pieces
    spawn_pieces(Vec2::new(150.0, 150.0), 0.0, &mut commands, &asset_server);
    spawn_pieces(
        Vec2::new(-350.0, 50.0),
        PI * 0.5,
        &mut commands,
        &asset_server,
    );
    spawn_pieces(Vec2::new(-150.0, -200.0), PI, &mut commands, &asset_server);
    spawn_pieces(
        Vec2::new(200.0, -50.0),
        PI * 1.5,
        &mut commands,
        &asset_server,
    );

    // Spawn Goal
    commands
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(450.0, -300.0, 0.0),
            texture: asset_server.load("hole_large_end.png"),
            ..default()
        })
        .insert(Collider::ball(28.0))
        .insert(Sensor)
        .insert(Goal);
}

fn spawn_players(
    id: usize,
    location: Vec2,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let image = if id == 0 { "blue" } else { "red" };

    // Spawn Player
    commands
        .spawn(SpriteBundle {
            transform: Transform::from_translation(location.extend(1.0)),
            texture: asset_server.load(format!("ball_{image}_large.png")),
            ..default()
        })
        .insert(Collider::ball(32.0))
        .insert(RigidBody::Dynamic)
        .insert(ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        })
        .insert(Restitution::coefficient(1.0))
        .insert(Damping {
            linear_damping: 0.6,
            angular_damping: 0.3,
        })
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::left_stick(), Action::Move)
                .set_gamepad(Gamepad { id })
                .insert(
                    if id == 0 {
                        VirtualDPad::wasd()
                    } else {
                        VirtualDPad::arrow_keys()
                    },
                    Action::Move,
                )
                .build(),
        })
        .insert(Player { id });
}

fn spawn_pieces(
    location: Vec2,
    rotation: f32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    commands
        .spawn(SpriteBundle {
            transform: Transform {
                translation: location.extend(0.0),
                rotation: Quat::from_rotation_z(rotation),
                ..default()
            },
            texture: asset_server.load("block_corner.png"),
            ..default()
        })
        .insert(Collider::round_triangle(
            Vec2::new(-23.0, -23.0),
            Vec2::new(-23.0, 23.0),
            Vec2::new(23.0, -23.0),
            0.05,
        ))
        .insert(RigidBody::Fixed)
        .insert(Restitution::coefficient(1.0));
}

fn movement(
    mut query: Query<(&ActionState<Action>, &mut ExternalForce), With<Player>>,
    time: Res<Time>,
) {
    for (action_state, mut external_force) in &mut query {
        let axis_vector = action_state.clamped_axis_pair(Action::Move).unwrap().xy();
        external_force.force = axis_vector * MOVE_FORCE * time.delta_seconds();
    }
}

fn win_condition(
    rapier_context: Res<RapierContext>,
    player_query: Query<(Entity, &Player)>,
    goal_query: Query<Entity, With<Goal>>,
) {
    let goal_entity = goal_query.single();
    for (player_entity, player) in player_query.iter() {
        if rapier_context.intersection_pair(goal_entity, player_entity) == Some(true) {
            println!("Player {} wins!", player.id);
        }
    }
}

fn collision_sounds(
    rapier_context: Res<RapierContext>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>
) {
    let mut just_collided = false;
    for pair in rapier_context.contact_pairs() {
        if pair.has_any_active_contacts() {
            just_collided = true;
        }
    }
    if just_collided {
        let sound = asset_server.load("impactGlass_heavy_002.ogg");
        audio.play(sound);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Rolling Game".into(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(setup)
        .add_system(movement)
        .add_system(win_condition)
        .add_system(collision_sounds)
        .run();
}
