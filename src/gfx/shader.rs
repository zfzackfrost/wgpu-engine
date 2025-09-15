use std::collections::HashMap;

use crate::gfx::VertexInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderCode(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderPath(pub String);

#[derive(Debug, Clone)]
pub struct ShaderLib {
    sources: HashMap<ShaderPath, ShaderCode>,
}
impl ShaderLib {
    #[inline]
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }
    #[inline]
    pub fn insert(&mut self, path: &str, code: &str) {
        self.sources
            .insert(ShaderPath(path.into()), ShaderCode(code.into()));
    }
}
impl FromIterator<(ShaderPath, ShaderCode)> for ShaderLib {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (ShaderPath, ShaderCode)>>(iter: T) -> Self {
        Self {
            sources: HashMap::from_iter(iter),
        }
    }
}
impl IntoIterator for ShaderLib {
    type Item = (ShaderPath, ShaderCode);
    type IntoIter = std::collections::hash_map::IntoIter<ShaderPath, ShaderCode>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.sources.into_iter()
    }
}
impl Extend<(ShaderPath, ShaderCode)> for ShaderLib {
    #[inline]
    fn extend<T: IntoIterator<Item = (ShaderPath, ShaderCode)>>(&mut self, iter: T) {
        self.sources.extend(iter)
    }
}

impl Default for ShaderLib {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

fn default_shaderlib(lib: Option<&ShaderLib>, vertex_info: &dyn VertexInfo) -> ShaderLib {
    let mut lib = lib.cloned().unwrap_or(ShaderLib::new());
    lib.extend([
        (
            ShaderPath("struct/VertexBuf".into()),
            vertex_info.shader_code(),
        ), // VertexBuf
    ]);
    lib
}
pub fn make_shader_module(
    device: &wgpu::Device,
    code: &str,
    vertex_info: &dyn VertexInfo,
    lib: Option<&ShaderLib>,
    label: Option<&str>,
) -> wgpu::ShaderModule {
    let lib = default_shaderlib(lib, vertex_info);
    let code = proc_shader_code(code, Some(&lib));
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&code)),
    })
}
fn handle_include(out: &mut String, directive: &str, lib: Option<&ShaderLib>) {
    let directive = directive
        .strip_prefix('"')
        .expect("Missing '\"' in @include directive");
    let directive = directive
        .strip_suffix('"')
        .expect("Missing '\"' in @include directive");
    let Some(lib) = lib else {
        return;
    };
    if let Some(source) = lib.sources.get(&ShaderPath(directive.into())) {
        *out += &source.0;
    }
}
pub fn proc_shader_code(code: &str, lib: Option<&ShaderLib>) -> String {
    const SPECIAL_COMMENT: &str = "///";

    type DirectiveHandler = fn(&mut String, &str, Option<&ShaderLib>);
    let handlers: HashMap<&str, DirectiveHandler> = HashMap::from_iter([
        ("@include", handle_include as DirectiveHandler), //
    ]);

    let mut out = String::with_capacity(code.len());
    let n_lines = code.lines().count();
    for (i, l) in code.lines().enumerate() {
        if l.trim().starts_with(SPECIAL_COMMENT) {
            let l = l.trim().strip_prefix(SPECIAL_COMMENT).unwrap().trim_start();

            // Apply each handler
            for (token, handler) in handlers.iter() {
                if l.starts_with(token) {
                    let directive = l.strip_prefix(token).unwrap();
                    // Make sure directive is preceded by a space
                    if !directive.starts_with(" ") {
                        panic!("Missing space after directive!");
                    }
                    let directive = directive.trim();

                    // Execute handler
                    handler(&mut out, directive, lib);
                }
            }
        } else {
            out += l;
        }
        if i < n_lines - 1 {
            out += "\n";
        }
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn proc_shader_code_include_panic_1() {
        let lines = [
            "432 Hello",                      //
            "World 123",                      //
            r#"  /// @include "greeting   "#, //
            "goodbye",                        //
            "world",                          //
        ];
        let mut lib = ShaderLib::new();
        lib.insert("greeting", "HELLO\nEARTH");

        let input = lines.join("\n");
        let _ = proc_shader_code(&input, Some(&lib));
    }
    #[test]
    #[should_panic]
    fn proc_shader_code_include_panic_2() {
        let lines = [
            "432 Hello",                      //
            "World 123",                      //
            r#"  /// @include greeting"   "#, //
            "goodbye",                        //
            "world",                          //
        ];
        let mut lib = ShaderLib::new();
        lib.insert("greeting", "HELLO\nEARTH");

        let input = lines.join("\n");
        let _ = proc_shader_code(&input, Some(&lib));
    }
    #[test]
    #[should_panic]
    fn proc_shader_code_include_panic_3() {
        let lines = [
            "432 Hello",                     //
            "World 123",                     //
            r#"  /// @include greeting   "#, //
            "goodbye",                       //
            "world",                         //
        ];
        let mut lib = ShaderLib::new();
        lib.insert("greeting", "HELLO\nEARTH");

        let input = lines.join("\n");
        let _ = proc_shader_code(&input, Some(&lib));
    }

    #[test]
    #[should_panic]
    fn proc_shader_code_include_panic_4() {
        let lines = [
            "432 Hello",                      //
            "World 123",                      //
            r#"  /// @include"greeting"   "#, //
            "goodbye",                        //
            "world",                          //
        ];
        let mut lib = ShaderLib::new();
        lib.insert("greeting", "HELLO\nEARTH");

        let input = lines.join("\n");
        let _ = proc_shader_code(&input, Some(&lib));
    }

    #[test]
    fn proc_shader_code_include_1() {
        let lines = [
            "432 Hello",                       //
            "World 123",                       //
            r#"  /// @include "greeting"   "#, //
            "goodbye",                         //
            "world",                           //
        ];
        let mut lib = ShaderLib::new();
        lib.insert("greeting", "HELLO\nEARTH");

        let input = lines.join("\n");
        let output = proc_shader_code(&input, Some(&lib));

        let mut expected = Vec::from(lines);
        expected.remove(2);
        expected.insert(2, "HELLO");
        expected.insert(3, "EARTH");
        let expected = expected.join("\n");

        assert_eq!(expected, output);
    }

    #[test]
    fn proc_shader_code_noop() {
        let lines = [
            "432 Hello", //
            "World 123", //
            "goodbye",   //
            "world",     //
        ];
        let input = lines.join("\n");
        let output = proc_shader_code(&input, None);
        assert_eq!(input, output);
    }
}
