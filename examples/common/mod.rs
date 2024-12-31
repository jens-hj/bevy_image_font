use bevy::color::palettes::tailwind;
use bevy::color::Srgba;

pub(crate) const TEXT: &str = "Sphinx of black quartz, judge my vow!";
pub(crate) const FONT_WIDTH: usize = 5;

#[allow(dead_code)]
pub(crate) const RAINBOW: [Srgba; 7] = [
    tailwind::RED_300,
    tailwind::ORANGE_300,
    tailwind::YELLOW_300,
    tailwind::GREEN_300,
    tailwind::BLUE_300,
    tailwind::INDIGO_300,
    tailwind::VIOLET_300,
];
