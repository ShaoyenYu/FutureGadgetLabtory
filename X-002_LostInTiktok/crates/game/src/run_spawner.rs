use bevy::prelude::*;
use soulforge_combat::components::*;
use soulforge_core::constants::*;
use soulforge_core::resources::*;
use soulforge_core::states::AppState;
use soulforge_data::loader::GameDatabase;
use soulforge_data::models::ItemType;
use soulforge_inventory::components::ItemComponent;
use soulforge_inventory::inventory::Inventory;
use soulforge_inventory::item_shape::ItemShape;
use soulforge_meta::save_data::PersistentSaveData;
use soulforge_meta::talent_systems::apply_talents_to_player;
use soulforge_render::pixel_generator::PixelAssets;

/// 局内游戏生成与初始化系统
pub fn spawn_run_world_system(
    mut commands: Commands,
    db: Res<GameDatabase>,
    save_data: Res<PersistentSaveData>,
    pixel_assets: Res<PixelAssets>,
    mut session: ResMut<RunSessionContext>,
    mut run_timer: ResMut<RunTimer>,
    mut next_state: ResMut<NextState<AppState>>,
    // 清理可能残留的旧局内实体
    old_entities: Query<Entity, Or<(With<Player>, With<Enemy>, With<Projectile>, With<MeleeSlash>, With<LootItem>, With<ExtractionPoint>)>>,
) {
    info!("Initializing new run session...");

    for entity in old_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // 重置全局会话统计与计时器
    *session = RunSessionContext::default();
    *run_timer = RunTimer::default();

    // 1. 初始化主角 CombatStats 并注入天赋加成
    let mut player_stats = CombatStats::default();
    apply_talents_to_player(&mut player_stats, &save_data, &db);

    // 2. 初始化主角背包 (5x4 网格，包含 1 个右上角魔法格)
    let mut inventory = Inventory::new_rectangular(6, 4)
        .with_magic_slot(5, 0, "magic_dmg_slot")
        .with_magic_slot(5, 1, "magic_bleed_slot");

    // 3. 生成初始武器
    let starting_weapon_id = &save_data.starting_weapon_id;
    let weapon_cfg = db.items.items.get(starting_weapon_id).cloned().unwrap_or_else(|| {
        db.items.items.get("sword_embryo_1").unwrap().clone()
    });

    let weapon_shape = ItemShape::new(
        weapon_cfg.shape.width,
        weapon_cfg.shape.height,
        weapon_cfg.shape.mask.clone(),
    );

    let weapon_entity = commands.spawn((
        ItemComponent {
            item_id: weapon_cfg.id.clone(),
            name: weapon_cfg.name.clone(),
            item_type: ItemType::Weapon,
            base_damage: weapon_cfg.base_damage,
            attack_rate: weapon_cfg.attack_rate,
            attack_range: weapon_cfg.attack_range,
            projectile_count: weapon_cfg.projectile_count,
            color_hex: weapon_cfg.color_hex.clone(),
            description: weapon_cfg.description.clone(),
            is_equipped: true,
            bound_to_player: true,
        },
        weapon_shape.clone(),
        Weapon {
            id: weapon_cfg.id.clone(),
            weapon_type: WeaponType::MeleeSlash,
            base_attack_rate: weapon_cfg.attack_rate,
            attack_range: weapon_cfg.attack_range,
            projectile_count: weapon_cfg.projectile_count,
            cooldown_timer: Timer::from_seconds(1.0 / weapon_cfg.attack_rate.max(0.1), TimerMode::Repeating),
            active: true,
        },
        Affixes::default(),
    )).id();

    // 将初始武器放置在背包 (0, 0)
    inventory.place_item(weapon_entity, &weapon_cfg.id, &weapon_shape, (0, 0));

    // 4. 生成主角实体
    commands.spawn((
        SpriteBundle {
            texture: pixel_assets.player.clone(),
            transform: Transform::from_xyz(0.0, 0.0, Z_PLAYER).with_scale(Vec3::splat(1.5)),
            ..default()
        },
        Player::default(),
        player_stats,
        inventory,
        soulforge_render::animation::AnimationState::Idle,
        soulforge_render::animation::AnimationTimer::default(),
    ));

    info!("Player spawned with starting weapon {}. Entering InRun state.", weapon_cfg.name);
    next_state.set(AppState::InRun);
}
