//! Procedurally baked pixel art.
//!
//! Every sprite in the game is written here as 16x16 string art plus a palette
//! that maps characters to colours. Baking the textures at startup keeps the
//! repository free of binary assets while still giving us real pixel art.
//!
//! Shared art + swapped palette is how the two snakes (and the full / empty
//! hearts) stay visually consistent.

use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::components::Player;

pub const ART_SIZE: usize = 16;

type Art = [&'static str; ART_SIZE];
type Palette = &'static [(char, [u8; 4])];

// ---------------------------------------------------------------------------
// Art
//
// '.' is transparent. Other characters are looked up in the palette; anything
// missing from the palette is treated as transparent too, which is what lets
// one piece of art serve several palettes.
// ---------------------------------------------------------------------------

/// Snake head, facing up. `E` eyes, `W` eye shine, `P` blush, `M` mouth.
///
/// The top is rounded but the bottom is nearly flat: that edge always faces the
/// neck, so a flat one keeps the snake reading as a single creature.
pub const SNAKE_HEAD: Art = [
    "....oooooooo....",
    "..ooLLLLLLLLoo..",
    ".oLLLLLLLLLLLLo.",
    "oLLLLLLLLLLLLLLo",
    "oLLLLLLLLLLLLLLo",
    "oLLWEELLLLWEELLo",
    "oLLEEELLLLEEELLo",
    "oLLEEELLLLEEELLo",
    "oLPPLLLLLLLLPPLo",
    "oLPPLLMLLMLLPPLo",
    "oLLLLLLMMLLLLLLo",
    "oLLLLLLLLLLLLLLo",
    "oDDDDDDDDDDDDDDo",
    "oDDDDDDDDDDDDDDo",
    "oDDDDDDDDDDDDDDo",
    ".oooooooooooooo.",
];

/// Same head with the eyes closed, swapped in for a moment now and then.
pub const SNAKE_HEAD_BLINK: Art = [
    "....oooooooo....",
    "..ooLLLLLLLLoo..",
    ".oLLLLLLLLLLLLo.",
    "oLLLLLLLLLLLLLLo",
    "oLLLLLLLLLLLLLLo",
    "oLLLLLLLLLLLLLLo",
    "oLLEEELLLLEEELLo",
    "oLLLLLLLLLLLLLLo",
    "oLPPLLLLLLLLPPLo",
    "oLPPLLMLLMLLPPLo",
    "oLLLLLLMMLLLLLLo",
    "oLLLLLLLLLLLLLLo",
    "oDDDDDDDDDDDDDDo",
    "oDDDDDDDDDDDDDDo",
    "oDDDDDDDDDDDDDDo",
    ".oooooooooooooo.",
];

/// Body segment: a diamond scale motif on a barely-rounded block, so segments
/// butt up against each other with only their outlines between them.
pub const SNAKE_BODY: Art = [
    ".oooooooooooooo.",
    "oLLLLLLLLLLLLLLo",
    "oLLLLLLLLLLLLLLo",
    "oLLLLLLBBLLLLLLo",
    "oLLLLLBBBBLLLLLo",
    "oLLLLBBBBBBLLLLo",
    "oLLLBBBBBBBBLLLo",
    "oLLLBBBBBBBBLLLo",
    "oLLLLBBBBBBLLLLo",
    "oLLLLLBBBBLLLLLo",
    "oLLLLLLBBLLLLLLo",
    "oLLLLLLLLLLLLLLo",
    "oDDDDDDDDDDDDDDo",
    "oDDDDDDDDDDDDDDo",
    "oDDDDDDDDDDDDDDo",
    ".oooooooooooooo.",
];

/// Tail, tapering towards the bottom of the art (rotated to point away from
/// the segment it is attached to).
pub const SNAKE_TAIL: Art = [
    ".oooooooooooooo.",
    "oLLLLLLLLLLLLLLo",
    "oLLLLLLLLLLLLLLo",
    "oLLLLLLBBLLLLLLo",
    "oLLLLLBBBBLLLLLo",
    "oLLLLLBBBBLLLLLo",
    ".oLLLLBBBBLLLLo.",
    ".oLLLLLBBLLLLLo.",
    "..oLLLLLLLLLLo..",
    "..oLLLLLLLLLLo..",
    "...oLLLLLLLLo...",
    "....oLLLLLLo....",
    "....oLDDDDLo....",
    ".....oDDDDo.....",
    "......oDDo......",
    ".......oo.......",
];

/// Apple: `s` stem, `g`/`G` leaf, `R`/`r` flesh, `W` shine.
pub const APPLE: Art = [
    "........o.......",
    "........s.......",
    "...ogggos.......",
    "..ogggGos.......",
    "...oGGos........",
    "....oooooooo....",
    "..ooRRRRRRRRoo..",
    ".oRWRRRRRRRRRRo.",
    "oRWWRRRRRRRRRrro",
    "oRWRRRRRRRRRRrro",
    "oRRRRRRRRRRRrrro",
    "oRRRRRRRRRRRrrro",
    ".oRRRRRRRRRrrro.",
    ".oRRRRRRRRrrrro.",
    "..ooRRRRrrrroo..",
    "....oooooooo....",
];

/// Bomb: `k`/`K` casing, `W` shine, `f` fuse, `F`/`Y` flame.
pub const BOMB: Art = [
    "..........YY....",
    ".........YFFY...",
    ".........YFFY...",
    "..........ff....",
    ".........ff.....",
    "......KKKKff....",
    "....KKkkkkKK....",
    "..KKkkWkkkkkKK..",
    ".KkkWWkkkkkkkkK.",
    ".KkkWkkkkkkkkkK.",
    ".Kkkkkkkkkkkkkk.",
    ".Kkkkkkkkkkkkkk.",
    ".Kkkkkkkkkkkkkk.",
    "..KKkkkkkkkkKK..",
    "....KKkkkkKK....",
    "......KKKK......",
];

/// Heart, used for the HP pips. Baked twice: filled and hollow.
pub const HEART: Art = [
    "................",
    "...oooo..oooo...",
    "..oHHHHooHHHHo..",
    ".oHWHHHHHHHHHHo.",
    "oHWWHHHHHHHHHHHo",
    "oHWHHHHHHHHHHHHo",
    "oHHHHHHHHHHHHHHo",
    "oHHHHHHHHHHHHHho",
    ".oHHHHHHHHHHhho.",
    "..oHHHHHHHHhho..",
    "...oHHHHHHhho...",
    "....oHHHHhho....",
    ".....oHHhho.....",
    "......ohho......",
    ".......oo.......",
    "................",
];

/// Tiny blossom used to decorate the arena floor. Deliberately low contrast:
/// it should read as texture, not as a pickup.
pub const FLOWER: Art = [
    "................",
    "................",
    "................",
    "................",
    "................",
    ".....pppppp.....",
    "....pppppppp....",
    "....pppyyppp....",
    "....pppyyppp....",
    "....pppppppp....",
    ".....pppppp.....",
    "................",
    "................",
    "................",
    "................",
    "................",
];

// ---------------------------------------------------------------------------
// Palettes
// ---------------------------------------------------------------------------

const P1_PALETTE: Palette = &[
    ('o', [46, 107, 69, 255]),
    ('L', [140, 235, 158, 255]),
    ('B', [95, 208, 124, 255]),
    ('D', [63, 168, 95, 255]),
    ('E', [43, 43, 60, 255]),
    ('W', [255, 255, 255, 255]),
    ('P', [255, 158, 181, 255]),
    ('M', [179, 67, 106, 255]),
];

const P2_PALETTE: Palette = &[
    ('o', [42, 78, 122, 255]),
    ('L', [169, 214, 255, 255]),
    ('B', [123, 184, 245, 255]),
    ('D', [79, 146, 220, 255]),
    ('E', [43, 43, 60, 255]),
    ('W', [255, 255, 255, 255]),
    ('P', [255, 179, 199, 255]),
    ('M', [179, 67, 106, 255]),
];

const APPLE_PALETTE: Palette = &[
    ('o', [142, 34, 51, 255]),
    ('R', [255, 107, 122, 255]),
    ('r', [224, 67, 88, 255]),
    ('W', [255, 255, 255, 255]),
    ('s', [122, 74, 43, 255]),
    ('g', [142, 217, 107, 255]),
    ('G', [99, 179, 72, 255]),
];

const BOMB_PALETTE: Palette = &[
    ('K', [43, 43, 60, 255]),
    ('k', [74, 74, 92, 255]),
    ('W', [255, 255, 255, 255]),
    ('f', [169, 116, 63, 255]),
    ('F', [255, 138, 61, 255]),
    ('Y', [255, 216, 77, 255]),
];

const HEART_PALETTE: Palette = &[
    ('o', [168, 30, 69, 255]),
    ('H', [255, 77, 109, 255]),
    ('h', [214, 43, 82, 255]),
    ('W', [255, 255, 255, 255]),
];

const HEART_EMPTY_PALETTE: Palette = &[
    ('o', [201, 185, 168, 255]),
    ('H', [239, 227, 210, 255]),
    ('h', [229, 214, 194, 255]),
    ('W', [247, 240, 228, 255]),
];

const FLOWER_PALETTE: Palette = &[
    ('p', [250, 226, 216, 255]),
    ('y', [255, 198, 206, 255]),
];

// ---------------------------------------------------------------------------
// Baking
// ---------------------------------------------------------------------------

fn lookup(palette: Palette, ch: char) -> [u8; 4] {
    palette
        .iter()
        .find(|(c, _)| *c == ch)
        .map(|(_, rgba)| *rgba)
        .unwrap_or([0, 0, 0, 0])
}

fn bake(art: &Art, palette: Palette) -> Image {
    let mut data = Vec::with_capacity(ART_SIZE * ART_SIZE * 4);
    for row in art.iter() {
        let mut chars = row.chars();
        for _ in 0..ART_SIZE {
            let rgba = match chars.next() {
                Some(ch) => lookup(palette, ch),
                None => [0, 0, 0, 0],
            };
            data.extend_from_slice(&rgba);
        }
    }

    Image::new(
        Extent3d {
            width: ART_SIZE as u32,
            height: ART_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

#[derive(Resource)]
pub struct PixelAssets {
    pub p1_head: Handle<Image>,
    pub p1_head_blink: Handle<Image>,
    pub p1_body: Handle<Image>,
    pub p1_tail: Handle<Image>,
    pub p2_head: Handle<Image>,
    pub p2_head_blink: Handle<Image>,
    pub p2_body: Handle<Image>,
    pub p2_tail: Handle<Image>,
    pub apple: Handle<Image>,
    pub bomb: Handle<Image>,
    pub heart: Handle<Image>,
    pub heart_empty: Handle<Image>,
    pub flower: Handle<Image>,
}

impl PixelAssets {
    pub fn head(&self, player: Player, blinking: bool) -> Handle<Image> {
        match (player, blinking) {
            (Player::One, false) => self.p1_head.clone(),
            (Player::One, true) => self.p1_head_blink.clone(),
            (Player::Two, false) => self.p2_head.clone(),
            (Player::Two, true) => self.p2_head_blink.clone(),
        }
    }

    pub fn body(&self, player: Player) -> Handle<Image> {
        match player {
            Player::One => self.p1_body.clone(),
            Player::Two => self.p2_body.clone(),
        }
    }

    pub fn tail(&self, player: Player) -> Handle<Image> {
        match player {
            Player::One => self.p1_tail.clone(),
            Player::Two => self.p2_tail.clone(),
        }
    }
}

pub fn setup_pixel_assets(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(PixelAssets {
        p1_head: images.add(bake(&SNAKE_HEAD, P1_PALETTE)),
        p1_head_blink: images.add(bake(&SNAKE_HEAD_BLINK, P1_PALETTE)),
        p1_body: images.add(bake(&SNAKE_BODY, P1_PALETTE)),
        p1_tail: images.add(bake(&SNAKE_TAIL, P1_PALETTE)),
        p2_head: images.add(bake(&SNAKE_HEAD, P2_PALETTE)),
        p2_head_blink: images.add(bake(&SNAKE_HEAD_BLINK, P2_PALETTE)),
        p2_body: images.add(bake(&SNAKE_BODY, P2_PALETTE)),
        p2_tail: images.add(bake(&SNAKE_TAIL, P2_PALETTE)),
        apple: images.add(bake(&APPLE, APPLE_PALETTE)),
        bomb: images.add(bake(&BOMB, BOMB_PALETTE)),
        heart: images.add(bake(&HEART, HEART_PALETTE)),
        heart_empty: images.add(bake(&HEART, HEART_EMPTY_PALETTE)),
        flower: images.add(bake(&FLOWER, FLOWER_PALETTE)),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A miscounted row would silently shift the whole sprite, so pin the width
    /// of every art row instead of hunting for it on screen.
    #[test]
    fn every_art_row_is_exactly_16_wide() {
        let arts: [(&str, &Art); 8] = [
            ("SNAKE_HEAD", &SNAKE_HEAD),
            ("SNAKE_HEAD_BLINK", &SNAKE_HEAD_BLINK),
            ("SNAKE_BODY", &SNAKE_BODY),
            ("SNAKE_TAIL", &SNAKE_TAIL),
            ("APPLE", &APPLE),
            ("BOMB", &BOMB),
            ("HEART", &HEART),
            ("FLOWER", &FLOWER),
        ];

        for (name, art) in arts {
            for (row_index, row) in art.iter().enumerate() {
                assert_eq!(
                    row.chars().count(),
                    ART_SIZE,
                    "{name} row {row_index} is `{row}`"
                );
            }
        }
    }

    /// Catches typos where art uses a character the palette never defines,
    /// which would punch a transparent hole in the sprite.
    #[test]
    fn art_characters_are_all_in_their_palette() {
        let pairs: [(&str, &Art, Palette); 9] = [
            ("head/p1", &SNAKE_HEAD, P1_PALETTE),
            ("head_blink/p1", &SNAKE_HEAD_BLINK, P1_PALETTE),
            ("body/p1", &SNAKE_BODY, P1_PALETTE),
            ("tail/p1", &SNAKE_TAIL, P1_PALETTE),
            ("head/p2", &SNAKE_HEAD, P2_PALETTE),
            ("apple", &APPLE, APPLE_PALETTE),
            ("bomb", &BOMB, BOMB_PALETTE),
            ("heart", &HEART, HEART_PALETTE),
            ("flower", &FLOWER, FLOWER_PALETTE),
        ];

        for (name, art, palette) in pairs {
            for row in art.iter() {
                for ch in row.chars() {
                    if ch == '.' {
                        continue;
                    }
                    assert!(
                        palette.iter().any(|(c, _)| *c == ch),
                        "{name}: character `{ch}` is missing from its palette"
                    );
                }
            }
        }
    }
}
