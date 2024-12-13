use tree_sitter::Node;
use tree_sitter_highlight::Highlighter;

pub struct TextAreaHighlighter {
    highlighter: Highlighter,
}

impl TextAreaHighlighter {
    pub fn new() -> Self {
        let mut highlighter = Highlighter::new();
        highlighter
            .parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .unwrap();
        Self { highlighter }
    }

    pub fn highlight(&mut self, content: String, lines: &mut Vec<(usize, String)>) {
        let Some(tree) = self.highlighter.parser.parse(content, None) else {
            return;
        };
        let program = tree.walk().node();
        assert_eq!(program.kind(), "program");
        let mut cursor = program.walk();
        for child in program.children(&mut cursor) {
            self.handle_program(child, lines)
        }
    }
    pub fn handle_program(&self, child: Node, lines: &mut Vec<(usize, String)>) {
        match child.kind() {
            "function_declaration" => self.handle_fn_decl(child, lines),
            _ => {}
        }
    }
    pub fn handle_fn_decl(&self, child: Node, lines: &mut Vec<(usize, String)>) {
        let mut cursor = child.walk();
        for child in child.children(&mut cursor) {
            match child.kind() {
                "function" => self.highlight_function(child, lines),
                _ => {}
            }
        }
    }
    pub fn highlight_function(&self, child: Node, lines: &mut Vec<(usize, String)>) {
        let range = child.range();
        lines[range.start_point.row]
            .1
            .insert_str(range.start_point.column, "\x1b[38;2;100;0;0m");
        lines[range.end_point.row]
            .1
            .insert_str(range.end_point.column, "\x1b[0m");
    }
}
