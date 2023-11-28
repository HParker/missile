use rand::Rng;

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, shoot_missile, move_missiles))
        .run();
}


#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Missile {
    progress: f32,
    start: Vec3,
    start_tag: Vec3,
    target: Vec3,
    target_tag: Vec3,
    sin_phase: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let player_capsule = shape::Capsule::default();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(player_capsule.into()),
            material: debug_material.clone(),
            transform: Transform::from_xyz(
                0.0,
                1.0,
                0.0,
            ),
            ..default()
        },
        Player,
    )).with_children(|parent| {
        parent.spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 8.0, 16.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        });
    });

    let enemy_cylinder = shape::Cylinder::default();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(enemy_cylinder.into()),
            material: debug_material.clone(),
            transform: Transform::from_xyz(
                0.0,
                1.0,
                -15.0,
            ),
            ..default()
        },
        Enemy,
    ));

    // point light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });
}

fn move_player(
    mut query: Query<&mut Transform, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for mut transform in &mut query {
        if keyboard_input.pressed(KeyCode::W) {
            let direction = -transform.local_z();
            transform.translation += direction * 5.0 * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::S) {
            let direction = transform.local_z();
            transform.translation += direction * 5.0 * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::A) {
            let direction = -transform.local_x();
            transform.translation += direction * 5.0 * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::D) {
            let direction = transform.local_x();
            transform.translation += direction * 5.0 * time.delta_seconds();
        }
    }
}

fn shoot_missile(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    enemy: Query<(&Transform, Entity), With<Enemy>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // TODO: just released?
    // press to lock on, release to fire?

    // TODO: should we make rng once-per-game?
    let mut rng = rand::thread_rng();

    if keyboard_input.just_pressed(KeyCode::E) || keyboard_input.pressed(KeyCode::F) {
        for player_transform in &player {

            let random_dest_tag_x: f32 = rng.gen_range(-8.0..8.0);
            let random_dest_tag_y: f32 = rng.gen_range(0.0..10.0);
            let random_dest_tag_z: f32 = rng.gen_range(-5.0..5.0);

            let mut distance: f32 = 0.0;
            let mut closest_enemy = None;
            for (enemy_transform, enemy_entity) in &enemy {
                if closest_enemy.is_none() ||
                    player_transform.translation.distance(enemy_transform.translation) < distance {
                        closest_enemy = Some((enemy_transform, enemy_entity));
                        distance = player_transform.translation.distance(enemy_transform.translation);
                    }
            }

            // TODO: remove entity
            if let Some((enemy_transform, _enemy_entity)) = closest_enemy {
                let debug_material = materials.add(StandardMaterial {
                    base_color_texture: Some(images.add(uv_debug_texture())),
                    ..default()
                });

                let bullet_sphere = shape::UVSphere::default();
                let bullet_transform = player_transform.clone().with_scale(Vec3::splat(0.2));

                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(bullet_sphere.into()),
                        material: debug_material.clone(),
                        transform: bullet_transform,
                        ..default()
                    },
                    Missile {
                        progress: 0.0,
                        start: bullet_transform.translation.clone(),
                        start_tag: bullet_transform.translation.clone() + Vec3::from((0.0, 8.0, 0.0)),
                        target: enemy_transform.translation.clone(),
                        target_tag: enemy_transform.translation.clone() + Vec3::from((random_dest_tag_x, random_dest_tag_y, random_dest_tag_z)),
                        sin_phase: rng.gen_range(-5.0..5.0),
                    },
                ));
            }
        }
    }
}

fn move_missiles(
    mut commands: Commands,
    mut bullet_query: Query<(&mut Transform, &mut Missile, Entity), With<Missile>>,
    time: Res<Time>,
) {
    for (mut missile_transform, mut missile, missile_entity) in &mut bullet_query {
        // do the thing!
        if missile.progress >= 1.0 {
            commands.entity(missile_entity).despawn();
        }

        let mut rng = rand::thread_rng();

        let progress = missile.progress;
        let envelope: f32 = 1.0 - (1.0 - 2.0 * progress) * (1.0 - 2.0 * progress);
        let time_left = 1.0 - progress;

        let start = missile.start;
        let start_tag = missile.start_tag;
        let finish = missile.target;
        let finish_tag = missile.target_tag;

        // Bezier curve
        // missile_transform.translation =
        //     (time_left * time_left * time_left)    * start +
	//     (3.0 * time_left * time_left * progress) * start_tag +
	//     (3.0 * time_left * progress * progress)  * finish_tag +
	//     (progress * progress * progress)       * finish;

        // bezier with sine "noise"
        missile_transform.translation =
            (time_left * time_left * time_left)    * start +
	    (3.0 * time_left * time_left * progress) * start_tag +
	    (3.0 * time_left * progress * progress)  * finish_tag +
	    (progress * progress * progress)       * finish;
        missile_transform.translation.x += envelope * f32::sin(progress * 8.0 + missile.sin_phase);
        missile_transform.translation.y += envelope * f32::sin(progress * 8.0 + missile.sin_phase + 16.0);

        // lerp
        // missile_transform.translation = start + (finish - start) * progress;

        // sine
        // missile_transform.translation = start + (finish - start) * progress;
        // missile_transform.translation.x += f32::sin(progress * 10.0 + missile.sin_phase);

        // spiral
        // missile_transform.translation.x += f32::sin(progress * 10.0);
        // missile_transform.translation.y += f32::sin(progress * 10.0 + 20.0);

        missile.progress += time.delta_seconds();
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
