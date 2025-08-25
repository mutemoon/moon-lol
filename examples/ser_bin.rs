use std::fs::File;
use std::io::BufReader;

use binrw::BinRead;
use moon_lol::league::{from_entry, AnimationGraphData, PropFile};

fn main() {
    let path = "assets/skin41.bin";

    println!("尝试读取文件: {}", path);

    let file = File::open(path).unwrap();

    let prop_file = PropFile::read(&mut BufReader::new(file)).unwrap();

    let data = from_entry::<AnimationGraphData>(
        &prop_file
            .entries
            .iter()
            .find(|v| v.hash == 0xec079eac)
            .unwrap(),
    );
    println!("反序列化成功，结果: {:#?}", data);

    // let data = bin_deserializer::from_entry::<BarracksConfig>(
    //     &prop_file
    //         .entries
    //         .iter()
    //         .find(|v| v.hash == 0x147211fb)
    //         .unwrap()
    // );

    // println!("反序列化成功，结果: {:#?}", data);
}
