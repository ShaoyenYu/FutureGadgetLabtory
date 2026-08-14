use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// 游戏核心像素贴图资源管理器
#[derive(Resource, Clone, Default)]
pub struct PixelAssets {
    pub player: Handle<Image>,
    pub soul_basic: Handle<Image>,
    pub flesh_brute: Handle<Image>,
    pub eye_stalker: Handle<Image>,
    pub slash_vfx: Handle<Image>,
    pub bullet_orb: Handle<Image>,
    pub floor_tile: Handle<Image>,
    pub extraction_cart: Handle<Image>,
    pub icon_sword: Handle<Image>,
    pub icon_dagger: Handle<Image>,
    pub icon_crossbow: Handle<Image>,
    pub icon_orb: Handle<Image>,
    pub icon_relic_blood: Handle<Image>,
    pub icon_relic_metal: Handle<Image>,
    pub icon_relic_venom: Handle<Image>,
    pub icon_potion: Handle<Image>,
    pub icon_crystal: Handle<Image>,
}

fn create_pixel_image(width: u32, height: u32, draw_fn: impl Fn(u32, u32) -> [u8; 4]) -> Image {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let color = draw_fn(x, y);
            data.extend_from_slice(&color);
        }
    }

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

pub fn generate_all_pixel_assets(mut images: ResMut<Assets<Image>>, mut pixel_assets: ResMut<PixelAssets>) {
    // 1. 玩家 32x32
    let player_img = create_pixel_image(32, 32, |x, y| {
        let dx = x as i32 - 16;
        let dy = y as i32 - 16;
        let d2 = dx * dx + dy * dy;
        if d2 <= 100 {
            if y < 12 {
                [60, 99, 130, 255] // 钢盔
            } else if y < 22 {
                [106, 176, 76, 255] // 战甲
            } else {
                [47, 53, 66, 255] // 战靴
            }
        } else if d2 <= 120 {
            [30, 39, 46, 255] // 轮廓描边
        } else {
            [0, 0, 0, 0]
        }
    });

    // 2. 基础怨魂 16x16
    let soul_img = create_pixel_image(16, 16, |x, y| {
        let dx = x as i32 - 8;
        let dy = y as i32 - 8;
        let d2 = dx * dx + dy * dy;
        if (x == 5 && y == 6) || (x == 10 && y == 6) {
            [235, 77, 75, 255] // 发光眼球
        } else if d2 <= 28 {
            [220, 221, 225, 230] // 幽灵体
        } else if d2 <= 38 {
            [113, 128, 147, 180] // 边缘虚光
        } else {
            [0, 0, 0, 0]
        }
    });

    // 3. 肉山巨兽 32x32
    let brute_img = create_pixel_image(32, 32, |x, y| {
        let dx = x as i32 - 16;
        let dy = y as i32 - 16;
        let d2 = dx * dx + dy * dy;
        if (x == 12 && y == 12) || (x == 20 && y == 12) {
            [255, 211, 42, 255] // 狂暴黄瞳
        } else if d2 <= 160 {
            [136, 84, 208, 255] // 重甲肌肉肉体
        } else if d2 <= 190 {
            [60, 40, 90, 255] // 外壳轮廓
        } else {
            [0, 0, 0, 0]
        }
    });

    // 4. 窥视之瞳 24x24
    let eye_img = create_pixel_image(24, 24, |x, y| {
        let dx = x as i32 - 12;
        let dy = y as i32 - 12;
        let d2 = dx * dx + dy * dy;
        if d2 <= 16 {
            [192, 57, 43, 255] // 猩红瞳孔
        } else if d2 <= 64 {
            [245, 246, 250, 255] // 眼白
        } else if d2 <= 85 {
            [155, 89, 182, 255] // 触手紫眶
        } else {
            [0, 0, 0, 0]
        }
    });

    // 5. 64x64 刀光弧形 VFX
    let slash_img = create_pixel_image(64, 64, |x, y| {
        let dx = x as f32 - 32.0;
        let dy = y as f32 - 32.0;
        let dist = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx);
        if dist >= 18.0 && dist <= 30.0 && angle > -1.2 && angle < 1.2 {
            let intensity = ((1.2 - angle.abs()) / 1.2 * 255.0) as u8;
            [255, 255, 255, intensity]
        } else {
            [0, 0, 0, 0]
        }
    });

    // 6. 8x8 能量弹丸
    let bullet_img = create_pixel_image(8, 8, |x, y| {
        let dx = x as i32 - 4;
        let dy = y as i32 - 4;
        if dx * dx + dy * dy <= 8 {
            [120, 224, 143, 255]
        } else {
            [0, 0, 0, 0]
        }
    });

    // 7. 32x32 地牢石砖地块
    let floor_img = create_pixel_image(32, 32, |x, y| {
        if x == 0 || y == 0 || x == 31 || y == 31 {
            [35, 39, 45, 255] // 砖缝暗色
        } else if (x + y) % 8 == 0 {
            [48, 54, 62, 255] // 砖纹微噪
        } else {
            [41, 46, 54, 255] // 灰暗地砖基色
        }
    });

    // 8. 48x48 撤离矿车/传送法阵
    let cart_img = create_pixel_image(48, 48, |x, y| {
        let dx = x as i32 - 24;
        let dy = y as i32 - 24;
        let d2 = dx * dx + dy * dy;
        if d2 <= 250 && d2 >= 180 {
            [46, 204, 113, 255] // 充能光环
        } else if d2 <= 140 {
            [52, 73, 94, 255] // 钢铁矿车基座
        } else {
            [0, 0, 0, 0]
        }
    });

    // 9. 24x24 图标系列
    let icon_sword_img = create_pixel_image(24, 24, |x, y| {
        if x == y || x == y + 1 || x + 1 == y {
            [220, 221, 225, 255]
        } else if x > 18 && y > 18 {
            [230, 126, 34, 255]
        } else {
            [0, 0, 0, 0]
        }
    });

    let icon_relic_img = create_pixel_image(24, 24, |x, y| {
        let dx = (x as i32 - 12).abs();
        let dy = (y as i32 - 12).abs();
        if dx + dy <= 8 {
            [231, 76, 60, 255] // 菱形红宝石
        } else {
            [0, 0, 0, 0]
        }
    });

    pixel_assets.player = images.add(player_img);
    pixel_assets.soul_basic = images.add(soul_img);
    pixel_assets.flesh_brute = images.add(brute_img);
    pixel_assets.eye_stalker = images.add(eye_img);
    pixel_assets.slash_vfx = images.add(slash_img);
    pixel_assets.bullet_orb = images.add(bullet_img);
    pixel_assets.floor_tile = images.add(floor_img);
    pixel_assets.extraction_cart = images.add(cart_img);
    pixel_assets.icon_sword = images.add(icon_sword_img.clone());
    pixel_assets.icon_dagger = images.add(icon_sword_img.clone());
    pixel_assets.icon_crossbow = images.add(icon_sword_img.clone());
    pixel_assets.icon_orb = images.add(icon_relic_img.clone());
    pixel_assets.icon_relic_blood = images.add(icon_relic_img.clone());
    pixel_assets.icon_relic_metal = images.add(icon_relic_img.clone());
    pixel_assets.icon_relic_venom = images.add(icon_relic_img.clone());
    pixel_assets.icon_potion = images.add(icon_relic_img.clone());
    pixel_assets.icon_crystal = images.add(icon_relic_img);
}
