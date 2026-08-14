use bevy::prelude::*;

pub const ARENA_WIDTH: u32 = 32;
pub const ARENA_HEIGHT: u32 = 18;

/// Height of the HUD strip at the top of the window. The arena is laid out in
/// the space *below* it so nothing is ever hidden behind the score cards.
pub const HUD_HEIGHT: f32 = 104.0;

/// Breathing room kept around the board so its frame is never clipped.
pub const ARENA_MARGIN: f32 = 28.0;

/// Upper bound for the configurable starting HP (also the number of heart
/// icons pre-spawned per player card).
pub const MAX_HP: u32 = 10;

// ---------------------------------------------------------------------------
// Candy Kawaii palette. Stored as RGBA bytes so the same values feed both the
// pixel-art baker and the UI.
// ---------------------------------------------------------------------------

pub const COL_BG: [u8; 4] = [255, 249, 236, 255]; // cream backdrop
pub const COL_FRAME: [u8; 4] = [217, 188, 133, 255]; // arena frame
pub const COL_FRAME_DARK: [u8; 4] = [193, 160, 104, 255];
pub const COL_TILE_A: [u8; 4] = [246, 231, 193, 255];
pub const COL_TILE_B: [u8; 4] = [239, 220, 176, 255];
pub const COL_INK: [u8; 4] = [74, 59, 42, 255]; // text / outlines

pub const COL_P1: [u8; 4] = [95, 208, 124, 255]; // mint
pub const COL_P1_DARK: [u8; 4] = [46, 125, 79, 255];
pub const COL_P2: [u8; 4] = [123, 184, 245, 255]; // sky
pub const COL_P2_DARK: [u8; 4] = [42, 107, 176, 255];

pub const COL_DEAD: [u8; 4] = [176, 166, 150, 255];
pub const COL_CANDY_PINK: [u8; 4] = [255, 158, 181, 255];

pub fn col(c: [u8; 4]) -> Color {
    Color::srgba_u8(c[0], c[1], c[2], c[3])
}

pub fn col_a(c: [u8; 4], alpha: f32) -> Color {
    let mut color = col(c);
    color.set_alpha(alpha);
    color
}
