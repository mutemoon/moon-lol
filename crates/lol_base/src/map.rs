use bevy::prelude::*;

/// 地图路径管理
#[derive(Clone, Resource)]
pub struct MapPaths {
    pub name: String,
}

impl MapPaths {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// 场景 RON 资源路径（用于 AssetServer 加载）
    pub fn scene_ron(&self) -> String {
        format!("maps/{}/scene.ron", self.name)
    }

    /// 导航网格 Bin 资源路径
    pub fn navgrid_bin(&self) -> String {
        format!("maps/{}/navgrid.bin", self.name)
    }

    /// 兵营 RON 资源路径
    pub fn barracks_ron(&self, id: u32) -> String {
        format!("maps/{}/barracks/{}.ron", self.name, id)
    }

    /// 地图几何 GLB 资源路径
    pub fn mapgeo_glb(&self) -> String {
        format!("maps/{}/mapgeo.glb", self.name)
    }

    /// 场景 RON 导出路径（用于写入文件）
    pub fn scene_ron_export(&self) -> String {
        format!("assets/maps/{}/scene.ron", self.name)
    }

    /// 导航网格 Bin 导出路径
    pub fn navgrid_bin_export(&self) -> String {
        format!("assets/maps/{}/navgrid.bin", self.name)
    }

    /// 兵营 RON 导出路径
    pub fn barracks_ron_export(&self, id: u32) -> String {
        format!("assets/maps/{}/barracks/{}.ron", self.name, id)
    }

    /// 地图几何 GLB 导出路径
    pub fn mapgeo_glb_export(&self) -> String {
        format!("assets/maps/{}/mapgeo.glb", self.name)
    }

    /// 游戏内素材路径（读取 WAD 用）
    pub fn materials_path(&self) -> String {
        format!("Maps/MapGeometry/Map11/{}", self.name)
    }

    /// 游戏内素材 Bin 路径
    pub fn materials_bin_path(&self) -> String {
        format!("data/{}.materials.bin", self.materials_path())
    }

    /// 游戏内地图几何路径
    pub fn mapgeo_path(&self) -> String {
        format!("data/maps/mapgeometry/map11/{}.mapgeo", self.name)
    }
}

impl Default for MapPaths {
    fn default() -> Self {
        Self::new("sr_seasonal_map")
    }
}

impl std::fmt::Display for MapPaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
