use std::path::Path;
use tree_sitter::{Node, Tree, TreeCursor};
use anyhow::Result;
use crate::core::ast_analyzer::{
    CodeSegment, SegmentExtractor, SegmentType, SegmentMetadata,
    FunctionSegment, ClassSegment, RouteSegment, DatabaseSegment
};
use crate::core::types::Framework;

pub struct PythonExtractor;

impl PythonExtractor {
    pub fn new() -> Self {
        Self
    }

    fn extract_function_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        let function_name = self.get_function_name(node, source)?;
        let parameters = self.extract_parameters(node, source);
        let return_type = self.extract_return_annotation(node, source);
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
            framework_context: self.detect_function_framework(node, source),
            business_hints: self.extract_business_hints(node, source),
        })
    }

    fn extract_class_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
        let class_name = self.get_class_name(node, source)?;
        let extends = self.get_base_classes(node, source);
        let methods = self.extract_class_methods(node, source);
        let is_model = self.is_database_model(node, source);

        // If it's a database model, create a DatabaseSegment instead
        if is_model {
            let table_name = self.extract_table_name(node, source);
            let fields = self.extract_model_fields(node, source);
            let relationships = self.extract_relationships(node, source);

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
                framework_context: self.detect_model_framework(node, source),
                business_hints: self.extract_business_hints(node, source),
            });
        }

        Some(CodeSegment {
            segment_type: SegmentType::Class(crate::core::ast_analyzer::ClassSegment {
                name: class_name,
                extends: extends.first().cloned(),
                implements: Vec::new(), // Python doesn't have interfaces like TypeScript
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

    fn extract_route_segment(&self, node: &Node, source: &str, file_path: &Path) -> Option<CodeSegment> {
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

    fn extract_parameters(&self, node: &Node, source: &str) -> Vec<String> {
        let mut parameters = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "parameters" {
                    let mut param_cursor = cursor.node().walk();
                    if param_cursor.goto_first_child() {
                        loop {
                            if param_cursor.node().kind() == "identifier" ||
                               param_cursor.node().kind() == "typed_parameter" ||
                               param_cursor.node().kind() == "default_parameter" {
                                let param_text = self.get_node_text(&param_cursor.node(), source);
                                if param_text != "self" && param_text != "cls" {
                                    parameters.push(param_text.to_string());
                                }
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

    fn extract_return_annotation(&self, node: &Node, source: &str) -> Option<String> {
        let content = self.get_node_text(node, source);
        if let Some(arrow_pos) = content.find("->") {
            let after_arrow = &content[arrow_pos + 2..];
            if let Some(colon_pos) = after_arrow.find(':') {
                return Some(after_arrow[..colon_pos].trim().to_string());
            }
        }
        None
    }

    fn is_async_function(&self, node: &Node, source: &str) -> bool {
        let content = self.get_node_text(node, source);
        content.starts_with("async def") || content.contains("async def")
    }

    fn extract_decorators(&self, node: &Node, source: &str) -> Vec<String> {
        let mut decorators = Vec::new();
        
        // Look for decorators before the function/class
        let start_line = node.start_position().row;
        let lines: Vec<&str> = source.lines().collect();
        
        // Check lines before the current node for decorators
        for i in (0..start_line).rev() {
            let line = lines.get(i).unwrap_or(&"").trim();
            if line.starts_with('@') {
                decorators.insert(0, line.to_string());
            } else if !line.is_empty() {
                break; // Stop at first non-empty, non-decorator line
            }
        }
        
        decorators
    }

    fn get_base_classes(&self, node: &Node, source: &str) -> Vec<String> {
        let mut base_classes = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "argument_list" {
                    let mut arg_cursor = cursor.node().walk();
                    if arg_cursor.goto_first_child() {
                        loop {
                            if arg_cursor.node().kind() == "identifier" {
                                base_classes.push(self.get_node_text(&arg_cursor.node(), source).to_string());
                            }
                            if !arg_cursor.goto_next_sibling() {
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

        base_classes
    }

    fn extract_class_methods(&self, node: &Node, source: &str) -> Vec<String> {
        let mut methods = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "block" {
                    let mut block_cursor = cursor.node().walk();
                    if block_cursor.goto_first_child() {
                        loop {
                            if block_cursor.node().kind() == "function_definition" {
                                if let Some(method_name) = self.get_function_name(&block_cursor.node(), source) {
                                    methods.push(method_name);
                                }
                            }
                            if !block_cursor.goto_next_sibling() {
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

    fn is_database_model(&self, node: &Node, source: &str) -> bool {
        let content = self.get_node_text(node, source);
        
        // SQLAlchemy patterns
        content.contains("db.Model") ||
        content.contains("Base") ||
        content.contains("declarative_base") ||
        content.contains("Column") ||
        content.contains("relationship") ||
        
        // Django patterns
        content.contains("models.Model") ||
        content.contains("models.CharField") ||
        content.contains("models.IntegerField") ||
        
        // Pydantic patterns for API models
        content.contains("BaseModel") ||
        content.contains("Field(")
    }

    fn extract_table_name(&self, node: &Node, source: &str) -> Option<String> {
        let content = self.get_node_text(node, source);
        
        // Look for __tablename__ attribute
        if let Some(table_start) = content.find("__tablename__") {
            let after_tablename = &content[table_start..];
            if let Some(equals_pos) = after_tablename.find('=') {
                let after_equals = &after_tablename[equals_pos + 1..];
                let line_end = after_equals.find('\n').unwrap_or(after_equals.len());
                let table_value = after_equals[..line_end].trim();
                return Some(table_value.trim_matches(|c| c == '"' || c == '\'').to_string());
            }
        }
        
        None
    }

    fn extract_model_fields(&self, node: &Node, source: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let content = self.get_node_text(node, source);
        
        // Look for field definitions
        for line in content.lines() {
            let trimmed = line.trim();
            
            // SQLAlchemy fields
            if (trimmed.contains("Column(") || 
                trimmed.contains("db.Column(") ||
                trimmed.contains("models.") ||
                trimmed.contains("Field(")) &&
                trimmed.contains("=") {
                
                if let Some(equals_pos) = trimmed.find('=') {
                    let field_name = trimmed[..equals_pos].trim();
                    if !field_name.is_empty() && !field_name.starts_with('_') {
                        fields.push(field_name.to_string());
                    }
                }
            }
        }
        
        fields
    }

    fn extract_relationships(&self, node: &Node, source: &str) -> Vec<String> {
        let mut relationships = Vec::new();
        let content = self.get_node_text(node, source);
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("relationship(") || 
               trimmed.contains("ForeignKey(") ||
               trimmed.contains("db.relationship(") ||
               trimmed.contains("models.ForeignKey(") {
                
                if let Some(equals_pos) = trimmed.find('=') {
                    let rel_name = trimmed[..equals_pos].trim();
                    if !rel_name.is_empty() {
                        relationships.push(rel_name.to_string());
                    }
                }
            }
        }
        
        relationships
    }

    fn parse_route_pattern(&self, content: &str) -> Option<RouteInfo> {
        // Flask route patterns
        if content.contains("@app.route") || content.contains("@bp.route") {
            return self.parse_flask_route(content);
        }
        
        // FastAPI route patterns  
        if content.contains("@app.get") || content.contains("@app.post") ||
           content.contains("@app.put") || content.contains("@app.delete") ||
           content.contains("@router.get") || content.contains("@router.post") {
            return self.parse_fastapi_route(content);
        }
        
        // Django view patterns
        if content.contains("def") && (content.contains("request") || content.contains("HttpResponse")) {
            return self.parse_django_view(content);
        }
        
        None
    }

    fn parse_flask_route(&self, content: &str) -> Option<RouteInfo> {
        for line in content.lines() {
            if line.contains("@app.route") || line.contains("@bp.route") {
                // Extract path from route decorator
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.find(')') {
                        let params = &line[start + 1..end];
                        let path = if let Some(comma) = params.find(',') {
                            params[..comma].trim()
                        } else {
                            params.trim()
                        };
                        
                        let path = path.trim_matches(|c| c == '"' || c == '\'');
                        
                        // Extract methods if specified
                        let methods = if params.contains("methods") {
                            self.extract_flask_methods(params)
                        } else {
                            vec!["GET".to_string()]
                        };
                        
                        return Some(RouteInfo {
                            path: path.to_string(),
                            method: methods.join(","),
                            handler: "Flask Route Handler".to_string(),
                            middleware: Vec::new(),
                        });
                    }
                }
            }
        }
        None
    }

    fn parse_fastapi_route(&self, content: &str) -> Option<RouteInfo> {
        for line in content.lines() {
            if line.contains("@app.") || line.contains("@router.") {
                let method = if line.contains(".get") { "GET" }
                else if line.contains(".post") { "POST" }
                else if line.contains(".put") { "PUT" }
                else if line.contains(".delete") { "DELETE" }
                else { "UNKNOWN" };

                // Extract path
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.find(')') {
                        let params = &line[start + 1..end];
                        let path = if let Some(comma) = params.find(',') {
                            params[..comma].trim()
                        } else {
                            params.trim()
                        };
                        
                        let path = path.trim_matches(|c| c == '"' || c == '\'');
                        
                        return Some(RouteInfo {
                            path: path.to_string(),
                            method: method.to_string(),
                            handler: "FastAPI Endpoint".to_string(),
                            middleware: Vec::new(),
                        });
                    }
                }
            }
        }
        None
    }

    fn parse_django_view(&self, content: &str) -> Option<RouteInfo> {
        // Django views are typically class-based or function-based
        // The routing is usually defined in urls.py, but we can detect view functions
        if content.contains("def") && content.contains("request") {
            return Some(RouteInfo {
                path: "/*".to_string(), // Django URLs are defined separately
                method: "ANY".to_string(),
                handler: "Django View".to_string(),
                middleware: Vec::new(),
            });
        }
        None
    }

    fn extract_flask_methods(&self, params: &str) -> Vec<String> {
        if let Some(methods_start) = params.find("methods") {
            let after_methods = &params[methods_start..];
            if let Some(bracket_start) = after_methods.find('[') {
                if let Some(bracket_end) = after_methods.find(']') {
                    let methods_str = &after_methods[bracket_start + 1..bracket_end];
                    return methods_str
                        .split(',')
                        .map(|m| m.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                        .collect();
                }
            }
        }
        vec!["GET".to_string()]
    }

    fn detect_function_framework(&self, node: &Node, source: &str) -> Option<Framework> {
        let content = self.get_node_text(node, source).to_lowercase();
        
        if content.contains("@app.route") || content.contains("flask") {
            Some(Framework::Flask)
        } else if content.contains("@app.get") || content.contains("fastapi") {
            Some(Framework::FastAPI)
        } else if content.contains("request") && content.contains("response") {
            Some(Framework::Django)
        } else {
            None
        }
    }

    fn detect_class_framework(&self, node: &Node, source: &str) -> Option<Framework> {
        let content = self.get_node_text(node, source).to_lowercase();
        
        if content.contains("db.model") || content.contains("sqlalchemy") {
            Some(Framework::Flask) // Assuming Flask-SQLAlchemy
        } else if content.contains("models.model") {
            Some(Framework::Django)
        } else if content.contains("basemodel") {
            Some(Framework::FastAPI) // Pydantic models
        } else {
            None
        }
    }

    fn detect_model_framework(&self, node: &Node, source: &str) -> Option<Framework> {
        self.detect_class_framework(node, source)
    }

    fn detect_route_framework(&self, content: &str) -> Option<Framework> {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("@app.route") || content_lower.contains("flask") {
            Some(Framework::Flask)
        } else if content_lower.contains("@app.get") || content_lower.contains("fastapi") {
            Some(Framework::FastAPI)
        } else if content_lower.contains("django") || content_lower.contains("httpresponse") {
            Some(Framework::Django)
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

impl SegmentExtractor for PythonExtractor {
    fn extract_segments(&self, source: &str, file_path: &Path) -> Result<Vec<CodeSegment>> {
        let mut segments = Vec::new();

        // Parse the source code with tree-sitter
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_python::language())?;
        
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
           content.contains("jwt") || content.contains("token") {
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
           content.contains("invoice") || content.contains("charge") {
            hints.push("Payment".to_string());
        }

        // Analytics hints
        if content.contains("analytics") || content.contains("tracking") || content.contains("metrics") ||
           content.contains("report") || content.contains("dashboard") {
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

impl PythonExtractor {
    fn traverse_node(&self, node: &Node, source: &str, file_path: &Path, segments: &mut Vec<CodeSegment>) {
        match node.kind() {
            "function_definition" => {
                if let Some(segment) = self.extract_function_segment(node, source, file_path) {
                    segments.push(segment);
                }
            }
            "class_definition" => {
                if let Some(segment) = self.extract_class_segment(node, source, file_path) {
                    segments.push(segment);
                }
            }
            _ => {
                // Check if this might be a route definition or other pattern
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

