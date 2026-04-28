use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Reflect)]
pub enum LayerTransitionBehavior {
    Unaffected,
    TurnInvisibleDoesNotMatchNewLayerFilter,
    TurnVisibleDoesMatchNewLayerFilter,
}

impl From<u8> for LayerTransitionBehavior {
    fn from(value: u8) -> Self {
        match value {
            1 => LayerTransitionBehavior::TurnInvisibleDoesNotMatchNewLayerFilter,
            2 => LayerTransitionBehavior::TurnVisibleDoesMatchNewLayerFilter,
            _ => LayerTransitionBehavior::Unaffected,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect, Default)]
pub struct EnvironmentVisibility(u8);

impl EnvironmentVisibility {
    pub const NO_LAYER: Self = Self(0);
    pub const LAYER1: Self = Self(1 << 0);
    pub const LAYER2: Self = Self(1 << 1);
    pub const LAYER3: Self = Self(1 << 2);
    pub const LAYER4: Self = Self(1 << 3);
    pub const LAYER5: Self = Self(1 << 4);
    pub const LAYER6: Self = Self(1 << 5);
    pub const LAYER7: Self = Self(1 << 6);
    pub const LAYER8: Self = Self(1 << 7);
    pub const ALL_LAYERS: Self = Self(255);

    #[inline]
    pub fn bits(&self) -> u8 {
        self.0
    }

    #[inline]
    pub fn from_bits_truncate(bits: u8) -> Self {
        Self(bits)
    }

    #[inline]
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub fn intersects(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    #[inline]
    pub fn union(&self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub fn intersection(&self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[inline]
    pub fn difference(&self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[inline]
    pub fn toggle(&mut self, other: Self) {
        self.0 ^= other.0
    }
}
