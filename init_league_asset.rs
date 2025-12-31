pub fn init_league_asset(app: &mut App, asset_loader_registry: &mut AssetLoaderRegistry) {app.init_asset::<StaticMaterialDef>();
asset_loader_registry.register::<StaticMaterialDef>();
app.init_asset::<VfxSystemDefinitionData>();
asset_loader_registry.register::<VfxSystemDefinitionData>();
app.init_asset::<UiElementGroupButtonData>();
asset_loader_registry.register::<UiElementGroupButtonData>();
app.init_asset::<UiElementEffectAnimationData>();
asset_loader_registry.register::<UiElementEffectAnimationData>();
app.init_asset::<UiElementIconData>();
asset_loader_registry.register::<UiElementIconData>();
app.init_asset::<SpellObject>();
asset_loader_registry.register::<SpellObject>();
app.init_asset::<UiElementRegionData>();
asset_loader_registry.register::<UiElementRegionData>();
}
