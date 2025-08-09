#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum QualityFilter {
    All,
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

pub fn parse_quality_filter(val: u8) -> QualityFilter {
    match val {
        1 => QualityFilter::VeryLow,
        2 => QualityFilter::Low,
        4 => QualityFilter::Medium,
        8 => QualityFilter::High,
        16 => QualityFilter::VeryHigh,
        _ => QualityFilter::All,
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LayerTransitionBehavior {
    Unaffected,
    TurnInvisibleDoesNotMatchNewLayerFilter,
    TurnVisibleDoesMatchNewLayerFilter,
}

pub fn parse_layer_transition_behavior(val: u8) -> LayerTransitionBehavior {
    match val {
        1 => LayerTransitionBehavior::TurnInvisibleDoesNotMatchNewLayerFilter,
        2 => LayerTransitionBehavior::TurnVisibleDoesMatchNewLayerFilter,
        _ => LayerTransitionBehavior::Unaffected,
    }
}
