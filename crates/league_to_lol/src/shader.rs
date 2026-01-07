use regex::Regex;

struct ShaderRewriter {
    lines: Vec<String>,
    in_count: usize,
    out_count: usize,
    binding_count: usize,
}

impl ShaderRewriter {
    fn new(start_binding: usize) -> Self {
        Self {
            lines: Vec::new(),
            in_count: 0,
            out_count: 0,
            binding_count: start_binding,
        }
    }

    fn push(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    /// 处理通用行逻辑：版本号、precise、out 变量
    /// 返回 true 表示该行已被处理，无需继续后续逻辑
    fn try_process_common(&mut self, line: &str) -> bool {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // 保留空行但不处理
            if !self.lines.is_empty() {
                self.push("");
            }
            return true;
        }

        if trimmed.starts_with("precise ") {
            self.push(line.replace("precise ", ""));
            return true;
        }

        if trimmed.starts_with("#version") {
            self.push("#version 450");
            return true;
        }

        if trimmed.starts_with("out ") {
            self.push(format!("layout(location = {}) {}", self.out_count, line));
            self.out_count += 1;
            return true;
        }

        false
    }

    fn add_in_auto(&mut self, line: &str) {
        self.push(format!("layout(location = {}) {}", self.in_count, line));
        self.in_count += 1;
    }

    fn finish(self) -> String {
        self.lines.join("\n")
    }
}

pub fn convert_vert(code: &str) -> String {
    let replacements = [
        ("mProj", "camera_view.clip_from_world"),
        ("vCamera", "camera_view.world_position"),
        ("VIEW_PROJECTION_MATRIX", "camera_view.clip_from_world"),
    ];

    // 预处理：替换 uniform 引用
    let mut stage1_code = code.to_string();
    for (from, to) in replacements {
        stage1_code = stage1_code.replace(&format!("_UniformsVertex.{}", from), to);
    }

    let mut rewriter = ShaderRewriter::new(0);
    let attr_re = Regex::new(r"^\s*in\s+[\w\d]+\s+ATTR(\d+);").unwrap();
    let mut in_struct = false;

    for line in stage1_code.lines() {
        let trimmed = line.trim();

        if rewriter.try_process_common(line) {
            continue;
        }

        // 处理 UniformsVertex 结构体替换
        if trimmed == "uniform UniformsVertex _UniformsVertex;" {
            rewriter.push("layout(set = 3, binding = 0) uniform UniformsVertex uniforms_vertext;");
            continue;
        }

        if trimmed.starts_with("struct UniformsVertex") {
            in_struct = true;
            rewriter.push(line);
            continue;
        }

        if in_struct {
            if trimmed.starts_with("};") {
                in_struct = false;
            }
            // 如果该字段在替换列表中，则跳过（删除）
            if replacements.iter().any(|(k, _)| trimmed.contains(k)) {
                continue;
            }
            rewriter.push(line);
            continue;
        }

        // 处理带 ATTR 的输入
        if let Some(caps) = attr_re.captures(line) {
            let loc = caps.get(1).unwrap().as_str();
            rewriter.push(format!("layout(location = {}) {}", loc, line));
            continue;
        }

        rewriter.push(line);
    }

    // 注入 CameraView
    let camera_view_def = r#"
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

layout(set = 0, binding = 0) uniform CameraView camera_view;
"#;

    // 插入到 #version 之后
    let mut final_code = rewriter.finish();
    if let Some(idx) = final_code.find("#version 450") {
        let insert_pos = idx + "#version 450".len();
        final_code.insert_str(insert_pos, camera_view_def);
    }

    final_code.replace("_UniformsVertex", "uniforms_vertext")
}

pub fn convert_frag(code: &str) -> String {
    let mut rewriter = ShaderRewriter::new(2); // Pixel bindings 从 2 开始
    let sampler_re = Regex::new(r"^\s*uniform\s+sampler2D\s+([a-zA-Z0-9_]+);").unwrap();
    let mut sampler_names = Vec::new();

    for line in code.lines() {
        let trimmed = line.trim();

        if rewriter.try_process_common(line) {
            continue;
        }

        if trimmed.starts_with("uniform UniformsPixel") {
            rewriter.push("layout(set = 3, binding = 1) uniform UniformsPixel uniforms_pixel;");
            continue;
        }

        // 拆分 sampler2D
        if let Some(caps) = sampler_re.captures(trimmed) {
            let name = caps.get(1).unwrap().as_str();
            sampler_names.push(name.to_string());

            rewriter.push(format!(
                "layout(set = 3, binding = {}) uniform texture2D {}_texture;",
                rewriter.binding_count, name
            ));
            rewriter.binding_count += 1;

            rewriter.push(format!(
                "layout(set = 3, binding = {}) uniform sampler {}_sampler;",
                rewriter.binding_count, name
            ));
            rewriter.binding_count += 1;
            continue;
        }

        if trimmed.starts_with("in ") {
            rewriter.add_in_auto(line);
            continue;
        }

        rewriter.push(line);
    }

    let mut final_code = rewriter.finish();
    final_code = final_code.replace("_UniformsPixel", "uniforms_pixel");

    // 替换 texture 调用
    for name in sampler_names.clone() {
        let pattern = format!(r"texture\s*\(\s*{}\s*,", name);
        let replacement = format!("texture(sampler2D({}_texture, {}_sampler),", name, name);
        let re = Regex::new(&pattern).unwrap();
        final_code = re.replace_all(&final_code, &replacement).to_string();
    }

    // 替换 texelFetch 调用
    for name in sampler_names {
        let pattern = format!(r"texelFetch\s*\(\s*{}\s*,", name);
        let replacement = format!("texelFetch(sampler2D({}_texture, {}_sampler),", name, name);
        let re = Regex::new(&pattern).unwrap();
        final_code = re.replace_all(&final_code, &replacement).to_string();
    }

    final_code
}
