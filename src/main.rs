use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::collide,
};

use rand::Rng;

fn main() {
    App::build()
        // common app data known as resources
        .add_resource(WindowDescriptor {
            title: "Space shooter".to_string(),
            width: 800.0,
            height: 800.0,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.08, 0.08, 0.08)))
        .add_resource(ScoreBoard {score: 0})
        // app plugins
        .add_plugins(DefaultPlugins)
        // systems fn, doing the actual work
        .add_system(ship_movement.system())
        .add_system(spawn_enemy.system())
        .add_system(shoot.system())
        .add_system(enemy_shoot.system())
        .add_system(move_laser.system())
        .add_system(laser_collision_system.system())
        .add_system(clear_offscreen_lasers.system())
        .add_system(scoreboard_system.system())
        // things we do before running anything
        .add_startup_system(setup.system())
        .add_startup_stage("space_shooter", SystemStage::single(spawn_ship.system()))
        // running
        .run();
}

// entities
struct Ship;
struct Laser;
struct Enemy;
struct EnemyLaser;

// resources
struct Materials {
    ship_material: Handle<ColorMaterial>,
    enemy_material: Handle<ColorMaterial>,
    weapons_material: Handle<ColorMaterial>,
    enemy_weapons_material: Handle<ColorMaterial>,
}

#[derive(Debug)]
struct LaserCooldown {
    cd_time: Timer,
}

#[derive(Debug)]
struct EnemySpawnTimer{
    enemy_timer: Timer,
}

struct EnemyShootTimer {
    enemy_shoot_timer: Timer,
}

struct ScoreBoard {
    score: usize,
}

// mechanics
enum Collider {
    Player,
    Solid,
    Projectile,
    Scorable,
}

const X_BOUND: f32 = 700.0;
const Y_BOUND: f32 = 700.0;

const PLAYER_SHIP_TEXTURE: &str = "textures/playerShip1_blue.png";
const PLAYER_SHIP_LASER_TEXTURE: &str = "textures/laserRed01.png";

const ENEMY_SHIP_TEXTURE: &str = "textures/enemyRed1.png";
const ENEMY_SHIP_LASER_TEXTURE: &str = "textures/laserGreen01.png";

const SCOREBOARD_FONT: &str = "fonts/kenvector_future_thin.ttf";

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // spawns cameras view
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default());

    // load assets and create ressources
    let player_texture_handle = asset_server.load(PLAYER_SHIP_TEXTURE);
    let enemy_texture_handle = asset_server.load(ENEMY_SHIP_TEXTURE);
    let laser_texture_handle = asset_server.load(PLAYER_SHIP_LASER_TEXTURE);
    let enemy_laser_texture_handle = asset_server.load(ENEMY_SHIP_LASER_TEXTURE);

    commands
        .insert_resource(Materials {
        ship_material: materials.add(player_texture_handle.into()),
        enemy_material: materials.add(enemy_texture_handle.into()),
        weapons_material: materials.add(laser_texture_handle.into()),
        enemy_weapons_material: materials.add(enemy_laser_texture_handle.into()),
    });

    commands
        .spawn(TextBundle {
            text: Text {
            font: asset_server.load(SCOREBOARD_FONT),
                value: "Score:".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.5, 0.5, 1.0),
                    font_size: 40.0,
                    ..Default::default()
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        });

    // Add enemy spawning timer
    commands
        .insert_resource(EnemySpawnTimer {
            enemy_timer: Timer::from_seconds(2.0, false)
        });

    // Add space boundaries
    let wall_material = materials.add(Color::rgb(0.4, 0.5, 0.8).into());
    let wall_thickness = 7.0;
    let bounds = Vec2::new(X_BOUND, Y_BOUND);

    commands
        // left wall
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(-bounds.x / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid)
        // right wall
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(bounds.x / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y + wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid)
        // bottom wall
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -bounds.y / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid)
        // top wall
        .spawn(SpriteBundle {
            material: wall_material,
            transform: Transform::from_translation(Vec3::new(0.0, bounds.y / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(bounds.x + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        .with(Collider::Solid);
}

fn spawn_ship(
    commands: &mut Commands,
    materials: ResMut<Materials>
) {
    commands
        .spawn(SpriteBundle {
            material: materials.ship_material.clone().into(),
            transform: {
                let mut t = Transform::from_translation(Vec3::new(0.0, -256.0, 0.0));
                t.apply_non_uniform_scale(Vec3::new(0.8, 0.8, 0.8));
                t
            },
            ..Default::default()
        })
        .with(Ship)
        .with(LaserCooldown {cd_time: Timer::from_seconds(0.4, false)})
        .with(Collider::Player);
}

fn ship_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Ship>>,
) {
    let v = 250.0;
    let dx = v * time.delta_seconds();
    for mut ship_pos in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Left) {
            ship_pos.translation.x -= dx;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            ship_pos.translation.x += dx;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            ship_pos.translation.y -= dx;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            ship_pos.translation.y += dx;
        }

        let translation = &mut ship_pos.translation;
        // bound ship movement within walls
        translation.x = translation
            .x
            .min(X_BOUND / 2.0 - 50.0)
            .max(-X_BOUND / 2.0 + 50.0);
        translation.y = translation
            .y
            .min(Y_BOUND / 2.0 - 50.0)
            .max(-Y_BOUND / 2.0 + 50.0);
    }
}

fn shoot(
    commands: &mut Commands,
    materials: ResMut<Materials>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut LaserCooldown), With<Ship>>,
) {
    // represent the offset, so lasers spawn at the nose of the ship
    const SHIP_OFFSET: f32 = 64.0;
    for (ship_pos, mut laser_cd) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Space)
            && laser_cd.cd_time.tick(time.delta_seconds()).finished()
        {
            commands
                .spawn(SpriteBundle {
                    material: materials.weapons_material.clone().into(),
                    transform: Transform::from_translation(Vec3::new(
                        ship_pos.translation.x,
                        ship_pos.translation.y + SHIP_OFFSET,
                        0.0,
                    )),
                    ..Default::default()
                })
                .with(Laser)
                .with(Collider::Projectile);
            laser_cd.cd_time.reset();
        }
    }
}

fn move_laser(
    time: Res<Time>,
    mut player_laser_query: Query<&mut Transform, With<Laser>>,
    mut enemy_laser_query: Query<&mut Transform, With<EnemyLaser>>
) {
    let v = 700.0;
    let dy = v * time.delta_seconds();
    for mut transform in player_laser_query.iter_mut() {
        transform.translation.y += dy;
    }
    for mut transform in enemy_laser_query.iter_mut() {
        transform.translation.y -= dy;
    }
}

fn clear_offscreen_lasers(
    commands: &mut Commands,
    window: Res<Windows>,
    mut player_laser_query: Query<(&Transform, Entity), With<Laser>>,
    mut enemy_laser_query: Query<(&Transform, Entity), With<Laser>>,
    //wall_query: Query<(&Entity, &Transform, &Collider)>
) {
    let window = window.get_primary().unwrap();
    const DESPAWN_RATIO: f32 = 0.4;
    for (laser_pos, laser) in player_laser_query.iter_mut() {
        // despawn lasers when they get out of window
        //    for (collider_entity, collider_transform, collider) in wall_query.iter_mut() {
        //
        //    }
        if laser_pos.translation.y >= window.height() * DESPAWN_RATIO {
            commands.despawn(laser);
        }
    }

    for (laser_pos, laser) in enemy_laser_query.iter_mut() {
        // despawn lasers when they get out of window
        //    for (collider_entity, collider_transform, collider) in wall_query.iter_mut() {
        //
        //    }
        if laser_pos.translation.y >= window.height() * DESPAWN_RATIO {
            commands.despawn(laser);
        }

    }
}

fn spawn_enemy(
    commands: &mut Commands,
    time: Res<Time>,
    mut enemy_timer: ResMut<EnemySpawnTimer>,
    material: Res<Materials>)
{
    if enemy_timer.enemy_timer.tick(time.delta_seconds()).finished() {
        const ENEMY_SPAWN_X: f32 = 256.0;
        let enemy_rand_x: f32 = rand::thread_rng().gen_range(-200.0..200.0);
        commands
            .spawn(SpriteBundle {
                material: material.enemy_material.clone().into(),
                transform: {
                    let mut t =
                        Transform::from_translation(Vec3::new(enemy_rand_x, ENEMY_SPAWN_X, 0.0));
                    t.apply_non_uniform_scale(Vec3::new(0.8, 0.8, 0.8));
                    t
                },
                ..Default::default()
            })
            .with(Enemy)
            .with(Collider::Scorable)
            .with(EnemyShootTimer {enemy_shoot_timer: Timer::from_seconds(1.5, false)});
        enemy_timer.enemy_timer.reset();
    }
}

fn enemy_shoot(
    commands: &mut Commands,
    time: Res<Time>,
    materials: ResMut<Materials>,
    mut enemy_shooting: Query<(&Transform, &mut EnemyShootTimer), With<Enemy>>
) {
    const ENEMY_OFFSET: f32 = 64.0;
    for (enemy_pos, mut enemy_shoot_timer) in enemy_shooting.iter_mut() {
        if enemy_shoot_timer.enemy_shoot_timer.tick(time.delta_seconds()).finished() {
            commands
                .spawn(SpriteBundle {
                    material: materials.enemy_weapons_material.clone().into(),
                    transform: Transform::from_translation(Vec3::new(
                        enemy_pos.translation.x,
                        enemy_pos.translation.y - ENEMY_OFFSET,
                        0.0,
                    )),
                    ..Default::default()
                })
                .with(EnemyLaser)
                .with(Collider::Projectile);
            enemy_shoot_timer.enemy_shoot_timer.reset();
        }
    }
}

fn laser_collision_system(
    commands: &mut Commands,
    mut scoreboard: ResMut<ScoreBoard>,
    mut laser_query: Query<(Entity, &Transform, &Sprite), With<Laser>>,
    mut collider_query: Query<(Entity, &Collider,&Transform, &Sprite)>,
) {
    for (laser, laser_transform, laser_sprite) in laser_query.iter_mut() {
        let laser_size = laser_sprite.size;

        for (collider_entity, collider, transform, sprite) in collider_query.iter_mut() {
            let collision = collide(
                laser_transform.translation,
                laser_size,
                transform.translation,
                sprite.size,
            );
            if let Some(_) = collision {
                // scorable colliders should be despawned and increment the scoreboard on collision
                if let Collider::Scorable = *collider {
                    commands.despawn(collider_entity);
                    scoreboard.score += 1;
                    commands.despawn(laser);
                }

            }
        }
    }
 }

fn scoreboard_system(
    scoreboard: Res<ScoreBoard>,
    mut query: Query<&mut Text>
) {
    for mut text in query.iter_mut() {
        text.value = format!("Score: {}", scoreboard.score);
    }
}
