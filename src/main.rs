use bevy::prelude::*;
use bevy::render::pass::ClearColor;

fn main() {
    App::build()
        // common app data known as resources
        .add_resource(WindowDescriptor {
            title: "Space shooter".to_string(),
            width: 1024.0,
            height: 1024.0,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.08, 0.08, 0.08)))
        // app plugins
        .add_plugins(DefaultPlugins)
        // systems fn, doing the actual work
        .add_system(ship_movement.system())
        .add_system(shoot.system())
        .add_system(move_laser.system())
        .add_system(clear_offscreen_lasers.system())
        // things we do before running anything
        .add_startup_system(setup.system())
        .add_startup_stage("space_shooter", SystemStage::single(spawn_ship.system()))
        // running
        .run();
}

// entities
struct Ship;
struct Laser;

// resources
struct Materials {
    ship_material:     Handle<ColorMaterial>,
    weapons_material:  Handle<ColorMaterial>
}

struct LaserCooldown {
    cd_time: Timer,
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player_texture_handle = asset_server.load("textures/playerShip1_blue.png");
    let laser_texture_handle  = asset_server.load("textures/laserRed01.png");
    commands
        // spawns cameras views
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default());
        // create ressources
    commands
        // creating a Materials resource that will handle all textures assets, so we load only once
        .insert_resource(Materials {
            ship_material:     materials.add(player_texture_handle.into()),
            weapons_material:  materials.add(laser_texture_handle.into())
        });
}

fn spawn_ship(
    commands: &mut Commands,
    materials: ResMut<Materials>,
) {
    commands
        .spawn(SpriteBundle {
            material: materials.ship_material.clone().into(),
            transform: Transform::from_translation(Vec3::new(0.0, -256.0, 0.0)),
            ..Default::default()
        })
        .with(Ship)
        .with(LaserCooldown {
            cd_time: Timer::from_seconds(0.4, false)
        });
}

fn ship_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Ship>>
) {
    let v = 250.0;
    let dx = v*time.delta_seconds();
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
    }
}

fn shoot(
    commands: &mut Commands,
    materials: ResMut<Materials>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut LaserCooldown), With<Ship>>
) {
    // represent the offset, so lasers spawn at the nose of the ship
    const SHIP_OFFSET: f32 = 64.0;
    for (ship_pos, mut laser_cd) in query.iter_mut() {
        if
           keyboard_input.pressed(KeyCode::Space)
           &&
           laser_cd.cd_time.tick(time.delta_seconds()).finished() {

            commands
                .spawn(SpriteBundle {
                    material: materials.weapons_material.clone().into(),
                    transform: Transform::from_translation(Vec3::new(ship_pos.translation.x,
                                                                     ship_pos.translation.y + SHIP_OFFSET,
                                                                     0.0)),
                    ..Default::default()
                })
                .with(Laser);
            laser_cd.cd_time.reset();
        }
    }
}

fn move_laser(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Laser>>
) {
    let v = 700.0;
    let dx = v*time.delta_seconds();
    for mut transform in query.iter_mut() {
        transform.translation.y += dx;
    }
}

fn clear_offscreen_lasers(
    commands: &mut Commands,
    window: Res<Windows>,
    query: Query<(&Transform, Entity), With<Laser>>
) {
    const DESPAWN_DIST: f32 = 100.0;
    let window = window.get_primary().unwrap();
    for (laser_pos, laser) in query.iter() {
        if laser_pos.translation.y >= window.height() - DESPAWN_DIST {
            commands
                .despawn(laser);
        }
    }
}
