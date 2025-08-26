use std::{collections::HashMap, hash::Hasher, io};

use bevy::transform::components::Transform;
use twox_hash::XxHash64;

use crate::{
    core::{ConfigGame, Team, CONFIG_PATH_GAME},
    league::{
        class_map_to_rust_code, extract_all_class, get_bin_path, get_hashes, merge_class_maps,
        save_struct_to_file, LeagueLoaderError, LeagueWadLoader, LeagueWadMapLoader, PropFile,
    },
};

const CHARACTER_WAD_PATHS: &[&str] = &[
    "DATA/FINAL/Champions/Hecarim.wad.client",
    "DATA/FINAL/Champions/Heimerdinger.wad.client",
    "DATA/FINAL/Champions/Hwei.wad.client",
    "DATA/FINAL/Champions/Illaoi.wad.client",
    "DATA/FINAL/Champions/Irelia.wad.client",
    "DATA/FINAL/Champions/Ivern.wad.client",
    "DATA/FINAL/Champions/Janna.wad.client",
    "DATA/FINAL/Champions/JarvanIV.wad.client",
    "DATA/FINAL/Champions/Jax.wad.client",
    "DATA/FINAL/Champions/Jayce.wad.client",
    "DATA/FINAL/Champions/Jhin.wad.client",
    "DATA/FINAL/Champions/Jinx.wad.client",
    "DATA/FINAL/Champions/Kaisa.wad.client",
    "DATA/FINAL/Champions/Kalista.wad.client",
    "DATA/FINAL/Champions/Karma.wad.client",
    "DATA/FINAL/Champions/Karthus.wad.client",
    "DATA/FINAL/Champions/Kassadin.wad.client",
    "DATA/FINAL/Champions/Katarina.wad.client",
    "DATA/FINAL/Champions/Kayle.wad.client",
    "DATA/FINAL/Champions/Kayn.wad.client",
    "DATA/FINAL/Champions/Kennen.wad.client",
    "DATA/FINAL/Champions/Khazix.wad.client",
    "DATA/FINAL/Champions/Kindred.wad.client",
    "DATA/FINAL/Champions/Kled.wad.client",
    "DATA/FINAL/Champions/KogMaw.wad.client",
    "DATA/FINAL/Champions/KSante.wad.client",
    "DATA/FINAL/Champions/Leblanc.wad.client",
    "DATA/FINAL/Champions/LeeSin.wad.client",
    "DATA/FINAL/Champions/Leona.wad.client",
    "DATA/FINAL/Champions/Lillia.wad.client",
    "DATA/FINAL/Champions/Lissandra.wad.client",
    "DATA/FINAL/Champions/Lucian.wad.client",
    "DATA/FINAL/Champions/Lulu.wad.client",
    "DATA/FINAL/Champions/Lux.wad.client",
    "DATA/FINAL/Champions/Malphite.wad.client",
    "DATA/FINAL/Champions/Malzahar.wad.client",
    "DATA/FINAL/Champions/Maokai.wad.client",
    "DATA/FINAL/Champions/MasterYi.wad.client",
    "DATA/FINAL/Champions/Mel.wad.client",
    "DATA/FINAL/Champions/Milio.wad.client",
    "DATA/FINAL/Champions/MissFortune.wad.client",
    "DATA/FINAL/Champions/MonkeyKing.wad.client",
    "DATA/FINAL/Champions/Mordekaiser.wad.client",
    "DATA/FINAL/Champions/Morgana.wad.client",
    "DATA/FINAL/Champions/Naafiri.wad.client",
    "DATA/FINAL/Champions/Nami.wad.client",
    "DATA/FINAL/Champions/Nasus.wad.client",
    "DATA/FINAL/Champions/Nautilus.wad.client",
    "DATA/FINAL/Champions/Neeko.wad.client",
    "DATA/FINAL/Champions/Nidalee.wad.client",
    "DATA/FINAL/Champions/Nilah.wad.client",
    "DATA/FINAL/Champions/Nocturne.wad.client",
    "DATA/FINAL/Champions/Nunu.wad.client",
    "DATA/FINAL/Champions/Olaf.wad.client",
    "DATA/FINAL/Champions/Orianna.wad.client",
    "DATA/FINAL/Champions/Ornn.wad.client",
    "DATA/FINAL/Champions/Pantheon.wad.client",
    "DATA/FINAL/Champions/Poppy.wad.client",
    "DATA/FINAL/Champions/Pyke.wad.client",
    "DATA/FINAL/Champions/Qiyana.wad.client",
    "DATA/FINAL/Champions/Quinn.wad.client",
    "DATA/FINAL/Champions/Rakan.wad.client",
    "DATA/FINAL/Champions/Rammus.wad.client",
    "DATA/FINAL/Champions/RekSai.wad.client",
    "DATA/FINAL/Champions/Rell.wad.client",
    "DATA/FINAL/Champions/Renata.wad.client",
    "DATA/FINAL/Champions/Renekton.wad.client",
    "DATA/FINAL/Champions/Rengar.wad.client",
    "DATA/FINAL/Champions/Riven.wad.client",
    "DATA/FINAL/Champions/Rumble.wad.client",
    "DATA/FINAL/Champions/Ryze.wad.client",
    "DATA/FINAL/Champions/Samira.wad.client",
    "DATA/FINAL/Champions/Sejuani.wad.client",
    "DATA/FINAL/Champions/Senna.wad.client",
    "DATA/FINAL/Champions/Seraphine.wad.client",
    "DATA/FINAL/Champions/Sett.wad.client",
    "DATA/FINAL/Champions/Shaco.wad.client",
    "DATA/FINAL/Champions/Shen.wad.client",
    "DATA/FINAL/Champions/Shyvana.wad.client",
    "DATA/FINAL/Champions/Singed.wad.client",
    "DATA/FINAL/Champions/Sion.wad.client",
    "DATA/FINAL/Champions/Sivir.wad.client",
    "DATA/FINAL/Champions/Skarner.wad.client",
    "DATA/FINAL/Champions/Smolder.wad.client",
    "DATA/FINAL/Champions/Sona.wad.client",
    "DATA/FINAL/Champions/Soraka.wad.client",
    "DATA/FINAL/Champions/Strawberry_Aurora.wad.client",
    "DATA/FINAL/Champions/Strawberry_Briar.wad.client",
    "DATA/FINAL/Champions/Strawberry_Illaoi.wad.client",
    "DATA/FINAL/Champions/Strawberry_Jinx.wad.client",
    "DATA/FINAL/Champions/Strawberry_Leona.wad.client",
    "DATA/FINAL/Champions/Strawberry_Riven.wad.client",
    "DATA/FINAL/Champions/Strawberry_Seraphine.wad.client",
    "DATA/FINAL/Champions/Strawberry_Xayah.wad.client",
    "DATA/FINAL/Champions/Strawberry_Yasuo.wad.client",
    "DATA/FINAL/Champions/Swain.wad.client",
    "DATA/FINAL/Champions/Sylas.wad.client",
    "DATA/FINAL/Champions/Syndra.wad.client",
    "DATA/FINAL/Champions/TahmKench.wad.client",
    "DATA/FINAL/Champions/Taliyah.wad.client",
    "DATA/FINAL/Champions/Talon.wad.client",
    "DATA/FINAL/Champions/Taric.wad.client",
    "DATA/FINAL/Champions/Teemo.wad.client",
    "DATA/FINAL/Champions/TFTChampion.wad.client",
    "DATA/FINAL/Champions/Thresh.wad.client",
    "DATA/FINAL/Champions/Tristana.wad.client",
    "DATA/FINAL/Champions/Trundle.wad.client",
    "DATA/FINAL/Champions/Tryndamere.wad.client",
    "DATA/FINAL/Champions/TwistedFate.wad.client",
    "DATA/FINAL/Champions/Twitch.wad.client",
    "DATA/FINAL/Champions/Udyr.wad.client",
    "DATA/FINAL/Champions/Urgot.wad.client",
    "DATA/FINAL/Champions/Varus.wad.client",
    "DATA/FINAL/Champions/Vayne.wad.client",
    "DATA/FINAL/Champions/Veigar.wad.client",
    "DATA/FINAL/Champions/Velkoz.wad.client",
    "DATA/FINAL/Champions/Vex.wad.client",
    "DATA/FINAL/Champions/Vi.wad.client",
    "DATA/FINAL/Champions/Viego.wad.client",
    "DATA/FINAL/Champions/Viktor.wad.client",
    "DATA/FINAL/Champions/Vladimir.wad.client",
    "DATA/FINAL/Champions/Volibear.wad.client",
    "DATA/FINAL/Champions/Warwick.wad.client",
    "DATA/FINAL/Champions/Xayah.wad.client",
    "DATA/FINAL/Champions/Xerath.wad.client",
    "DATA/FINAL/Champions/XinZhao.wad.client",
    "DATA/FINAL/Champions/Yasuo.wad.client",
    "DATA/FINAL/Champions/Yone.wad.client",
    "DATA/FINAL/Champions/Yorick.wad.client",
    "DATA/FINAL/Champions/Yunara.wad.client",
    "DATA/FINAL/Champions/Yuumi.wad.client",
    "DATA/FINAL/Champions/Zac.wad.client",
    "DATA/FINAL/Champions/Zed.wad.client",
    "DATA/FINAL/Champions/Zeri.wad.client",
    "DATA/FINAL/Champions/Ziggs.wad.client",
    "DATA/FINAL/Champions/Zilean.wad.client",
    "DATA/FINAL/Champions/Zoe.wad.client",
    "DATA/FINAL/Champions/Zyra.wad.client",
    "DATA/FINAL/Champions/Aatrox.wad.client",
    "DATA/FINAL/Champions/Ahri.wad.client",
    "DATA/FINAL/Champions/Akali.wad.client",
    "DATA/FINAL/Champions/Akshan.wad.client",
    "DATA/FINAL/Champions/Alistar.wad.client",
    "DATA/FINAL/Champions/Ambessa.wad.client",
    "DATA/FINAL/Champions/Amumu.wad.client",
    "DATA/FINAL/Champions/Anivia.wad.client",
    "DATA/FINAL/Champions/Annie.wad.client",
    "DATA/FINAL/Champions/Aphelios.wad.client",
    "DATA/FINAL/Champions/Ashe.wad.client",
    "DATA/FINAL/Champions/AurelionSol.wad.client",
    "DATA/FINAL/Champions/Aurora.wad.client",
    "DATA/FINAL/Champions/Azir.wad.client",
    "DATA/FINAL/Champions/Bard.wad.client",
    "DATA/FINAL/Champions/Belveth.wad.client",
    "DATA/FINAL/Champions/Blitzcrank.wad.client",
    "DATA/FINAL/Champions/Brand.wad.client",
    "DATA/FINAL/Champions/Braum.wad.client",
    "DATA/FINAL/Champions/Briar.wad.client",
    "DATA/FINAL/Champions/Caitlyn.wad.client",
    "DATA/FINAL/Champions/Camille.wad.client",
    "DATA/FINAL/Champions/Cassiopeia.wad.client",
    "DATA/FINAL/Champions/Chogath.wad.client",
    "DATA/FINAL/Champions/Corki.wad.client",
    "DATA/FINAL/Champions/Darius.wad.client",
    "DATA/FINAL/Champions/Diana.wad.client",
    "DATA/FINAL/Champions/Draven.wad.client",
    "DATA/FINAL/Champions/DrMundo.wad.client",
    "DATA/FINAL/Champions/Ekko.wad.client",
    "DATA/FINAL/Champions/Elise.wad.client",
    "DATA/FINAL/Champions/Evelynn.wad.client",
    "DATA/FINAL/Champions/Ezreal.wad.client",
    "DATA/FINAL/Champions/FiddleSticks.wad.client",
    "DATA/FINAL/Champions/Fiora.wad.client",
    "DATA/FINAL/Champions/Fizz.wad.client",
    "DATA/FINAL/Champions/Galio.wad.client",
    "DATA/FINAL/Champions/Gangplank.wad.client",
    "DATA/FINAL/Champions/Garen.wad.client",
    "DATA/FINAL/Champions/Gnar.wad.client",
    "DATA/FINAL/Champions/Gragas.wad.client",
    "DATA/FINAL/Champions/Graves.wad.client",
    "DATA/FINAL/Champions/Gwen.wad.client",
];

pub struct LeagueLoader {
    pub root_dir: String,
    pub wads: Vec<LeagueWadLoader>,
    pub map_loader: LeagueWadMapLoader,
}

impl LeagueLoader {
    pub fn new(root_dir: &str, map: &str) -> Result<LeagueLoader, LeagueLoaderError> {
        let loader = LeagueWadLoader::from_relative_path(
            root_dir,
            "DATA/FINAL/Maps/Shipping/Map11.wad.client",
        )?;

        let map_loader = LeagueWadMapLoader::from_loader(loader, map)?;

        Ok(LeagueLoader {
            root_dir: root_dir.to_string(),
            wads: vec![],
            map_loader,
        })
    }

    pub fn full(root_dir: &str, map: &str) -> Result<LeagueLoader, LeagueLoaderError> {
        let loader = LeagueWadLoader::from_relative_path(
            root_dir,
            "DATA/FINAL/Maps/Shipping/Map11.wad.client",
        )?;

        let map_loader = LeagueWadMapLoader::from_loader(loader, map)?;

        Ok(LeagueLoader {
            root_dir: root_dir.to_string(),
            wads: CHARACTER_WAD_PATHS
                .iter()
                .map(|path| LeagueWadLoader::from_relative_path(root_dir, path).unwrap())
                .collect::<Vec<LeagueWadLoader>>(),
            map_loader,
        })
    }

    pub async fn extract_all_classes(
        &self,
        file_paths: &[&str],
        hash_paths: &[&str],
    ) -> Result<String, LeagueLoaderError> {
        let mut class_map = HashMap::new();
        for (i, file_path) in file_paths.iter().enumerate() {
            println!("{:?}/{:?}", i, file_paths.len());
            let Ok(bin) = self.get_prop_bin_by_path(file_path) else {
                continue;
            };
            let bin_class_map = extract_all_class(&bin).await?;
            merge_class_maps(&mut class_map, bin_class_map);
        }

        let hashes = get_hashes(hash_paths);

        let rust_code = class_map_to_rust_code(&mut class_map, &hashes).await?;

        Ok(rust_code)
    }

    pub fn get_prop_bin_by_path(&self, path: &str) -> Result<PropFile, LeagueLoaderError> {
        if let Ok(bin) = self.map_loader.wad_loader.get_prop_bin_by_path(path) {
            return Ok(bin);
        }

        for wad_loader in &self.wads {
            if let Ok(bin) = wad_loader.get_prop_bin_by_path(path) {
                return Ok(bin);
            }
        }

        Err(LeagueLoaderError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "Prop file not found",
        )))
    }

    pub async fn save_legends(&self, champion: &str, skin: &str) -> Result<(), LeagueLoaderError> {
        let wad_relative_path = format!("DATA/FINAL/Champions/{}.wad.client", champion);

        let loader = LeagueWadLoader::from_relative_path(&self.root_dir, &wad_relative_path)?;

        // let character_record_path = format!("Characters/{}/CharacterRecords/Root", character);

        let skin_path: String = format!("Characters/{}/Skins/{}", champion, skin);

        // let character_record = wad_loader.load_character_record(&character_record_path);

        let skin = loader.save_environment_object(&skin_path).await?;

        let config_path = get_bin_path(CONFIG_PATH_GAME);

        let config_game = ConfigGame {
            legends: vec![(Transform::default(), Team::Order, skin)],
        };

        save_struct_to_file(&config_path, &config_game).await?;

        Ok(())
    }

    pub fn hash_wad(s: &str) -> u64 {
        let mut h = XxHash64::with_seed(0);
        h.write(s.to_ascii_lowercase().as_bytes());
        h.finish()
    }

    pub fn hash_bin(s: &str) -> u32 {
        s.to_ascii_lowercase().bytes().fold(0x811c9dc5_u32, |h, b| {
            (h ^ b as u32).wrapping_mul(0x01000193)
        })
    }

    pub fn hash_joint(s: &str) -> u32 {
        let mut hash = 0u32;
        for b in s.to_ascii_lowercase().bytes() {
            hash = (hash << 4) + (b as u32);
            let high = hash & 0xf0000000;
            if high != 0 {
                hash ^= high >> 24;
            }
            hash &= !high;
        }
        hash
    }
}
