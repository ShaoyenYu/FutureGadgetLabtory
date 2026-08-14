use crate::pixel_generator::PixelAssets;
use bevy::prelude::*;
use soulforge_combat::components::Player;
use soulforge_core::constants::Z_BACKGROUND;

#[derive(Component)]
pub struct DungeonFloorTile {
    pub grid_pos: (i32, i32),
}

/// 围绕玩家动态生成与回收地牢石砖地面的系统
pub fn update_dungeon_tiles_system(
    player_query: Query<&Transform, With<Player>>,
    tile_query: Query<(Entity, &DungeonFloorTile, &Transform), Without<Player>>,
    pixel_assets: Res<PixelAssets>,
    mut commands: Commands,
) {
    let Ok(player_tf) = player_query.get_single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    let tile_size = 64.0; // 2x scaled 32px
    let radius_cells = 12; // 覆盖约 800x800 区域

    let center_cell_x = (player_pos.x / tile_size).floor() as i32;
    let center_cell_y = (player_pos.y / tile_size).floor() as i32;

    let mut existing = std::collections::HashSet::new();
    for (entity, tile, tf) in tile_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist > (radius_cells as f32 + 3.0) * tile_size {
            commands.entity(entity).despawn_recursive();
        } else {
            existing.insert(tile.grid_pos);
        }
    }

    for cy in (center_cell_y - radius_cells)..=(center_cell_y + radius_cells) {
        for cx in (center_cell_x - radius_cells)..=(center_cell_x + radius_cells) {
            if !existing.contains(&(cx, cy)) {
                let world_x = cx as f32 * tile_size;
                let world_y = cy as f32 * tile_size;

                commands.spawn((
                    SpriteBundle {
                        texture: pixel_assets.floor_tile.clone(),
                        transform: Transform::from_xyz(world_x, world_y, Z_BACKGROUND)
                            .with_scale(Vec3::splat(2.0)),
                        ..default()
                    },
                    DungeonFloorTile {
                        grid_pos: (cx, cy),
                    },
                ));
            }
        }
    }
}
