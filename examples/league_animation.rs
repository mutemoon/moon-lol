use bevy::animation::AnimationClip;
use binrw::prelude::*;
use binrw::BinRead;
use moon_lol::render::AnimationFile;
use moon_lol::render::LeagueLoader;
use moon_lol::render::LeagueSkeleton;
use std::fs::File;
use std::io::BufReader;

// **************************************************************************
// * 主入口和错误处理
// **************************************************************************

fn main() {
    // 请将 "assets/your_animation_file.anm" 替换为你的文件路径
    // 这个路径是基于 C# 代码中的示例文件名
    let path = "assets/sruap_orderturret1_idle1.anm";
    println!("尝试解析文件: {}", path);

    let mut reader = BufReader::new(File::open(path).unwrap());
    let asset = AnimationFile::read(&mut reader).unwrap();

    // 使用 {:#?} 来进行格式化（pretty-print）输出
    match asset {
        AnimationFile::Compressed(modern) => {
            println!("解析成功，版本: Compressed");
            println!("{:#?}", modern.data.joint_hashes);
        }
        AnimationFile::Uncompressed(legacy) => {
            println!("解析成功，版本: Uncompressed");
            // println!("{:#?}", legacy.data);
            match legacy.data {
                moon_lol::render::UncompressedData::V3(uncompressed_data_v3) => {
                    println!("V3");
                }
                moon_lol::render::UncompressedData::V4(uncompressed_data_v4) => {
                    println!("V4");
                }
                moon_lol::render::UncompressedData::V5(uncompressed_data_v5) => {
                    println!("V5{:#?}", uncompressed_data_v5.joint_hashes);
                }
            }
        }
    }

    let clip = AnimationClip::default();

    let path = "assets/turret.skl";

    let mut reader = BufReader::new(File::open(path).unwrap());

    let skeleton = LeagueSkeleton::read(&mut reader).unwrap();

    println!(
        "{:#?}",
        skeleton
            .modern_data
            .joints
            .iter()
            .map(|v| LeagueLoader::compute_joint_hash(&v.name))
            .collect::<Vec<u32>>()
    );
}
