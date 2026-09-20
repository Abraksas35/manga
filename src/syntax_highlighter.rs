//! Syntax highlighter module - Tree-sitter based syntax highlighting

use tree_sitter::{Parser, Language, Query, QueryCursor};
use std::collections::HashMap;

/// Color scheme for syntax highlighting
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub keyword: u32,      // ARGB format
    pub function: u32,
    pub type_name: u32,
    pub string: u32,
    pub number: u32,
    pub comment: u32,
    pub operator: u32,
    pub variable: u32,
    pub constant: u32,
    pub parameter: u32,
    pub method: u32,
    pub default: u32,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            keyword: 0xFF569CD6,      // Blue
            function: 0xFFDCDCAA,     // Yellow
            type_name: 0xFF4EC9B0,    // Cyan
            string: 0xFFCE9178,       // Orange/Brown
            number: 0xFFB5CEA8,       // Light Green
            comment: 0xFF6A9955,      // Green
            operator: 0xFFD4D4D4,     // Light Gray
            variable: 0xFF9CDCFE,     // Light Blue
            constant: 0xFF4FC1FF,     // Bright Blue
            parameter: 0xFF9CDCFE,    // Light Blue
            method: 0xFFDCDCAA,       // Yellow
            default: 0xFFD4D4D4,      // Light Gray
        }
    }
}

/// Highlighted text segment
#[derive(Debug, Clone)]
pub struct HighlightedSegment {
    /// Start character index
    pub start: usize,
    /// End character index
    pub end: usize,
    /// Color in ARGB format
    pub color: u32,
}

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Cpp,
    C,
    Unknown,
}

impl SupportedLanguage {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Self::Rust,
            "py" => Self::Python,
            "js" | "jsx" => Self::JavaScript,
            "ts" | "tsx" => Self::TypeScript,
            "cpp" | "cc" | "cxx" | "h" | "hpp" => Self::Cpp,
            "c" => Self::C,
            _ => Self::Unknown,
        }
    }

    /// Get tree-sitter language for the supported language
    pub fn get_language(&self) -> Option<Language> {
        match self {
            Self::Rust => Some(tree_sitter_rust::language()),
            Self::Python => Some(tree_sitter_python::language()),
            Self::JavaScript => Some(tree_sitter_javascript::language()),
            Self::TypeScript => Some(tree_sitter_typescript::language_tsx()),
            Self::Cpp => Some(tree_sitter_cpp::language()),
            Self::C => Some(tree_sitter_cpp::language()), // Use C++ parser for C
            Self::Unknown => None,
        }
    }
}

/// Syntax highlighter
pub struct SyntaxHighlighter {
    /// Parser instance
    parser: Parser,
    /// Current language
    current_language: Option<SupportedLanguage>,
    /// Color scheme
    color_scheme: ColorScheme,
    /// Cache of queries for each language
    queries: HashMap<SupportedLanguage, Query>,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter
    pub fn new() -> Self {
        let mut parser = Parser::new();
        
        Self {
            parser,
            current_language: None,
            color_scheme: ColorScheme::default(),
            queries: HashMap::new(),
        }
    }

    /// Set the language for highlighting
    pub fn set_language(&mut self, lang: SupportedLanguage) -> Result<(), &'static str> {
        if let Some(tree_lang) = lang.get_language() {
            self.parser.set_language(tree_lang).map_err(|_| "Failed to set language")?;
            self.current_language = Some(lang);
            
            // Load query for this language if not already loaded
            if !self.queries.contains_key(&lang) {
                let query_str = Self::get_query_for_language(lang);
                if let Ok(query) = Query::new(&tree_lang, query_str) {
                    self.queries.insert(lang, query);
                }
            }
            
            Ok(())
        } else {
            Err("Unsupported language")
        }
    }

    /// Set language from file extension
    pub fn set_language_from_extension(&mut self, ext: &str) -> Result<(), &'static str> {
        let lang = SupportedLanguage::from_extension(ext);
        self.set_language(lang)
    }

    /// Get query string for a language
    fn get_query_for_language(lang: SupportedLanguage) -> &'static str {
        match lang {
            SupportedLanguage::Rust => r#"
                (line_comment) @comment
                (block_comment) @comment
                (string_literal) @string
                (raw_string_literal) @string
                (integer_literal) @number
                (float_literal) @number
                (call_expression function: (identifier) @function)
                (method_invocation name: (field_identifier) @method)
                (struct_item name: (type_identifier) @type)
                (enum_item name: (type_identifier) @type)
                (trait_item name: (type_identifier) @type)
                (impl_trait_type trait: (type_identifier) @type)
                (primitive_type) @type
                (self) @variable
                (parameter pattern: (identifier) @parameter)
                (let_declaration pattern: (identifier) @variable)
                (const_item) @constant
                (static_item) @constant
                (closure_parameters) @parameter
                "fn" @keyword
                "let" @keyword
                "mut" @keyword
                "if" @keyword
                "else" @keyword
                "match" @keyword
                "for" @keyword
                "while" @keyword
                "loop" @keyword
                "return" @keyword
                "break" @keyword
                "continue" @keyword
                "in" @keyword
                "use" @keyword
                "mod" @keyword
                "pub" @keyword
                "struct" @keyword
                "enum" @keyword
                "trait" @keyword
                "impl" @keyword
                "type" @keyword
                "where" @keyword
                "async" @keyword
                "await" @keyword
                "unsafe" @keyword
                "extern" @keyword
                "crate" @keyword
                "super" @keyword
                "Self" @type
                "bool" @type
                "char" @type
                "str" @type
                "i8" @type
                "i16" @type
                "i32" @type
                "i64" @type
                "i128" @type
                "isize" @type
                "u8" @type
                "u16" @type
                "u32" @type
                "u64" @type
                "u128" @type
                "usize" @type
                "f32" @type
                "f64" @type
                (operator) @operator
            "#,
            SupportedLanguage::Python => r#"
                (comment) @comment
                (string) @string
                (integer) @number
                (float) @number
                (call function: (identifier) @function)
                (class_definition name: (identifier) @type)
                (function_definition name: (identifier) @function)
                (parameter identifier: (identifier) @parameter)
                (assignment left: (identifier) @variable)
                "def" @keyword
                "class" @keyword
                "if" @keyword
                "elif" @keyword
                "else" @keyword
                "for" @keyword
                "while" @keyword
                "return" @keyword
                "import" @keyword
                "from" @keyword
                "as" @keyword
                "try" @keyword
                "except" @keyword
                "finally" @keyword
                "with" @keyword
                "lambda" @keyword
                "yield" @keyword
                "pass" @keyword
                "break" @keyword
                "continue" @keyword
                "and" @keyword
                "or" @keyword
                "not" @keyword
                "in" @keyword
                "is" @keyword
                "None" @constant
                "True" @constant
                "False" @constant
            "#,
            SupportedLanguage::JavaScript => r#"
                (comment) @comment
                (string) @string
                (template_string) @string
                (number) @number
                (call_expression function: (identifier) @function)
                (method_definition name: (property_identifier) @method)
                (class_declaration name: (identifier) @type)
                (variable_declarator name: (identifier) @variable)
                (formal_parameter name: (identifier) @parameter)
                "function" @keyword
                "const" @keyword
                "let" @keyword
                "var" @keyword
                "if" @keyword
                "else" @keyword
                "for" @keyword
                "while" @keyword
                "do" @keyword
                "return" @keyword
                "break" @keyword
                "continue" @keyword
                "switch" @keyword
                "case" @keyword
                "default" @keyword
                "try" @keyword
                "catch" @keyword
                "finally" @keyword
                "throw" @keyword
                "new" @keyword
                "class" @keyword
                "extends" @keyword
                "import" @keyword
                "export" @keyword
                "from" @keyword
                "async" @keyword
                "await" @keyword
                "yield" @keyword
                "typeof" @keyword
                "instanceof" @keyword
                "null" @constant
                "undefined" @constant
                "true" @constant
                "false" @constant
            "#,
            SupportedLanguage::TypeScript => r#"
                (comment) @comment
                (string) @string
                (template_string) @string
                (number) @number
                (call_expression function: (identifier) @function)
                (method_definition name: (property_identifier) @method)
                (class_declaration name: (identifier) @type)
                (interface_declaration name: (identifier) @type)
                (type_alias_declaration name: (identifier) @type)
                (variable_declarator name: (identifier) @variable)
                (required_parameter name: (identifier) @parameter)
                "function" @keyword
                "const" @keyword
                "let" @keyword
                "var" @keyword
                "if" @keyword
                "else" @keyword
                "for" @keyword
                "while" @keyword
                "return" @keyword
                "break" @keyword
                "continue" @keyword
                "switch" @keyword
                "case" @keyword
                "default" @keyword
                "try" @keyword
                "catch" @keyword
                "finally" @keyword
                "throw" @keyword
                "new" @keyword
                "class" @keyword
                "extends" @keyword
                "import" @keyword
                "export" @keyword
                "from" @keyword
                "async" @keyword
                "await" @keyword
                "interface" @keyword
                "type" @keyword
                "enum" @keyword
                "namespace" @keyword
                "module" @keyword
                "declare" @keyword
                "implements" @keyword
                "public" @keyword
                "private" @keyword
                "protected" @keyword
                "readonly" @keyword
                "null" @constant
                "undefined" @constant
                "true" @constant
                "false" @constant
            "#,
            SupportedLanguage::Cpp => r#"
                (comment) @comment
                (string_literal) @string
                (character_literal) @string
                (number_literal) @number
                (call_expression function: (identifier) @function)
                (function_definition declarator: (function_declarator declarator: (identifier) @function))
                (class_specifier name: (type_identifier) @type)
                (struct_specifier name: (type_identifier) @type)
                (primitive_type) @type
                (parameter_declaration declarator: (identifier) @parameter)
                "int" @keyword
                "float" @keyword
                "double" @keyword
                "char" @keyword
                "void" @keyword
                "bool" @keyword
                "long" @keyword
                "short" @keyword
                "unsigned" @keyword
                "signed" @keyword
                "if" @keyword
                "else" @keyword
                "for" @keyword
                "while" @keyword
                "do" @keyword
                "return" @keyword
                "break" @keyword
                "continue" @keyword
                "switch" @keyword
                "case" @keyword
                "default" @keyword
                "try" @keyword
                "catch" @keyword
                "throw" @keyword
                "new" @keyword
                "delete" @keyword
                "class" @keyword
                "struct" @keyword
                "union" @keyword
                "enum" @keyword
                "namespace" @keyword
                "using" @keyword
                "template" @keyword
                "typename" @keyword
                "typedef" @keyword
                "const" @keyword
                "volatile" @keyword
                "static" @keyword
                "extern" @keyword
                "inline" @keyword
                "virtual" @keyword
                "override" @keyword
                "final" @keyword
                "public" @keyword
                "private" @keyword
                "protected" @keyword
                "nullptr" @constant
                "true" @constant
                "false" @constant
                "this" @variable
            "#,
            SupportedLanguage::C => r#"
                (comment) @comment
                (string_literal) @string
                (char_literal) @string
                (number_literal) @number
                (call_expression function: (identifier) @function)
                (function_definition declarator: (function_declarator declarator: (identifier) @function))
                (primitive_type) @type
                (parameter_declaration declarator: (identifier) @parameter)
                "int" @keyword
                "float" @keyword
                "double" @keyword
                "char" @keyword
                "void" @keyword
                "long" @keyword
                "short" @keyword
                "unsigned" @keyword
                "signed" @keyword
                "if" @keyword
                "else" @keyword
                "for" @keyword
                "while" @keyword
                "do" @keyword
                "return" @keyword
                "break" @keyword
                "continue" @keyword
                "switch" @keyword
                "case" @keyword
                "default" @keyword
                "struct" @keyword
                "union" @keyword
                "enum" @keyword
                "typedef" @keyword
                "const" @keyword
                "volatile" @keyword
                "static" @keyword
                "extern" @keyword
                "register" @keyword
                "auto" @keyword
                "goto" @keyword
                "sizeof" @keyword
                "NULL" @constant
                "true" @constant
                "false" @constant
            "#,
            SupportedLanguage::Unknown => "",
        }
    }

    /// Highlight code and return segments
    pub fn highlight(&self, code: &str) -> Vec<HighlightedSegment> {
        if self.current_language.is_none() {
            return vec![];
        }

        let tree = self.parser.parse(code, None).unwrap();
        let root = tree.root_node();
        
        let lang = self.current_language.unwrap();
        let query = match self.queries.get(&lang) {
            Some(q) => q,
            None => return vec![],
        };

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, root, code.as_bytes());

        let mut segments = Vec::new();

        for m in matches {
            for capture in m.captures {
                let node = capture.node;
                let start = node.start_byte();
                let end = node.end_byte();
                
                // Get the capture name from the query
                let capture_name = query.capture_names()[capture.index as usize];
                let color = self.get_color_for_capture(capture_name);
                
                segments.push(HighlightedSegment {
                    start,
                    end,
                    color,
                });
            }
        }

        // Sort by start position
        segments.sort_by_key(|s| s.start);
        
        // Merge overlapping segments (keep the later one which is more specific)
        let mut merged = Vec::new();
        for segment in segments {
            if let Some(last) = merged.last_mut() {
                if last.end > segment.start {
                    // Overlapping, replace with the new one (more specific)
                    *last = segment;
                } else {
                    merged.push(segment);
                }
            } else {
                merged.push(segment);
            }
        }

        merged
    }

    /// Get color for a capture based on its name
    fn get_color_for_capture(&self, capture_name: &str) -> u32 {
        match capture_name {
            "comment" => self.color_scheme.comment,
            "string" => self.color_scheme.string,
            "number" => self.color_scheme.number,
            "function" | "method" => self.color_scheme.function,
            "type" => self.color_scheme.type_name,
            "keyword" => self.color_scheme.keyword,
            "variable" => self.color_scheme.variable,
            "parameter" => self.color_scheme.parameter,
            "constant" => self.color_scheme.constant,
            "operator" => self.color_scheme.operator,
            _ => self.color_scheme.default,
        }
    }

    /// Set custom color scheme
    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    /// Get current color scheme
    pub fn color_scheme(&self) -> &ColorScheme {
        &self.color_scheme
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
