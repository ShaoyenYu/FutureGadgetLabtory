use crate::components::GridSlot;
use crate::item_shape::ItemShape;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// 核心背包组件 (挂载在 Player 或 BaseCamp Storage 上)
#[derive(Component, Debug, Clone)]
pub struct Inventory {
    pub max_width: u8,
    pub max_height: u8,
    pub unlocked_cells: HashSet<(u8, u8)>, // 支持非矩形的异形背包解锁
    pub slots: HashMap<(u8, u8), GridSlot>,
}

impl Inventory {
    /// 创建标准矩形背包
    pub fn new_rectangular(width: u8, height: u8) -> Self {
        let mut unlocked = HashSet::new();
        let mut slots = HashMap::new();
        for x in 0..width {
            for y in 0..height {
                unlocked.insert((x, y));
                slots.insert((x, y), GridSlot::default());
            }
        }
        Self {
            max_width: width,
            max_height: height,
            unlocked_cells: unlocked,
            slots,
        }
    }

    /// 创建带魔法格的进阶背包
    pub fn with_magic_slot(mut self, x: u8, y: u8, buff_id: &str) -> Self {
        if let Some(slot) = self.slots.get_mut(&(x, y)) {
            slot.magic_buff = Some(buff_id.to_string());
        }
        self
    }

    /// 解锁新的网格单元
    pub fn unlock_cell(&mut self, x: u8, y: u8) {
        if x < self.max_width && y < self.max_height {
            self.unlocked_cells.insert((x, y));
            self.slots.entry((x, y)).or_insert_with(GridSlot::default);
        }
    }

    /// 校验物品是否可以在指定位置放下（越界检查 + 解锁检查 + 重叠检查）
    pub fn can_place_item(
        &self,
        shape: &ItemShape,
        top_left: (u8, u8),
        ignore_entity: Option<Entity>,
    ) -> bool {
        let (tl_x, tl_y) = top_left;

        // 边界快速检查
        if tl_x + shape.width > self.max_width || tl_y + shape.height > self.max_height {
            return false;
        }

        // 逐格检查掩码
        for dy in 0..shape.height {
            for dx in 0..shape.width {
                if shape.get(dx, dy) {
                    let cell_x = tl_x + dx;
                    let cell_y = tl_y + dy;

                    // 必须是已解锁单元
                    if !self.unlocked_cells.contains(&(cell_x, cell_y)) {
                        return false;
                    }

                    // 检查是否被其他实体占据
                    if let Some(slot) = self.slots.get(&(cell_x, cell_y)) {
                        if let Some(existing_entity) = slot.item_entity {
                            if Some(existing_entity) != ignore_entity {
                                return false; // 重叠冲突
                            }
                        }
                    } else {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// 放置物品到背包中
    pub fn place_item(
        &mut self,
        entity: Entity,
        item_id: &str,
        shape: &ItemShape,
        top_left: (u8, u8),
    ) -> bool {
        if !self.can_place_item(shape, top_left, Some(entity)) {
            return false;
        }

        let (tl_x, tl_y) = top_left;
        for dy in 0..shape.height {
            for dx in 0..shape.width {
                if shape.get(dx, dy) {
                    let cell = (tl_x + dx, tl_y + dy);
                    let slot = self.slots.entry(cell).or_insert_with(GridSlot::default);
                    slot.item_entity = Some(entity);
                    slot.item_id = Some(item_id.to_string());
                }
            }
        }

        true
    }

    /// 从背包中移除指定物品实体，并返回其原先占据的左上角最小坐标
    pub fn remove_item(&mut self, entity: Entity) -> Option<(u8, u8)> {
        let mut min_pos: Option<(u8, u8)> = None;
        let mut found = false;

        for (pos, slot) in self.slots.iter_mut() {
            if slot.item_entity == Some(entity) {
                slot.item_entity = None;
                slot.item_id = None;
                found = true;

                min_pos = match min_pos {
                    None => Some(*pos),
                    Some((mx, my)) => Some((mx.min(pos.0), my.min(pos.1))),
                };
            }
        }

        if found {
            min_pos
        } else {
            None
        }
    }

    /// 搜寻第一个可容纳该形状的空格位（可用于一键自动整理/自动拾取入包）
    pub fn find_first_available_slot(&self, shape: &ItemShape) -> Option<(u8, u8)> {
        for y in 0..self.max_height {
            for x in 0..self.max_width {
                if self.can_place_item(shape, (x, y), None) {
                    return Some((x, y));
                }
            }
        }
        None
    }

    /// 搜寻自动入包（若原方向放不下，尝试旋转90/180/270度）
    pub fn find_available_slot_with_rotation(&self, shape: &mut ItemShape) -> Option<(u8, u8)> {
        for _ in 0..4 {
            if let Some(pos) = self.find_first_available_slot(shape) {
                return Some(pos);
            }
            shape.rotate_90();
        }
        None
    }

    /// 获取物品所覆盖的所有魔法格 Buff ID
    pub fn get_magic_buffs_under_item(&self, top_left: (u8, u8), shape: &ItemShape) -> Vec<String> {
        let mut buffs = Vec::new();
        let (tl_x, tl_y) = top_left;

        for dy in 0..shape.height {
            for dx in 0..shape.width {
                if shape.get(dx, dy) {
                    let cell = (tl_x + dx, tl_y + dy);
                    if let Some(slot) = self.slots.get(&cell) {
                        if let Some(buff) = &slot.magic_buff {
                            buffs.push(buff.clone());
                        }
                    }
                }
            }
        }
        buffs
    }

    /// 获取背包中所有独立物品实体与占据列表
    pub fn get_all_contained_entities(&self) -> HashSet<Entity> {
        self.slots
            .values()
            .filter_map(|s| s.item_entity)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_placement_and_overlap() {
        let mut inv = Inventory::new_rectangular(4, 4);
        let sword_shape = ItemShape::new(1, 3, vec![true, true, true]);
        let e1 = Entity::from_raw(10);
        let e2 = Entity::from_raw(20);

        // Place e1 at (0, 0)
        assert!(inv.place_item(e1, "sword", &sword_shape, (0, 0)));

        // Cannot place e2 at (0, 1) due to overlap
        assert!(!inv.place_item(e2, "sword", &sword_shape, (0, 1)));

        // Cannot place at (0, 2) due to out of bounds (height 3 from y=2 exceeds max_height 4)
        assert!(!inv.can_place_item(&sword_shape, (0, 2), None));

        // Can place at (1, 0)
        assert!(inv.place_item(e2, "sword", &sword_shape, (1, 0)));

        // Remove e1
        assert_eq!(inv.remove_item(e1), Some((0, 0)));

        // Now can place at (0, 0) again
        assert!(inv.can_place_item(&sword_shape, (0, 0), None));
    }

    #[test]
    fn test_magic_slots_buff_query() {
        let inv = Inventory::new_rectangular(4, 4).with_magic_slot(1, 1, "bonus_crit");
        let shape_2x2 = ItemShape::new(2, 2, vec![true, true, true, true]);

        let buffs = inv.get_magic_buffs_under_item((0, 0), &shape_2x2);
        assert_eq!(buffs, vec!["bonus_crit".to_string()]);
    }
}
