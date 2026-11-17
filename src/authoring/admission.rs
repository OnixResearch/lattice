//! Structural source admission through the selected Nickel parser, not token heuristics.
//! Until captured-map-only import resolution exists, source editing supports the
//! import-free cohort only. Retaining extra module bytes does not relax admission.

use nickel_lang_core::parser::ErrorTolerantParser as _;

pub fn require_import_free(source: &str) -> Result<(), &'static str> {
    let file = nickel_lang_core::files::Files::empty().add("<captured-workflow>", source.to_owned());
    let allocator = nickel_lang_core::ast::alloc::AstAlloc::new();
    let ast = nickel_lang_core::parser::grammar::TermParser::new()
        .parse_strict(&allocator, file, nickel_lang_core::parser::lexer::Lexer::new(source))
        .map_err(|_| "source syntax admission failed")?;
    if contains_import(&ast) {
        return Err("captured module import evaluation is unsupported; source preview requires import-free entry");
    }
    Ok(())
}

fn contains_import(ast: &nickel_lang_core::ast::Ast<'_>) -> bool {
    use nickel_lang_core::traverse::TraverseAlloc as _;
    ast.find_map(|term: &nickel_lang_core::ast::Ast<'_>| {
        matches!(&term.node, nickel_lang_core::ast::Node::Import(_)).then_some(())
    })
    .is_some()
}
