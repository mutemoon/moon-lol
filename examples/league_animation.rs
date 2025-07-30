use bevy::animation::AnimationClip;
use binrw::io::Read;
use binrw::BinRead;
use binrw::prelude::*;
use moon_lol::render::AnimationFile;
use std::fs::File;
use std::io::BufReader;

// **************************************************************************
// * 主入口和错误处理
// **************************************************************************

fn main() {
    // 请将 "assets/your_animation_file.anm" 替换为你的文件路径
    // 这个路径是基于 C# 代码中的示例文件名
    let path = "assets/attack1.srt_2025_split1_solkimlayer.anm";
    println!("尝试解析文件: {}", path);

    let mut reader = BufReader::new(File::open(path).unwrap());
    let asset = AnimationFile::read(&mut reader);

    // 使用 {:#?} 来进行格式化（pretty-print）输出
    // println!("成功解析文件! 内容如下:\n{:#?}", asset);

    let clip = AnimationClip::default();
}
