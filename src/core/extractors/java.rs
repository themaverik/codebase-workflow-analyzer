use std::path::Path;
use tree_sitter::{Node, Tree, TreeCursor};
use anyhow::Result;
use crate::core::ast_analyzer::{
    CodeSegment, SegmentExtractor, SegmentType, SegmentMetadata,
    FunctionSegment, ClassSegment, InterfaceSegment, RouteSegment, DatabaseSegment
};
use crate::core::types::Framework;

pub struct JavaExtractor;

impl JavaExtractor {
    pub fn new() -> Self {
        Self
    }

    fn extract_method_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        let method_name = self.get_method_name(node, source)?;
        let parameters = self.extract_parameters(node, source);
        let return_type = self.extract_return_type(node, source);
        let is_async = false; // Java methods are not async in the same way as JS/Python
        let annotations = self.extract_annotations(node, source);

        Some(CodeSegment {
            segment_type: SegmentType::Function(FunctionSegment {
                name: method_name,
                parameters,
                return_type,
                is_async,
                decorators: annotations,
            }),
            content: self.get_node_text(node, source).to_string(),
            metadata: SegmentMetadata {
                line_start: node.start_position().row + 1,
                line_end: node.end_position().row + 1,
                file_path: file_path.to_path_buf(),
                byte_start: node.start_byte(),
                byte_end: node.end_byte(),
            },
            framework_context: self.detect_method_framework(node, source),
            business_hints: self.extract_business_hints(node, source),
        })
    }

    fn extract_class_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        let class_name = self.get_class_name(node, source)?;
        let extends = self.get_extends_clause(node, source);
        let implements = self.get_implements_clause(node, source);
        let methods = self.extract_class_methods(node, source);
        let is_entity = self.is_jpa_entity(node, source);

        // If it's a JPA entity, create a DatabaseSegment instead
        if is_entity {
            let table_name = self.extract_table_name(node, source);
            let fields = self.extract_entity_fields(node, source);
            let relationships = self.extract_entity_relationships(node, source);

            return Some(CodeSegment {
                segment_type: SegmentType::Database(DatabaseSegment {
                    model_name: class_name,
                    table_name,
                    fields,
                    relationships,
                }),
                content: self.get_node_text(node, source).to_string(),
                metadata: SegmentMetadata {
                    line_start: node.start_position().row + 1,
                    line_end: node.end_position().row + 1,
                    file_path: file_path.to_path_buf(),
                    byte_start: node.start_byte(),
                    byte_end: node.end_byte(),
                },
                framework_context: Some(Framework::SpringBoot),
                business_hints: self.extract_business_hints(node, source),
            });
        }

        Some(CodeSegment {
            segment_type: SegmentType::Class(ClassSegment {
                name: class_name,
                extends,
                implements,
                is_react_component: false,
                props: Vec::new(),
                hooks: Vec::new(),
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
            framework_context: self.detect_class_framework(node, source),
            business_hints: self.extract_business_hints(node, source),
        })
    }

    fn extract_interface_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        let interface_name = self.get_interface_name(node, source)?;
        let extends = self.get_interface_extends(node, source);
        let methods = self.extract_interface_methods(node, source);

        Some(CodeSegment {
            segment_type: SegmentType::Interface(InterfaceSegment {
                name: interface_name,
                extends,
                properties: methods, // Java interfaces have methods, not properties
            }),
            content: self.get_node_text(node, source).to_string(),
            metadata: SegmentMetadata {
                line_start: node.start_position().row + 1,
                line_end: node.end_position().row + 1,
                file_path: file_path.to_path_buf(),
                byte_start: node.start_byte(),
                byte_end: node.end_byte(),
            },
            framework_context: self.detect_interface_framework(node, source),
            business_hints: self.extract_business_hints(node, source),
        })
    }

    fn extract_route_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        let content = self.get_node_text(node, source);
        
        if let Some(route_info) = self.parse_spring_route(content) {
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
                framework_context: Some(Framework::SpringBoot),
                business_hints: self.extract_business_hints(node, source),
            });
        }

        None
    }

    // Helper methods
    fn get_node_text<'a>(&self, node: &Node, source: &'a str) -> &'a str {
        &source[node.start_byte()..node.end_byte()]
    }

    fn get_method_name(&self, node: &Node, source: &str) -> Option<String> {
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

    fn get_interface_name(&self, node: &Node, source: &str) -> Option<String> {
        self.get_class_name(node, source) // Same logic
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
                            if param_cursor.node().kind() == "formal_parameter" {
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
        
        // Look for type before method name
        if cursor.goto_first_child() {
            loop {
                let kind = cursor.node().kind();
                if kind == "type_identifier" || kind == "generic_type" || kind == "void_type" {
                    return Some(self.get_node_text(&cursor.node(), source).to_string());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }

    fn extract_annotations(&self, node: &Node, source: &str) -> Vec<String> {
        let mut annotations = Vec::new();
        
        // Look for annotations before the method/class
        let start_line = node.start_position().row;
        let lines: Vec<&str> = source.lines().collect();
        
        // Check lines before the current node for annotations
        for i in (0..start_line).rev() {
            let line = lines.get(i).unwrap_or(&"").trim();
            if line.starts_with('@') {
                annotations.insert(0, line.to_string());
            } else if !line.is_empty() && !line.starts_with("//") && !line.starts_with("/*") {
                break; // Stop at first non-empty, non-comment, non-annotation line
            }
        }
        
        annotations
    }

    fn get_extends_clause(&self, node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "superclass" {
                    let mut super_cursor = cursor.node().walk();
                    if super_cursor.goto_first_child() && super_cursor.node().kind() == "type_identifier" {
                        return Some(self.get_node_text(&super_cursor.node(), source).to_string());
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }

    fn get_implements_clause(&self, node: &Node, source: &str) -> Vec<String> {
        let mut implements = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "super_interfaces" {
                    let mut interface_cursor = cursor.node().walk();
                    if interface_cursor.goto_first_child() {
                        loop {
                            if interface_cursor.node().kind() == "type_identifier" {
                                implements.push(self.get_node_text(&interface_cursor.node(), source).to_string());
                            }
                            if !interface_cursor.goto_next_sibling() {
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

        implements
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
                            if body_cursor.node().kind() == "method_declaration" {
                                if let Some(method_name) = self.get_method_name(&body_cursor.node(), source) {
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
        let mut extends = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "extends_interfaces" {
                    let mut extends_cursor = cursor.node().walk();
                    if extends_cursor.goto_first_child() {
                        loop {
                            if extends_cursor.node().kind() == "type_identifier" {
                                extends.push(self.get_node_text(&extends_cursor.node(), source).to_string());
                            }
                            if !extends_cursor.goto_next_sibling() {
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

        extends
    }

    fn extract_interface_methods(&self, node: &Node, source: &str) -> Vec<String> {
        let mut methods = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "interface_body" {
                    let mut body_cursor = cursor.node().walk();
                    if body_cursor.goto_first_child() {
                        loop {
                            if body_cursor.node().kind() == "method_declaration" {
                                if let Some(method_name) = self.get_method_name(&body_cursor.node(), source) {
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

    fn is_jpa_entity(&self, node: &Node, source: &str) -> bool {
        let content = self.get_node_text(node, source);
        content.contains("@Entity") ||
        content.contains("@Table") ||
        content.contains("@Id") ||
        content.contains("@Column") ||
        content.contains("@JoinColumn") ||
        content.contains("@ManyToOne") ||
        content.contains("@OneToMany") ||
        content.contains("@ManyToMany")
    }

    fn extract_table_name(&self, node: &Node, source: &str) -> Option<String> {
        let content = self.get_node_text(node, source);
        
        // Look for @Table annotation
        if let Some(table_start) = content.find("@Table") {
            let after_table = &content[table_start..];
            if let Some(name_start) = after_table.find("name") {
                let after_name = &after_table[name_start..];
                if let Some(equals) = after_name.find('=') {
                    let after_equals = &after_name[equals + 1..];
                    let end_pos = after_equals.find(|c: char| c == ',' || c == ')').unwrap_or(after_equals.len());
                    let table_name = after_equals[..end_pos].trim();
                    return Some(table_name.trim_matches(|c| c == '"' || c == '\'').to_string());
                }
            }
        }
        
        None
    }

    fn extract_entity_fields(&self, node: &Node, source: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let content = self.get_node_text(node, source);
        
        // Look for field declarations with JPA annotations
        for line in content.lines() {
            let trimmed = line.trim();
            if (trimmed.contains("@Column") || 
                trimmed.contains("@Id") ||
                trimmed.contains("@GeneratedValue")) &&
                !trimmed.starts_with("//") {
                
                // Look for the field declaration in the next few lines
                // This is a simplified approach - proper AST traversal would be better
                continue;
            }
            
            // Look for field declarations (private/public/protected fields)
            if (trimmed.starts_with("private ") || 
                trimmed.starts_with("public ") ||
                trimmed.starts_with("protected ")) &&
                trimmed.contains(";") && 
                !trimmed.contains("(") { // Exclude methods
                
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 3 {
                    let field_name = parts[parts.len() - 1].trim_end_matches(';');
                    fields.push(field_name.to_string());
                }
            }
        }
        
        fields
    }

    fn extract_entity_relationships(&self, node: &Node, source: &str) -> Vec<String> {
        let mut relationships = Vec::new();
        let content = self.get_node_text(node, source);
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("@OneToMany") || 
               trimmed.contains("@ManyToOne") ||
               trimmed.contains("@ManyToMany") ||
               trimmed.contains("@OneToOne") {
                
                // The relationship field is typically on the next line or same line
                // This is simplified - would need proper AST traversal for accuracy
                if let Some(field_line) = self.find_next_field_line(content, line) {
                    relationships.push(field_line);
                }
            }
        }
        
        relationships
    }

    fn find_next_field_line(&self, content: &str, current_line: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        if let Some(current_index) = lines.iter().position(|&line| line == current_line) {
            for i in (current_index + 1)..lines.len() {
                let line = lines[i].trim();
                if (line.starts_with("private ") || 
                    line.starts_with("public ") ||
                    line.starts_with("protected ")) &&
                    line.contains(";") {
                    
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let field_name = parts[parts.len() - 1].trim_end_matches(';');
                        return Some(field_name.to_string());
                    }
                }
            }
        }
        None
    }

    fn parse_spring_route(&self, content: &str) -> Option<RouteInfo> {
        // Spring Boot REST controller mappings
        if content.contains("@RequestMapping") || 
           content.contains("@GetMapping") ||
           content.contains("@PostMapping") ||
           content.contains("@PutMapping") ||
           content.contains("@DeleteMapping") {
            return self.parse_spring_mapping(content);
        }
        
        None
    }

    fn parse_spring_mapping(&self, content: &str) -> Option<RouteInfo> {
        let method = if content.contains("@GetMapping") { "GET".to_string() }
        else if content.contains("@PostMapping") { "POST".to_string() }
        else if content.contains("@PutMapping") { "PUT".to_string() }
        else if content.contains("@DeleteMapping") { "DELETE".to_string() }
        else if content.contains("@RequestMapping") { 
            // Try to extract method from @RequestMapping
            self.extract_request_mapping_method(content).unwrap_or("ANY".to_string())
        } else { "UNKNOWN".to_string() };

        // Extract path from mapping annotation
        let path = self.extract_mapping_path(content).unwrap_or("/".to_string());

        Some(RouteInfo {
            path,
            method,
            handler: "Spring Boot Controller Method".to_string(),
            middleware: Vec::new(),
        })
    }

    fn extract_request_mapping_method(&self, content: &str) -> Option<String> {
        if let Some(method_start) = content.find("method") {
            let after_method = &content[method_start..];
            if let Some(equals) = after_method.find('=') {
                let after_equals = &after_method[equals + 1..];
                if after_equals.contains("RequestMethod.GET") {
                    return Some("GET".to_string());
                } else if after_equals.contains("RequestMethod.POST") {
                    return Some("POST".to_string());
                } else if after_equals.contains("RequestMethod.PUT") {
                    return Some("PUT".to_string());
                } else if after_equals.contains("RequestMethod.DELETE") {
                    return Some("DELETE".to_string());
                }
            }
        }
        None
    }

    fn extract_mapping_path(&self, content: &str) -> Option<String> {
        // Look for value or path parameter in mapping annotations
        for annotation in ["@GetMapping", "@PostMapping", "@PutMapping", "@DeleteMapping", "@RequestMapping"] {
            if let Some(annotation_start) = content.find(annotation) {
                let after_annotation = &content[annotation_start..];
                if let Some(paren_start) = after_annotation.find('(') {
                    if let Some(paren_end) = after_annotation.find(')') {
                        let params = &after_annotation[paren_start + 1..paren_end];
                        
                        // Handle different parameter formats
                        if params.starts_with('"') || params.starts_with('\'') {
                            // Simple case: @GetMapping("/path")
                            return Some(params.trim_matches(|c| c == '"' || c == '\'').to_string());
                        } else if params.contains("value") || params.contains("path") {
                            // Handle value = "/path" or path = "/path"
                            if let Some(equals) = params.find('=') {
                                let after_equals = &params[equals + 1..];
                                let path_end = after_equals.find(|c: char| c == ',' || c == ')').unwrap_or(after_equals.len());
                                let path = after_equals[..path_end].trim();
                                return Some(path.trim_matches(|c| c == '"' || c == '\'').to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn detect_method_framework(&self, node: &Node, source: &str) -> Option<Framework> {
        let content = self.get_node_text(node, source);
        
        if content.contains("@RestController") || 
           content.contains("@RequestMapping") ||
           content.contains("@GetMapping") ||
           content.contains("@Service") ||
           content.contains("@Component") {
            Some(Framework::SpringBoot)
        } else {
            None
        }
    }

    fn detect_class_framework(&self, node: &Node, source: &str) -> Option<Framework> {
        let content = self.get_node_text(node, source);
        
        if content.contains("@RestController") ||
           content.contains("@Service") ||
           content.contains("@Repository") ||
           content.contains("@Component") ||
           content.contains("@Entity") ||
           content.contains("@Configuration") {
            Some(Framework::SpringBoot)
        } else {
            None
        }
    }

    fn detect_interface_framework(&self, node: &Node, source: &str) -> Option<Framework> {
        let content = self.get_node_text(node, source);
        
        if content.contains("@Repository") || 
           content.contains("JpaRepository") ||
           content.contains("CrudRepository") {
            Some(Framework::SpringBoot)
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

impl SegmentExtractor for JavaExtractor {
    fn extract_segments(&self, source: &str, file_path: &Path) -> Result<Vec<CodeSegment>> {
        let mut segments = Vec::new();

        // Parse the source code with tree-sitter
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_java::language())?;
        
        if let Some(tree) = parser.parse(source, None) {
            let root = tree.root_node();
            self.traverse_node(&root, source, file_path, &mut segments);
        }

        Ok(segments)
    }

    fn extract_business_hints(&self, node: &Node, source: &str) -> Vec<String> {
        let mut hints = Vec::new();
        let content = self.get_node_text(node, source).to_lowercase();

        // Authentication hints
        if content.contains("auth") || content.contains("login") || content.contains("password") ||
           content.contains("jwt") || content.contains("token") || content.contains("security") {
            hints.push("Authentication".to_string());
        }

        // User management hints
        if content.contains("user") || content.contains("profile") || content.contains("account") ||
           content.contains("permission") || content.contains("role") {
            hints.push("User Management".to_string());
        }

        // Notification hints
        if content.contains("notification") || content.contains("email") || content.contains("sms") ||
           content.contains("alert") || content.contains("message") {
            hints.push("Notification".to_string());
        }

        // Payment hints
        if content.contains("payment") || content.contains("billing") || content.contains("subscription") ||
           content.contains("invoice") || content.contains("charge") || content.contains("transaction") {
            hints.push("Payment".to_string());
        }

        // Analytics hints
        if content.contains("analytics") || content.contains("tracking") || content.contains("metrics") ||
           content.contains("report") || content.contains("dashboard") || content.contains("audit") {
            hints.push("Analytics".to_string());
        }

        // E-commerce hints
        if content.contains("product") || content.contains("order") || content.contains("cart") ||
           content.contains("inventory") || content.contains("catalog") {
            hints.push("E-commerce".to_string());
        }

        hints
    }
}

impl JavaExtractor {
    fn traverse_node(&self, node: &Node, source: &str, file_path: &Path, segments: &mut Vec<CodeSegment>) {
        match node.kind() {
            "method_declaration" => {
                if let Some(segment) = self.extract_method_segment(node, source, file_path) {
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

