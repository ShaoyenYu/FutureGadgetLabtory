use crate::save_data::PersistentSaveData;
use bevy::prelude::*;
use rand::Rng;
use soulforge_combat::components::{AffixModifier, Affixes, Weapon};
use soulforge_core::events::ForgeRequestEvent;
use soulforge_data::loader::GameDatabase;
use soulforge_inventory::components::ItemComponent;

/// 铁匠铺重铸/锻造管线系统
pub fn forging_pipeline_system(
    mut forge_events: EventReader<ForgeRequestEvent>,
    mut weapon_query: Query<(Entity, &ItemComponent, Option<&mut Affixes>, Option<&mut Weapon>)>,
    material_query: Query<(Entity, &ItemComponent), Without<Weapon>>,
    db: Res<GameDatabase>,
    save_data: Res<PersistentSaveData>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    for event in forge_events.read() {
        // 1. 获取消耗的材料圣骸
        let Ok((mat_entity, mat_item)) = material_query.get(event.material_used) else {
            warn!("Forging failed: material entity not found");
            continue;
        };

        // 2. 获取目标武器
        let Ok((weapon_entity, weapon_item, maybe_affixes, _)) = weapon_query.get_mut(event.target_weapon) else {
            warn!("Forging failed: target weapon not found");
            continue;
        };

        let Some(mat_config) = db.items.items.get(&mat_item.item_id) else {
            warn!("Forging failed: material config not found in DB");
            continue;
        };

        if mat_config.possible_affixes.is_empty() {
            warn!("Forging failed: material {} has no possible affixes", mat_item.name);
            continue;
        }

        // 3. 计算权重并进行 RNG 掷骰子抽取词条
        let total_weight: u32 = mat_config.possible_affixes.iter().map(|a| a.weight).sum();
        let mut roll = rng.gen_range(0..total_weight.max(1));
        let mut chosen_affix = &mat_config.possible_affixes[0];

        for affix in &mat_config.possible_affixes {
            if roll < affix.weight {
                chosen_affix = affix;
                break;
            }
            roll -= affix.weight;
        }

        // 4. 将抽取结果转为 AffixModifier
        let modifier = match chosen_affix.affix_type.as_str() {
            "FlatDamage" => AffixModifier::FlatDamage(chosen_affix.value),
            "MultiplierDamage" => AffixModifier::MultiplierDamage(chosen_affix.value),
            "LifeSteal" => AffixModifier::LifeSteal(chosen_affix.value),
            "Pierce" => AffixModifier::Pierce(chosen_affix.value as u8),
            "Bleed" => AffixModifier::Bleed {
                chance: chosen_affix.value,
                dps: chosen_affix.secondary_value.unwrap_or(8.0),
                duration: chosen_affix.duration.unwrap_or(3.0),
            },
            "AttackSpeed" => AffixModifier::AttackSpeed(chosen_affix.value),
            "RangeBoost" => AffixModifier::RangeBoost(chosen_affix.value),
            _ => AffixModifier::FlatDamage(chosen_affix.value),
        };

        // 5. 挂载或更新武器的 Affixes 组件
        if let Some(mut affixes) = maybe_affixes {
            affixes.mods.push(modifier.clone());
        } else {
            commands.entity(weapon_entity).insert(Affixes {
                mods: vec![modifier.clone()],
            });
        }

        info!(
            "Forged successfully! Added {:?} to weapon {}",
            modifier, weapon_item.name
        );

        // 6. 销毁消耗掉的材料圣骸实体
        commands.entity(mat_entity).despawn_recursive();

        // 7. 保存持久化存档
        save_data.save_to_disk();
    }
}
