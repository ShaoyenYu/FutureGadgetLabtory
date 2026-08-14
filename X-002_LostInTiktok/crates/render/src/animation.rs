use bevy::prelude::*;

/// 角色与怪物动画状态机
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    #[default]
    Idle,
    Walk,
    Attack,
    Death,
}

/// 动画帧计时器
#[derive(Component, Debug, Clone)]
pub struct AnimationTimer {
    pub timer: Timer,
    pub loop_anim: bool,
    pub frame_count: usize,
    pub current_frame: usize,
}

impl Default for AnimationTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            loop_anim: true,
            frame_count: 4,
            current_frame: 0,
        }
    }
}

/// 动画播放与视觉缩放微动系统
pub fn animation_update_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &AnimationState, &mut Transform)>,
) {
    for (mut anim, state, mut transform) in query.iter_mut() {
        anim.timer.tick(time.delta());

        if anim.timer.just_finished() {
            anim.current_frame = (anim.current_frame + 1) % anim.frame_count;

            // 依据动画状态产生像素微动
            match state {
                AnimationState::Idle => {
                    let bob = (anim.current_frame as f32 * 0.5).sin() * 0.05;
                    transform.scale = Vec3::new(1.0, 1.0 + bob, 1.0);
                }
                AnimationState::Walk => {
                    let tilt = if anim.current_frame % 2 == 0 { 0.05 } else { -0.05 };
                    transform.rotation = Quat::from_rotation_z(tilt);
                    transform.scale = Vec3::ONE;
                }
                AnimationState::Attack => {
                    transform.scale = Vec3::new(1.15, 1.15, 1.0);
                }
                AnimationState::Death => {
                    transform.scale = Vec3::new(0.8, 0.8, 1.0);
                }
            }
        }
    }
}
