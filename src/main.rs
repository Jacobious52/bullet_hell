use std::f32::consts::PI;

use bevy::{
    math::vec2,
    prelude::*,
    window::WindowResized,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            Update,
            (
                move_ship,
                point_ship,
                add_drag,
                resize_background,
                scale_ship,
                spawn_bullet,
                remove_bullets,
                check_collisions,
            ),
        )
        .add_systems(FixedUpdate, apply_velocity)
        .run();
}

#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Bullet;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct BulletTimer(Timer);

fn remove_bullets(
    mut commands: Commands,
    mut q: Query<(Entity, &mut BulletTimer)>,
    time: Res<Time>,
) {
    for (entity, mut fuse_timer) in q.iter_mut() {
        // timers gotta be ticked, to work
        fuse_timer.0.tick(time.delta());

        // if it finished, despawn the bomb
        if fuse_timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn check_collisions(
    mut commands: Commands,
    q: Query<(Entity, &Transform, &BulletTimer), With<Bullet>>,
    q2: Query<(Entity, &Transform), With<Ship>>,
) {
    for (entity, bullet_transform, bullet_timer) in q.iter() {
        if bullet_timer.0.elapsed_secs() < 0.2 {
            continue;
        }
        for (_entity2, ship_transform) in q2.iter() {
            let bs = vec2(249.0, 144.0) * bullet_transform.scale.xy();
            let r1 = Rect::new(
                bullet_transform.translation.x,
                bullet_transform.translation.y,
                bullet_transform.translation.x + bs.x,
                bullet_transform.translation.y + bs.y,
            );

            let ss = vec2(100.0, 100.0) * ship_transform.scale.xy();
            let r2 = Rect::new(
                ship_transform.translation.x,
                ship_transform.translation.y,
                ship_transform.translation.x + ss.x,
                ship_transform.translation.y + ss.y,
            );

            if !r1.intersect(r2).is_empty() {
                commands.entity(entity).despawn();
                //ship_transform.translation = Vec3::new(0.0, 0.0, 0.0);
            }
        }
    }
}

fn spawn_bullet(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(&Transform, &Velocity), With<Ship>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }

    const BULLET_SPEED: f32 = 1000.0;

    for (transform, vel) in query.iter() {
        commands.spawn((
            Bullet,
            BulletTimer(Timer::from_seconds(2.0, TimerMode::Once)),
            Velocity(vel.normalize() * BULLET_SPEED),
            SpriteBundle {
                transform: Transform {
                    translation: transform.translation,
                    rotation: transform.rotation * Quat::from_rotation_z(PI / 2.0),
                    scale: Vec3::new(0.4, 0.4, 1.0),
                },
                texture: asset_server.load("laser.png"),
                ..default()
            },
        ));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Background,
        SpriteBundle {
            texture: asset_server.load("background.png"),
            ..default()
        },
    ));

    commands.spawn((
        Ship,
        Velocity(Vec2::new(0.0, 0.0)),
        SpriteBundle {
            texture: asset_server.load("ship.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            ..default()
        },
    ));
}

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity)>,
    camera: Query<&Camera>,
) {
    let delta = time.delta().as_secs_f32();
    let camera = camera.single();
    let camera_rect = camera.logical_viewport_rect().unwrap();

    for (mut transform, vel) in query.iter_mut() {
        transform.translation += vel.0.extend(0.0) * delta;

        // if ship goes off screen, wrap it around
        if transform.translation.x < camera_rect.min.x - camera_rect.half_size().x {
            transform.translation.x = camera_rect.max.x - camera_rect.half_size().x;
        } else if transform.translation.x > camera_rect.max.x - camera_rect.half_size().x {
            transform.translation.x = camera_rect.min.x - camera_rect.half_size().x;
        }

        if transform.translation.y < camera_rect.min.y - camera_rect.half_size().y {
            transform.translation.y = camera_rect.max.y - camera_rect.half_size().y;
        } else if transform.translation.y > camera_rect.max.y - camera_rect.half_size().y {
            transform.translation.y = camera_rect.min.y - camera_rect.half_size().y;
        }
    }
}

fn add_drag(mut query: Query<&mut Velocity, With<Ship>>) {
    const DRAG: f32 = 0.005;

    for mut vel in query.iter_mut() {
        vel.x *= 1.0 - DRAG;
        vel.y *= 1.0 - DRAG;
    }
}

fn move_ship(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Velocity, With<Ship>>) {
    const ACCELERATION: f32 = 10.0;

    for mut vel in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Left) {
            vel.x -= ACCELERATION;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            vel.x += ACCELERATION;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            vel.y += ACCELERATION;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            vel.y -= ACCELERATION;
        }

        vel.x = vel.x.min(500.0).max(-500.0);
        vel.y = vel.y.min(500.0).max(-500.0);
    }
}

// point the ship in the direction of travel
fn point_ship(mut query: Query<(&mut Transform, &Velocity), With<Ship>>) {
    for (mut transform, vel) in query.iter_mut() {
        if vel.length_squared() > 0.0 {
            transform.rotation =
                Quat::from_rotation_z(-PI / 2.0) * Quat::from_rotation_z(vel.y.atan2(vel.x));
        }
    }
}

fn resize_background(
    mut events: EventReader<WindowResized>,
    mut query: Query<&mut Transform, With<Background>>,
    camera: Query<&Camera>,
) {
    for e in events.read() {
        let camera = camera.single();
        let camera_rect = camera.logical_viewport_rect().unwrap();

        let mut background_transform = query.single_mut();

        // resize background to fit camera
        background_transform.scale = Vec3::new(
            1.1 * camera_rect.width() / e.width,
            1.1 * camera_rect.height() / e.height,
            1.0,
        );
    }
}

// make the ship bigger as it gets to the edges of the screen
fn scale_ship(mut query: Query<&mut Transform, With<Ship>>, camera: Query<&Camera>) {
    let camera = camera.single();
    let camera_rect = camera.logical_viewport_rect().unwrap();

    for mut transform in query.iter_mut() {
        let scale =
            1.0 + 0.5 * (transform.translation.x - camera_rect.min.x).abs() / camera_rect.width();
        transform.scale = Vec3::new(scale, scale, 1.0);
    }
}
