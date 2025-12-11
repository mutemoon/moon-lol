fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        println!("cargo:warning=WINDOWS 平台需要注册 resources.rc");
        embed_resource::compile("resources.rc", embed_resource::NONE);
    }
}
