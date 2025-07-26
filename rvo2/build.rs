fn main() {
    println!("cargo:rerun-if-changed=src/rvo2/Agent.cc");
    println!("cargo:rerun-if-changed=src/rvo2/Agent.h");
    println!("cargo:rerun-if-changed=src/rvo2/Bridge.cpp");
    println!("cargo:rerun-if-changed=src/rvo2/Bridge.h");
    println!("cargo:rerun-if-changed=src/rvo2/Definitions.h");
    println!("cargo:rerun-if-changed=src/rvo2/Export.cc");
    println!("cargo:rerun-if-changed=src/rvo2/Export.h");
    println!("cargo:rerun-if-changed=src/rvo2/KdTree.cc");
    println!("cargo:rerun-if-changed=src/rvo2/KdTree.h");
    println!("cargo:rerun-if-changed=src/rvo2/Line.cc");
    println!("cargo:rerun-if-changed=src/rvo2/Line.h");
    println!("cargo:rerun-if-changed=src/rvo2/Obstacle.cc");
    println!("cargo:rerun-if-changed=src/rvo2/Obstacle.h");
    println!("cargo:rerun-if-changed=src/rvo2/RVO.h");
    println!("cargo:rerun-if-changed=src/rvo2/RVOSimulator.cc");
    println!("cargo:rerun-if-changed=src/rvo2/RVOSimulator.h");
    println!("cargo:rerun-if-changed=src/rvo2/Vector2.cc");
    println!("cargo:rerun-if-changed=src/rvo2/Vector2.h");

    cc::Build::new()
        .cpp(true)
        .file("src/rvo2/Agent.cc")
        .file("src/rvo2/Bridge.cpp")
        .file("src/rvo2/Export.cc")
        .file("src/rvo2/KdTree.cc")
        .file("src/rvo2/Line.cc")
        .file("src/rvo2/Obstacle.cc")
        .file("src/rvo2/RVOSimulator.cc")
        .file("src/rvo2/Vector2.cc")
        .include("src/rvo2")
        .flag_if_supported("/EHsc") // Windows MSVC 异常处理
        .flag_if_supported("/std:c++14")
        .compile("rvo2");
}
