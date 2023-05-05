use bevy::prelude::*;
use lib_tower_defense::resource_system;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_startup_system(setup)
        .add_system(animate_sprite)
        .run();
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Load player sprite sheet
    let player_resource_id =
        resource_system::Resource::create("assets/sara-cal.png".to_owned(), false).unwrap();
    let player_texture_handle = asset_server.load(
        resource_system::Resource::try_get(player_resource_id)
            .unwrap()
            .paths,
    );
    let player_texture_atlas = TextureAtlas::from_grid(
        player_texture_handle,
        Vec2::new(24.0, 24.0),
        7,
        1,
        None,
        None,
    );
    let player_texture_atlas_handle = texture_atlases.add(player_texture_atlas);

    // Load enemy sprite sheet
    let enemy_resource_id =
        resource_system::Resource::create("assets/sara-cal.png".to_owned(), false).unwrap();
    let enemy_texture_handle = asset_server.load(
        resource_system::Resource::try_get(enemy_resource_id)
            .unwrap()
            .paths,
    );
    let enemy_texture_atlas = TextureAtlas::from_grid(
        enemy_texture_handle,
        Vec2::new(24.0, 24.0),
        7,
        1,
        None,
        None,
    );
    let enemy_texture_atlas_handle = texture_atlases.add(enemy_texture_atlas);

    // Player animation indices
    let player_animation_indices = AnimationIndices { first: 1, last: 6 };

    // Enemy animation indices
    let enemy_animation_indices = AnimationIndices { first: 1, last: 6 };

    commands.spawn(Camera2dBundle::default());

    // Spawn player entity
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: player_texture_atlas_handle,
            sprite: TextureAtlasSprite::new(player_animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        player_animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Player,
    ));

    // Spawn enemy entity
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: enemy_texture_atlas_handle,
            sprite: TextureAtlasSprite::new(enemy_animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        enemy_animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Enemy,
    ));
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;
