use bevy::prelude::*;
use std::collections::HashMap;

/// 高性能 2D 空间哈希表，用于应对大量怪物的无 O(N^2) 邻域查询、碰撞检测与群聚排斥力计算
#[derive(Resource, Debug, Clone)]
pub struct SpatialHash2D {
    pub cell_size: f32,
    grid: HashMap<(i32, i32), Vec<(Entity, Vec2)>>,
}

impl Default for SpatialHash2D {
    fn default() -> Self {
        Self::new(32.0)
    }
}

impl SpatialHash2D {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: cell_size.max(4.0),
            grid: HashMap::with_capacity(1024),
        }
    }

    #[inline]
    fn to_cell_coords(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    pub fn insert(&mut self, entity: Entity, pos: Vec2) {
        let cell = self.to_cell_coords(pos);
        self.grid.entry(cell).or_default().push((entity, pos));
    }

    /// 查询指定半径圆内的所有实体与坐标
    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<(Entity, Vec2)> {
        let mut results = Vec::new();
        let r2 = radius * radius;
        let min_cell = self.to_cell_coords(center - Vec2::splat(radius));
        let max_cell = self.to_cell_coords(center + Vec2::splat(radius));

        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                if let Some(entities) = self.grid.get(&(cx, cy)) {
                    for &(entity, pos) in entities {
                        if center.distance_squared(pos) <= r2 {
                            results.push((entity, pos));
                        }
                    }
                }
            }
        }
        results
    }

    /// 查询指定半径内的实体列表
    pub fn query_radius_entities(&self, center: Vec2, radius: f32) -> Vec<Entity> {
        self.query_radius(center, radius)
            .into_iter()
            .map(|(e, _)| e)
            .collect()
    }

    /// 计算基于群聚排斥力（Boids Separation）的位移修正向量
    pub fn compute_separation(
        &self,
        self_entity: Entity,
        pos: Vec2,
        separation_radius: f32,
        max_neighbors: usize,
    ) -> Vec2 {
        let neighbors = self.query_radius(pos, separation_radius);
        let mut force = Vec2::ZERO;
        let mut count = 0;

        for (neighbor_entity, neighbor_pos) in neighbors {
            if neighbor_entity == self_entity {
                continue;
            }
            let diff = pos - neighbor_pos;
            let dist = diff.length();
            if dist > 0.001 && dist < separation_radius {
                let strength = (separation_radius - dist) / separation_radius;
                force += (diff / dist) * strength;
                count += 1;
                if count >= max_neighbors {
                    break;
                }
            }
        }

        if count > 0 {
            force / (count as f32)
        } else {
            Vec2::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_query_and_separation() {
        let mut spatial = SpatialHash2D::new(32.0);
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        spatial.insert(e1, Vec2::new(10.0, 10.0));
        spatial.insert(e2, Vec2::new(15.0, 10.0));
        spatial.insert(e3, Vec2::new(100.0, 100.0));

        let queried = spatial.query_radius(Vec2::new(10.0, 10.0), 10.0);
        assert_eq!(queried.len(), 2);

        let sep = spatial.compute_separation(e1, Vec2::new(10.0, 10.0), 20.0, 5);
        assert!(sep.x < 0.0, "Should push away from neighbor on the right");
    }
}
