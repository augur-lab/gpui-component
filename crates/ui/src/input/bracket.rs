// vendor/gpui-component/crates/ui/src/input/bracket.rs

use tree_sitter::Node;

/// 括号对配置
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BracketPair {
    pub start: char,
    pub end: char,
    /// 是否自动闭合
    pub close: bool,
    /// 是否支持选中文本环绕
    pub surround: bool,
}

impl BracketPair {
    pub const fn new(start: char, end: char, close: bool, surround: bool) -> Self {
        Self { start, end, close, surround }
    }
}

/// 默认括号对配置
pub const DEFAULT_BRACKET_PAIRS: &[BracketPair] = &[
    BracketPair::new('{', '}', true, true),
    BracketPair::new('[', ']', true, true),
    BracketPair::new('(', ')', true, true),
    BracketPair::new('"', '"', true, true),
    BracketPair::new('\'', '\'', true, true),
];

/// 仅匹配高亮的括号对（不自动闭合）
/// 需要通过 is_in_template_context 判断上下文
pub const MATCH_ONLY_BRACKET_PAIRS: &[BracketPair] = &[
    BracketPair::new('<', '>', false, false),
];

/// 判断字符是否是括号的开始
pub fn is_bracket_start(ch: char) -> bool {
    DEFAULT_BRACKET_PAIRS.iter().any(|p| p.start == ch)
        || MATCH_ONLY_BRACKET_PAIRS.iter().any(|p| p.start == ch)
}

/// 判断字符是否是括号的结束
pub fn is_bracket_end(ch: char) -> bool {
    DEFAULT_BRACKET_PAIRS.iter().any(|p| p.end == ch)
        || MATCH_ONLY_BRACKET_PAIRS.iter().any(|p| p.end == ch)
}

/// 获取括号对配置
pub fn get_bracket_pair_for_start(ch: char) -> Option<&'static BracketPair> {
    DEFAULT_BRACKET_PAIRS.iter().find(|p| p.start == ch)
        .or_else(|| MATCH_ONLY_BRACKET_PAIRS.iter().find(|p| p.start == ch))
}

/// 获取括号对配置（通过结束字符）
pub fn get_bracket_pair_for_end(ch: char) -> Option<&'static BracketPair> {
    DEFAULT_BRACKET_PAIRS.iter().find(|p| p.end == ch)
        .or_else(|| MATCH_ONLY_BRACKET_PAIRS.iter().find(|p| p.end == ch))
}

/// 判断节点类型是否是注释或字符串
pub fn is_comment_or_string(kind: &str) -> bool {
    matches!(
        kind,
        "comment" |
        "line_comment" |
        "block_comment" |
        "string" |
        "string_literal" |
        "raw_string_literal" |
        "char_literal" |
        "character_literal" |
        "string_content" |
        "escape_sequence"
    )
}

/// 判断位置是否在模板上下文中
/// C++ 中 < 和 > 可能是模板参数、泛型、比较运算符、位移运算符等
/// 需要通过 tree-sitter 判断是否在 template_argument, template_declaration 等节点内
/// 注意：需要遍历所有祖先节点（包括非命名节点），因为 < > 字符本身可能不在命名节点内
pub fn is_in_template_context(tree: &tree_sitter::Tree, pos: usize) -> bool {
    let root = tree.root_node();
    // Try both methods to find the node at position
    let node = root
        .descendant_for_byte_range(pos, pos)
        .or_else(|| root.named_descendant_for_byte_range(pos, pos));

    if let Some(start_node) = node {
        let mut current: Option<Node> = Some(start_node);
        let mut node_types: Vec<&str> = Vec::new();
        while let Some(n) = current {
            let kind = n.kind();
            node_types.push(kind);
            // Check for template-related nodes (both named and unnamed)
            // C++ template-related nodes
            if kind == "template_argument_list"
                || kind == "template_parameter_list"
                || kind == "template_declaration"
                || kind == "explicit_specialization"
                || kind == "template_instantiation"
                || kind == "type_parameter"
                || kind == "required_parameter"
                || kind == "template_argument"
                || kind == "template_parameter"
                // Type-related nodes that may contain template arguments
                || kind == "generic_type"
                || kind == "scoped_type_identifier"
                || kind == "nested_type_identifier"
                || kind == "type_specifier"
                || kind == "type_identifier"
                || kind == "qualified_identifier"
                || kind == "decltype"
                // Declaration-related nodes
                || kind == "declaration"
                || kind == "declarator"
                || kind == "init_declarator"
                || kind == "field_declaration"
                || kind == "parameter_declaration"
                // Other C++ specific nodes that may contain templates
                || kind == "compound_literal_expression"
                || kind == "cast_expression"
                || kind == "sizeof_expression"
                || kind == "type_trait_expression"
                || kind == "sizeof_pack_expression"
                || kind == "alignof_expression"
                || kind == "noexcept_expression"
            {
                log::info!("[bracket] pos {} is in template context (matched: {}), node types: {:?}", pos, kind, node_types);
                return true;
            }
            current = n.parent();
        }
        log::info!("[bracket] pos {} NOT in template context, node types: {:?}", pos, node_types);
    } else {
        log::warn!("[bracket] pos {} - could not find node", pos);
    }
    false
}
