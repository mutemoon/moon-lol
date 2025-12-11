use std::fs::read_to_string;
use std::str::FromStr;

use glsl_lang_pp::processor::event::{DirectiveKind, Event};
use glsl_lang_pp::processor::fs::StdProcessor;
use glsl_lang_pp::processor::nodes::{Define, DefineObject};
use glsl_lang_pp::processor::ProcessorState;

fn main() {
    let source = read_to_string("assets/shaders/particlesystem/quad.frag").unwrap();

    let target =
        read_to_string("assets/shaders_extract/particlesystem/quad/ps/ALPHA_TEST.frag").unwrap();

    let res = preprocess_glsl(&source, &["ALPHA_TEST"]);

    let is_same = verify_glsl(&res, &target);

    println!("结构相同: {}", is_same);
}

pub fn preprocess_glsl(source: &str, conditions: &[&str]) -> String {
    let mut processor = StdProcessor::default();

    let parsed = processor.parse_source(source, "input.glsl".as_ref());

    let mut state_builder = ProcessorState::builder();

    for cond in conditions {
        let parts: Vec<&str> = cond.splitn(2, ' ').collect();
        let name = parts[0];
        let value = if parts.len() > 1 { parts[1] } else { "1" };

        if let Ok(obj) = DefineObject::from_str(value) {
            state_builder = state_builder.definition(Define::object(name.into(), obj, false));
        }
    }

    let state = state_builder.finish();

    let mut output = String::new();

    for event in parsed.process(state) {
        if let Ok(event) = event {
            match event {
                Event::Token { token, masked, .. } => {
                    if !masked {
                        output.push_str(token.text());
                    }
                }
                Event::Directive { directive, masked } => {
                    if !masked {
                        match directive.kind() {
                            DirectiveKind::Version(_)
                            | DirectiveKind::Extension(_)
                            | DirectiveKind::Pragma(_)
                            | DirectiveKind::Line(_) => {
                                output.push_str(&directive.to_string());
                            }
                            _ => {
                                output.push('\n');
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    output
}

use glsl_lang::ast;
use glsl_lang::parse::DefaultParse;
use glsl_lang::visitor::{HostMut, Visit, VisitorMut};

struct Anonymizer;

impl VisitorMut for Anonymizer {
    fn visit_identifier(&mut self, ident: &mut ast::Identifier) -> Visit {
        ident.content.0 = "ident".into();
        Visit::Children
    }

    fn visit_type_name(&mut self, type_name: &mut ast::TypeName) -> Visit {
        type_name.content.0 = "type".into();
        Visit::Children
    }
}

pub fn verify_glsl(source: &str, target: &str) -> bool {
    let mut source_ast = match ast::TranslationUnit::parse(source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Source parse error: {:?}", e);
            return false;
        }
    };

    let mut target_ast = match ast::TranslationUnit::parse(target) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Target parse error: {:?}", e);
            return false;
        }
    };

    let mut anonymizer = Anonymizer;
    source_ast.visit_mut(&mut anonymizer);
    target_ast.visit_mut(&mut anonymizer);

    let source_decls = &source_ast.0;
    let target_decls = &target_ast.0;

    if source_decls.len() != target_decls.len() {
        println!(
            "Mismatch in number of declarations: source has {}, target has {}",
            source_decls.len(),
            target_decls.len()
        );
        return false;
    }

    for (i, (s, t)) in source_decls.iter().zip(target_decls.iter()).enumerate() {
        if s != t {
            println!("Mismatch at declaration #{}", i);
            println!("Source: {:#?}", s);
            println!("Target: {:#?}", t);
            return false;
        }
    }

    true
}
