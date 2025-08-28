use std::path::Path;
use tree_sitter::{Node, Tree, TreeCursor};
use anyhow::Result;
use crate::core::ast_analyzer::{
    CodeSegment, SegmentExtractor, SegmentType, SegmentMetadata,
    FunctionSegment, ClassSegment, InterfaceSegment, RouteSegment
};
use crate::core::types::Framework;

pub struct TypeScriptExtractor;

impl TypeScriptExtractor {
    pub fn new() -> Self {
        Self
    }

    fn extract_function_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        let function_name = self.get_function_name(node, source)?;
        let parameters = self.extract_parameters(node, source);
        let return_type = self.extract_return_type(node, source);
        let is_async = self.is_async_function(node, source);
        let decorators = self.extract_decorators(node, source);

        Some(CodeSegment {
            segment_type: SegmentType::Function(FunctionSegment {
                name: function_name,
                parameters,
                return_type,
                is_async,
                decorators,
            }),
            content: self.get_node_text(node, source).to_string(),
            metadata: SegmentMetadata {
                line_start: node.start_position().row + 1,
                line_end: node.end_position().row + 1,
                file_path: file_path.to_path_buf(),
                byte_start: node.start_byte(),
                byte_end: node.end_byte(),
            },
            framework_context: None,
            business_hints: self.extract_business_hints(node, source),
        })
    }

    fn extract_class_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        let class_name = self.get_class_name(node, source)?;
        let extends = self.get_extends_clause(node, source);
        let implements = self.get_implements_clause(node, source);
        let is_react_component = self.is_react_component(node, source);
        let props = self.extract_react_props(node, source);
        let hooks = self.extract_react_hooks(node, source);
        let methods = self.extract_class_methods(node, source);

        Some(CodeSegment {
            segment_type: SegmentType::Class(ClassSegment {
                name: class_name,
                extends,
                implements,
                is_react_component,
                props,
                hooks,
                methods,
            }),
            content: self.get_node_text(node, source).to_string(),
            metadata: SegmentMetadata {
                line_start: node.start_position().row + 1,
                line_end: node.end_position().row + 1,
                file_path: file_path.to_path_buf(),
                byte_start: node.start_byte(),
                byte_end: node.end_byte(),
            },
            framework_context: if is_react_component { Some(Framework::React) } else { None },
            business_hints: self.extract_business_hints(node, source),
        })
    }

    fn extract_interface_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        let interface_name = self.get_interface_name(node, source)?;
        let extends = self.get_interface_extends(node, source);
        let properties = self.extract_interface_properties(node, source);

        Some(CodeSegment {
            segment_type: SegmentType::Interface(InterfaceSegment {
                name: interface_name,
                extends,
                properties,
            }),
            content: self.get_node_text(node, source).to_string(),
            metadata: SegmentMetadata {
                line_start: node.start_position().row + 1,
                line_end: node.end_position().row + 1,
                file_path: file_path.to_path_buf(),
                byte_start: node.start_byte(),
                byte_end: node.end_byte(),
            },
            framework_context: None,
            business_hints: self.extract_business_hints(node, source),
        })
    }

    fn extract_route_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        // Look for Express.js, NestJS, or Next.js API routes
        let content = self.get_node_text(node, source);
        
        if let Some(route_info) = self.parse_route_pattern(content) {
            return Some(CodeSegment {
                segment_type: SegmentType::Route(RouteSegment {
                    path: route_info.path,
                    method: route_info.method,
                    handler: route_info.handler,
                    middleware: route_info.middleware,
                }),
                content: content.to_string(),
                metadata: SegmentMetadata {
                    line_start: node.start_position().row + 1,
                    line_end: node.end_position().row + 1,
                    file_path: file_path.to_path_buf(),
                    byte_start: node.start_byte(),
                    byte_end: node.end_byte(),
                },
                framework_context: self.detect_route_framework(content),
                business_hints: self.extract_business_hints(node, source),
            });
        }

        None
    }

    // Helper methods
    fn get_node_text<'a>(&self, node: &Node, source: &'a str) -> &'a str {
        &source[node.start_byte()..node.end_byte()]
    }

    fn get_function_name(&self, node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "identifier" {
                    return Some(self.get_node_text(&cursor.node(), source).to_string());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }

    fn get_class_name(&self, node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "type_identifier" || cursor.node().kind() == "identifier" {
                    return Some(self.get_node_text(&cursor.node(), source).to_string());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }

    fn get_interface_name(&self, node: &Node, source: &str) -> Option<String> {
        self.get_class_name(node, source) // Same logic for interfaces
    }

    fn extract_parameters(&self, node: &Node, source: &str) -> Vec<String> {
        let mut parameters = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "formal_parameters" {
                    let mut param_cursor = cursor.node().walk();
                    if param_cursor.goto_first_child() {
                        loop {
                            if param_cursor.node().kind() == "required_parameter" ||
                               param_cursor.node().kind() == "optional_parameter" {
                                let param_text = self.get_node_text(&param_cursor.node(), source);
                                parameters.push(param_text.to_string());
                            }
                            if !param_cursor.goto_next_sibling() {
                                break;
                            }
                        }
                    }
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        parameters
    }

    fn extract_return_type(&self, node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "type_annotation" {
                    return Some(self.get_node_text(&cursor.node(), source).to_string());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }

    fn is_async_function(&self, node: &Node, source: &str) -> bool {
        let content = self.get_node_text(node, source);
        content.contains("async ")
    }

    fn extract_decorators(&self, node: &Node, source: &str) -> Vec<String> {
        let mut decorators = Vec::new();
        let content = self.get_node_text(node, source);
        
        // Simple regex-like matching for decorators
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('@') {
                decorators.push(trimmed.to_string());
            }
        }
        
        decorators
    }

    fn get_extends_clause(&self, node: &Node, source: &str) -> Option<String> {
        let content = self.get_node_text(node, source);
        if let Some(extends_start) = content.find("extends ") {
            let after_extends = &content[extends_start + 8..];
            if let Some(end) = after_extends.find(|c: char| c == '{' || c == ' ' || c == '\n') {
                return Some(after_extends[..end].trim().to_string());
            }
        }
        None
    }

    fn get_implements_clause(&self, node: &Node, source: &str) -> Vec<String> {
        let content = self.get_node_text(node, source);
        let mut implements = Vec::new();
        
        if let Some(implements_start) = content.find("implements ") {
            let after_implements = &content[implements_start + 11..];
            if let Some(end) = after_implements.find('{') {
                let implements_str = after_implements[..end].trim();
                for interface in implements_str.split(',') {
                    implements.push(interface.trim().to_string());
                }
            }
        }
        
        implements
    }

    fn is_react_component(&self, node: &Node, source: &str) -> bool {
        let content = self.get_node_text(node, source);
        content.contains("React.Component") ||
        content.contains("Component") ||
        content.contains("jsx") ||
        content.contains("tsx") ||
        content.contains("useState") ||
        content.contains("useEffect")
    }

    fn extract_react_props(&self, node: &Node, source: &str) -> Vec<String> {
        let mut props = Vec::new();
        let content = self.get_node_text(node, source);
        
        // Look for props destructuring patterns
        for line in content.lines() {
            if line.contains("props") || line.contains("const {") {
                // Simple extraction - could be enhanced with proper AST traversal
                if let Some(start) = line.find('{') {
                    if let Some(end) = line.find('}') {
                        let props_str = &line[start + 1..end];
                        for prop in props_str.split(',') {
                            props.push(prop.trim().to_string());
                        }
                    }
                }
            }
        }
        
        props
    }

    fn extract_react_hooks(&self, node: &Node, source: &str) -> Vec<String> {
        let mut hooks = Vec::new();
        let content = self.get_node_text(node, source);
        
        let hook_patterns = [
            "useState", "useEffect", "useContext", "useReducer",
            "useCallback", "useMemo", "useRef", "useLayoutEffect"
        ];
        
        for hook in hook_patterns {
            if content.contains(hook) {
                hooks.push(hook.to_string());
            }
        }
        
        hooks
    }

    fn extract_class_methods(&self, node: &Node, source: &str) -> Vec<String> {
        let mut methods = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "class_body" {
                    let mut body_cursor = cursor.node().walk();
                    if body_cursor.goto_first_child() {
                        loop {
                            if body_cursor.node().kind() == "method_definition" {
                                if let Some(method_name) = self.get_function_name(&body_cursor.node(), source) {
                                    methods.push(method_name);
                                }
                            }
                            if !body_cursor.goto_next_sibling() {
                                break;
                            }
                        }
                    }
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        methods
    }

    fn get_interface_extends(&self, node: &Node, source: &str) -> Vec<String> {
        self.get_implements_clause(node, source) // Similar logic
    }

    fn extract_interface_properties(&self, node: &Node, source: &str) -> Vec<String> {
        let mut properties = Vec::new();
        let content = self.get_node_text(node, source);
        
        // Extract property names from interface body
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains(':') && !trimmed.starts_with("//") {
                if let Some(colon_pos) = trimmed.find(':') {
                    let prop_name = trimmed[..colon_pos].trim();
                    if !prop_name.is_empty() {
                        properties.push(prop_name.to_string());
                    }
                }
            }
        }
        
        properties
    }

    fn parse_route_pattern(&self, content: &str) -> Option<RouteInfo> {
        // NestJS route patterns
        if content.contains("@Get") || content.contains("@Post") || 
           content.contains("@Put") || content.contains("@Delete") {
            return self.parse_nestjs_route(content);
        }
        
        // Express.js route patterns
        if content.contains("app.get") || content.contains("app.post") ||
           content.contains("router.get") || content.contains("router.post") {
            return self.parse_express_route(content);
        }
        
        // Next.js API routes (file-based)
        if content.contains("export default") && content.contains("req") && content.contains("res") {
            return self.parse_nextjs_route(content);
        }
        
        None
    }

    fn parse_nestjs_route(&self, content: &str) -> Option<RouteInfo> {
        let method = if content.contains("@Get") { "GET" }
        else if content.contains("@Post") { "POST" }
        else if content.contains("@Put") { "PUT" }
        else if content.contains("@Delete") { "DELETE" }
        else { "UNKNOWN" };

        // Extract path from decorator
        let path = if let Some(start) = content.find(&format!("@{}", method.to_title_case())) {
            let after_decorator = &content[start..];
            if let Some(paren_start) = after_decorator.find('(') {
                if let Some(paren_end) = after_decorator.find(')') {
                    let path_str = &after_decorator[paren_start + 1..paren_end];
                    path_str.trim_matches(|c| c == '\'' || c == '"').to_string()
                } else {
                    "/".to_string()
                }
            } else {
                "/".to_string()
            }
        } else {
            "/".to_string()
        };

        Some(RouteInfo {
            path,
            method: method.to_string(),
            handler: "NestJS Controller Method".to_string(),
            middleware: Vec::new(),
        })
    }

    fn parse_express_route(&self, content: &str) -> Option<RouteInfo> {
        // Basic Express.js route parsing
        for line in content.lines() {
            if line.contains("app.") || line.contains("router.") {
                if let Some(method_start) = line.find('.') {
                    let after_dot = &line[method_start + 1..];
                    if let Some(paren) = after_dot.find('(') {
                        let method = after_dot[..paren].to_uppercase();
                        // Extract path from first parameter
                        let after_paren = &after_dot[paren + 1..];
                        if let Some(comma) = after_paren.find(',') {
                            let path_str = after_paren[..comma].trim();
                            let path = path_str.trim_matches(|c| c == '\'' || c == '"').to_string();
                            return Some(RouteInfo {
                                path,
                                method,
                                handler: "Express Route Handler".to_string(),
                                middleware: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn parse_nextjs_route(&self, content: &str) -> Option<RouteInfo> {
        // Next.js API routes are file-based, so path comes from file structure
        Some(RouteInfo {
            path: "/api/*".to_string(), // Will be refined with file path context
            method: "ANY".to_string(),
            handler: "Next.js API Handler".to_string(),
            middleware: Vec::new(),
        })
    }

    fn detect_route_framework(&self, content: &str) -> Option<Framework> {
        if content.contains("@Get") || content.contains("@Post") || content.contains("@Injectable") {
            Some(Framework::NestJS)
        } else if content.contains("app.get") || content.contains("express") {
            Some(Framework::Express)
        } else if content.contains("NextApiRequest") || content.contains("NextApiResponse") {
            Some(Framework::NextJS)
        } else {
            None
        }
    }
}

struct RouteInfo {
    path: String,
    method: String,
    handler: String,
    middleware: Vec<String>,
}

trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut chars = self.chars();
        let first = chars.next().unwrap().to_uppercase().collect::<String>();
        let rest = chars.collect::<String>().to_lowercase();
        first + &rest
    }
}

impl SegmentExtractor for TypeScriptExtractor {
    fn extract_segments(&self, source: &str, file_path: &Path) -> Result<Vec<CodeSegment>> {
        let mut segments = Vec::new();

        // Parse the source code with tree-sitter
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_typescript::language_typescript())?;
        
        if let Some(tree) = parser.parse(source, None) {
            let root = tree.root_node();
            let mut cursor = root.walk();

            // Traverse the AST to find relevant segments
            self.traverse_node(&root, source, file_path, &mut segments);
        }

        Ok(segments)
    }

    fn extract_business_hints(&self, node: &Node, source: &str) -> Vec<String> {
        let mut hints = Vec::new();
        let content = self.get_node_text(node, source).to_lowercase();

        // Authentication hints
        if content.contains("auth") || content.contains("login") || content.contains("password") {
            hints.push("Authentication".to_string());
        }

        // User management hints
        if content.contains("user") || content.contains("profile") || content.contains("account") {
            hints.push("User Management".to_string());
        }

        // Notification hints
        if content.contains("notification") || content.contains("email") || content.contains("sms") {
            hints.push("Notification".to_string());
        }

        // Payment hints
        if content.contains("payment") || content.contains("billing") || content.contains("subscription") {
            hints.push("Payment".to_string());
        }

        // Analytics hints
        if content.contains("analytics") || content.contains("tracking") || content.contains("metrics") {
            hints.push("Analytics".to_string());
        }

        hints
    }
}

impl TypeScriptExtractor {
    fn traverse_node(&self, node: &Node, source: &str, file_path: &Path, segments: &mut Vec<CodeSegment>) {
        match node.kind() {
            "function_declaration" | "arrow_function" | "method_definition" => {
                if let Some(segment) = self.extract_function_segment(node, source, file_path) {
                    segments.push(segment);
                }
            }
            "class_declaration" => {
                if let Some(segment) = self.extract_class_segment(node, source, file_path) {
                    segments.push(segment);
                }
            }
            "interface_declaration" => {
                if let Some(segment) = self.extract_interface_segment(node, source, file_path) {
                    segments.push(segment);
                }
            }
            _ => {
                // Check if this might be a route definition
                if let Some(segment) = self.extract_route_segment(node, source, file_path) {
                    segments.push(segment);
                }
            }
        }

        // Recursively traverse child nodes
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                self.traverse_node(&cursor.node(), source, file_path, segments);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }
}

