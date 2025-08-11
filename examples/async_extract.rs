use moon_lol::league::LeagueLoader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    let loader = LeagueLoader::new(
        r"/mnt/c/Program Files (x86)/WeGameApps/英雄联盟/game",
        r"DATA/FINAL/Maps/Shipping/Map11.wad.client",
        r"data/maps/mapgeometry/map11/bloom.mapgeo",
    )?;
    #[cfg(windows)]
    let loader = LeagueLoader::new(
        r"C:\Program Files (x86)\WeGameApps\英雄联盟\game",
        r"DATA\FINAL\Maps\Shipping\Map11.wad.client",
        r"data/maps/mapgeometry/map11/bloom.mapgeo",
    )?;

    println!("开始异步处理配置文件...");
    let start = std::time::Instant::now();

    let _configs = loader.save_configs_async().await?;

    let duration = start.elapsed();
    println!("异步版本完成！耗时: {:?}", duration);

    Ok(())
}
