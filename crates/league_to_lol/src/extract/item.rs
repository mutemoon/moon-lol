use league_core::extract::ItemData;
use lol_base::item::ConfigItem;

pub fn extract_item_data(item_data: &ItemData) -> ConfigItem {
    ConfigItem {
        id: item_data.item_id,
        name: item_data.m_display_name.clone().unwrap_or_default(),
        description: item_data
            .m_item_data_client
            .m_description
            .clone()
            .unwrap_or_default(),
        price: item_data.price.unwrap_or_default(),
        icon_path: item_data.m_item_data_client.inventory_icon.clone(),
    }
}
