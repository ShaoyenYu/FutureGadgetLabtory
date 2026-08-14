use bevy::prelude::*;
use rand::Rng;
use soulforge_combat::components::Player;

/// 摄像机震屏控制器资源
#[derive(Resource, Debug, Clone, Default)]
pub struct CameraScreenShake {
    pub trauma: f32,
    pub max_offset: f32,
}

impl CameraScreenShake {
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }
}

/// 主游戏摄像机标记组件
#[derive(Component)]
pub struct MainGameCamera;

/// 摄像机跟随玩家与震屏平滑系统
pub fn camera_follow_system(
    time: Res<Time>,
    mut shake: ResMut<CameraScreenShake>,
    player_query: Query<&Transform, (With<Player>, Without<MainGameCamera>)>,
    mut camera_query: Query<&mut Transform, With<MainGameCamera>>,
) {
    let Ok(player_tf) = player_query.get_single() else {
        return;
    };
    let Ok(mut camera_tf) = camera_query.get_single_mut() else {
        return;
    };

    let target_pos = player_tf.translation.truncate();
    let dt = time.delta_seconds();

    // 平滑跟随
    let current_cam = camera_tf.translation.truncate();
    let smooth_cam = current_cam.lerp(target_pos, 10.0 * dt);

    // 震屏偏移
    let mut shake_offset = Vec2::ZERO;
    if shake.trauma > 0.001 {
        let mut rng = rand::thread_rng();
        let shake_amount = shake.trauma * shake.trauma;
        shake_offset = Vec2::new(
            rng.gen_range(-1.0..1.0) * shake.max_offset.max(8.0) * shake_amount,
            rng.gen_range(-1.0..1.0) * shake.max_offset.max(8.0) * shake_amount,
        );

        // 创伤自然衰减
        shake.trauma = (shake.trauma - dt * 2.5).max(0.0);
    }

    camera_tf.translation.x = smooth_cam.x + shake_offset.x;
    camera_tf.translation.y = smooth_cam.y + shake_offset.y;
}
