use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 物品形状与旋转定义（支持俄罗斯方块等非规则形状）
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemShape {
    pub width: u8,
    pub height: u8,
    // 1D 数组模拟 2D 矩阵，行优先存储，true 代表占用
    pub mask: Vec<bool>,
}

impl ItemShape {
    pub fn new(width: u8, height: u8, mask: Vec<bool>) -> Self {
        let size = (width as usize) * (height as usize);
        let mut actual_mask = mask;
        if actual_mask.len() < size {
            actual_mask.resize(size, true);
        }
        Self {
            width,
            height,
            mask: actual_mask,
        }
    }

    pub fn single_cell() -> Self {
        Self {
            width: 1,
            height: 1,
            mask: vec![true],
        }
    }

    #[inline]
    pub fn get(&self, x: u8, y: u8) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let index = (y as usize) * (self.width as usize) + (x as usize);
        self.mask.get(index).copied().unwrap_or(false)
    }

    #[inline]
    pub fn set(&mut self, x: u8, y: u8, value: bool) {
        if x < self.width && y < self.height {
            let index = (y as usize) * (self.width as usize) + (x as usize);
            if index < self.mask.len() {
                self.mask[index] = value;
            }
        }
    }

    /// 顺时针旋转 90 度的矩阵转置算法
    /// (x, y) 映射为 (height - 1 - y, x)
    pub fn rotate_90(&mut self) {
        let old_w = self.width as usize;
        let old_h = self.height as usize;
        let new_w = old_h;
        let new_h = old_w;

        let mut new_mask = vec![false; new_w * new_h];

        for y in 0..old_h {
            for x in 0..old_w {
                let old_idx = y * old_w + x;
                let val = self.mask.get(old_idx).copied().unwrap_or(false);

                let new_x = old_h - 1 - y;
                let new_y = x;
                let new_idx = new_y * new_w + new_x;

                if new_idx < new_mask.len() {
                    new_mask[new_idx] = val;
                }
            }
        }

        self.width = new_w as u8;
        self.height = new_h as u8;
        self.mask = new_mask;
    }

    /// 逆时针旋转 90 度
    pub fn rotate_counter_clockwise(&mut self) {
        self.rotate_90();
        self.rotate_90();
        self.rotate_90();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_rotation() {
        // 1x3 vertical line
        let mut shape = ItemShape::new(1, 3, vec![true, true, true]);
        assert_eq!(shape.width, 1);
        assert_eq!(shape.height, 3);

        shape.rotate_90();
        assert_eq!(shape.width, 3);
        assert_eq!(shape.height, 1);
        assert_eq!(shape.mask, vec![true, true, true]);

        // 2x2 L shape
        // [true, false]
        // [true, true]
        let mut l_shape = ItemShape::new(2, 2, vec![true, false, true, true]);
        l_shape.rotate_90();
        // After 90 clockwise:
        // [true, true]
        // [true, false]
        assert_eq!(l_shape.get(0, 0), true);
        assert_eq!(l_shape.get(1, 0), true);
        assert_eq!(l_shape.get(0, 1), true);
        assert_eq!(l_shape.get(1, 1), false);
    }
}
