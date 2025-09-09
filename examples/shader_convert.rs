use regex::Regex;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "assets/shaders_extract/vs_quad_vs/BASE.vert";

    let source_code = fs::read_to_string(path).unwrap();

    let converted_code = convert(&source_code);

    fs::write(path.replace("shaders_extract", "shaders"), &converted_code).unwrap();

    println!("{}", converted_code);

    let path = "assets/shaders_extract/ps_quad_ps/BASE.frag";

    let source_code = fs::read_to_string(path).unwrap();

    let converted_code = convert_frag(&source_code);

    fs::write(path.replace("shaders_extract", "shaders"), &converted_code).unwrap();

    println!("{}", converted_code);

    Ok(())
}

fn convert(code: &str) -> String {
    // 1. 执行初始的、非上下文相关的替换。这里的顺序很重要。
    //    首先替换最具体的限定用法，然后才替换通用的变量名。
    let mut result = code.replace("_UniformsVertex.mProj", "camera_view.clip_from_world");
    result = result.replace("_UniformsVertex.vCamera", "camera_view.world_position");

    // 2. 对代码进行逐行处理以实现更复杂的结构性更改。
    let mut processed_lines = Vec::new();
    let mut in_uniforms_vertex_struct = false;
    let mut out_location_counter = 0;

    // 用于查找名为 ATTR<number> 的 `in` 变量并捕获其数字的正则表达式。
    let in_re = Regex::new(r"^\s*in\s+[\w\d]+\s+ATTR(\d+);").unwrap();

    for line in result.lines() {
        let trimmed_line = line.trim();

        // 特殊处理 uniform UniformsVertex _UniformsVertex; 这一行
        if trimmed_line == "uniform UniformsVertex _UniformsVertex;" {
            processed_lines.push(
                "layout(set = 2, binding = 0) uniform UniformsVertex uniforms_vertext;".to_string(),
            );
            continue;
        }

        // 处理我们想要修改的结构体定义的开始
        if trimmed_line.starts_with("struct UniformsVertex") {
            in_uniforms_vertex_struct = true;
            processed_lines.push(line.to_string());
            continue;
        }

        // 在结构体内部，过滤掉要删除的行
        if in_uniforms_vertex_struct {
            if trimmed_line.contains("mProj") || trimmed_line.contains("vCamera") {
                // 跳过这些行
                continue;
            }
            // 结构体定义的结束
            if trimmed_line.starts_with("};") {
                in_uniforms_vertex_struct = false;
            }
            processed_lines.push(line.to_string());
            continue;
        }

        // 为 `in` 变量添加 layout
        if let Some(caps) = in_re.captures(line) {
            if let Some(location_num) = caps.get(1) {
                let new_line = format!("layout(location = {}) {}", location_num.as_str(), line);
                processed_lines.push(new_line);
                continue;
            }
        }

        // 为 `out` 变量添加 layout
        if trimmed_line.starts_with("out ") {
            let new_line = format!("layout(location = {}) {}", out_location_counter, line);
            processed_lines.push(new_line);
            out_location_counter += 1;
            continue;
        }

        // 如果没有特殊处理，则直接添加该行
        processed_lines.push(line.to_string());
    }

    // 3. 注入 CameraView 结构体及其 uniform 声明
    let camera_view_definitions = r#"
struct CameraView {
    mat4 clip_from_world;
    mat4 unjittered_clip_from_world;
    mat4 world_from_clip;
    mat4 world_from_view;
    mat4 view_from_world;
    mat4 clip_from_view;
    mat4 view_from_clip;
    vec3 world_position;
};

layout(set = 0, binding = 0) uniform CameraView camera_view;"#;

    let mut final_lines = Vec::new();
    let mut injected = false;
    for line in processed_lines {
        let is_version_line = line.contains("#version 150"); // 检查原始版本号
        if is_version_line {
            final_lines.push("#version 450".to_string());
            final_lines.push(camera_view_definitions.to_string());
            injected = true;
        } else {
            final_lines.push(line);
        }
    }

    // 4. 全局替换变量名并将所有行合并成单个字符串返回
    let final_code = final_lines.join("\n");
    final_code.replace("_UniformsVertex", "uniforms_vertext")
}

/// Converts a GLSL 150 fragment shader string to GLSL 450.
fn convert_frag(code: &str) -> String {
    // 正则表达式，用于查找 "uniform sampler2D <name>;" 声明。
    let sampler_re = Regex::new(r"^\s*uniform\s+sampler2D\s+([a-zA-Z0-9_]+);").unwrap();

    // 存储采样器的原始名称，以便后续修改 texture() 调用。
    let mut sampler_names = Vec::new();

    // --- 第 1 部分：逐行处理着色器以转换声明部分 ---

    let mut processed_lines = Vec::new();
    let mut binding_counter = 1; // set=2 的绑定索引从 1 开始
    let mut in_location_counter = 0;
    let mut out_location_counter = 0;
    let mut main_declaration_found = false;

    for line in code.lines() {
        let trimmed_line = line.trim();

        // 跳过文件开头的空行
        if processed_lines.is_empty() && trimmed_line.is_empty() {
            continue;
        }

        // 规则: #version 150 -> #version 450
        if trimmed_line.starts_with("#version") {
            processed_lines.push("#version 450".to_string());
            continue;
        }

        // 规则: 转换 UniformsPixel uniform 块
        if trimmed_line.starts_with("uniform UniformsPixel") {
            let new_line = format!(
                "layout(set = 2, binding = {}) uniform UniformsPixel uniforms_pixel;",
                binding_counter
            );
            processed_lines.push(new_line);
            binding_counter += 1;
            continue;
        }

        // 规则: 将 sampler2D 分离为 texture2D 和 sampler
        if let Some(caps) = sampler_re.captures(trimmed_line) {
            // 可以安全地 unwrap，因为正则表达式中定义了一个捕获组。
            let name = caps.get(1).unwrap().as_str();
            sampler_names.push(name.to_string());

            let texture_line = format!(
                "layout(set = 2, binding = {}) uniform texture2D {}_texture;",
                binding_counter, name
            );
            processed_lines.push(texture_line);
            binding_counter += 1;

            let sampler_line = format!(
                "layout(set = 2, binding = {}) uniform sampler {}_sampler;",
                binding_counter, name
            );
            processed_lines.push(sampler_line);
            binding_counter += 1;
            continue;
        }

        // 规则: 为 'in' 变量添加 layout(location=...)
        if trimmed_line.starts_with("in ") {
            let new_line = format!("layout(location = {}) {}", in_location_counter, line);
            processed_lines.push(new_line);
            in_location_counter += 1;
            continue;
        }

        // 规则: 为 'out' 变量添加 layout(location=...)
        if trimmed_line.starts_with("out ") {
            let new_line = format!("layout(location = {}) {}", out_location_counter, line);
            processed_lines.push(new_line);
            out_location_counter += 1;
            continue;
        }

        // 如果没有匹配任何规则，则直接添加原始行。
        processed_lines.push(line.to_string());
    }

    // --- 第 2 部分：对已处理的代码执行全局替换 ---

    let mut final_code = processed_lines.join("\n");

    // 规则: 在函数体中将 _UniformsPixel 重命名为 uniforms_pixel
    final_code = final_code.replace("_UniformsPixel", "uniforms_pixel");

    // 规则: 更新 texture() 调用以使用合并的采样器构造函数
    for name in &sampler_names {
        // 使用正则表达式来稳健地处理 texture() 调用中可能存在的不同空格情况
        let texture_call_re = Regex::new(&format!(r"texture\s*\(\s*{}\s*,", name)).unwrap();
        let new_call = format!("texture(sampler2D({}_texture, {}_sampler),", name, name);

        final_code = texture_call_re
            .replace_all(&final_code, &new_call)
            .to_string();
    }

    final_code
}
