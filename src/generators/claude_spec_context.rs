use super::{DocumentGenerator, DocumentType};
use crate::core::CodebaseAnalysis;
use crate::intelligence::IntelligentAnalysis;
use anyhow::Result;

pub struct ClaudeSpecContextGenerator;

impl ClaudeSpecContextGenerator {
    pub fn new() -> Self {
        Self
    }

    fn extract_code_patterns(analysis: &CodebaseAnalysis) -> String {
        let mut patterns = String::new();
        
        patterns.push_str("### Framework-Specific Patterns\n\n");
        
        // Generate patterns based on detected frameworks
        for framework in &analysis.framework_analysis.detected_frameworks {
            match framework.name.as_str() {
                "React" => {
                    patterns.push_str("#### React Component Patterns\n\n");
                    patterns.push_str("```typescript\n");
                    patterns.push_str("// Functional Component with TypeScript\n");
                    patterns.push_str("interface ComponentProps {\n");
                    patterns.push_str("  title: string;\n");
                    patterns.push_str("  data: DataType[];\n");
                    patterns.push_str("  onAction?: (id: string) => void;\n");
                    patterns.push_str("}\n\n");
                    patterns.push_str("const Component: React.FC<ComponentProps> = ({ title, data, onAction }) => {\n");
                    patterns.push_str("  const [loading, setLoading] = useState<boolean>(false);\n");
                    patterns.push_str("  \n");
                    patterns.push_str("  useEffect(() => {\n");
                    patterns.push_str("    // Effect logic here\n");
                    patterns.push_str("  }, [data]);\n");
                    patterns.push_str("  \n");
                    patterns.push_str("  return (\n");
                    patterns.push_str("    <div className=\"component-container\">\n");
                    patterns.push_str("      <h2>{title}</h2>\n");
                    patterns.push_str("      {/* Component content */}\n");
                    patterns.push_str("    </div>\n");
                    patterns.push_str("  );\n");
                    patterns.push_str("};\n");
                    patterns.push_str("```\n\n");
                },
                "NestJS" => {
                    patterns.push_str("#### NestJS Architecture Patterns\n\n");
                    patterns.push_str("```typescript\n");
                    patterns.push_str("// Controller Pattern\n");
                    patterns.push_str("@Controller('api/resource')\n");
                    patterns.push_str("export class ResourceController {\n");
                    patterns.push_str("  constructor(private readonly resourceService: ResourceService) {}\n");
                    patterns.push_str("  \n");
                    patterns.push_str("  @Get()\n");
                    patterns.push_str("  async findAll(@Query() query: QueryDto): Promise<ResourceDto[]> {\n");
                    patterns.push_str("    return this.resourceService.findAll(query);\n");
                    patterns.push_str("  }\n");
                    patterns.push_str("  \n");
                    patterns.push_str("  @Post()\n");
                    patterns.push_str("  async create(@Body() createDto: CreateResourceDto): Promise<ResourceDto> {\n");
                    patterns.push_str("    return this.resourceService.create(createDto);\n");
                    patterns.push_str("  }\n");
                    patterns.push_str("}\n\n");
                    patterns.push_str("// Service Pattern\n");
                    patterns.push_str("@Injectable()\n");
                    patterns.push_str("export class ResourceService {\n");
                    patterns.push_str("  constructor(\n");
                    patterns.push_str("    @InjectRepository(Resource)\n");
                    patterns.push_str("    private resourceRepository: Repository<Resource>,\n");
                    patterns.push_str("  ) {}\n");
                    patterns.push_str("  \n");
                    patterns.push_str("  async findAll(query: QueryDto): Promise<ResourceDto[]> {\n");
                    patterns.push_str("    // Service logic here\n");
                    patterns.push_str("  }\n");
                    patterns.push_str("}\n");
                    patterns.push_str("```\n\n");
                },
                "Next.js" => {
                    patterns.push_str("#### Next.js App Router Patterns\n\n");
                    patterns.push_str("```typescript\n");
                    patterns.push_str("// app/page.tsx - Server Component\n");
                    patterns.push_str("export default async function Page() {\n");
                    patterns.push_str("  const data = await fetchData();\n");
                    patterns.push_str("  \n");
                    patterns.push_str("  return (\n");
                    patterns.push_str("    <main>\n");
                    patterns.push_str("      <ClientComponent data={data} />\n");
                    patterns.push_str("    </main>\n");
                    patterns.push_str("  );\n");
                    patterns.push_str("}\n\n");
                    patterns.push_str("// app/api/route.ts - API Route\n");
                    patterns.push_str("export async function GET(request: Request) {\n");
                    patterns.push_str("  const data = await processRequest();\n");
                    patterns.push_str("  return Response.json(data);\n");
                    patterns.push_str("}\n");
                    patterns.push_str("```\n\n");
                },
                _ => {}
            }
        }
        
        // Add general patterns based on component types
        if analysis.components.iter().any(|c| matches!(c.component_type, crate::core::ComponentType::Service)) {
            patterns.push_str("#### Service Layer Pattern\n\n");
            patterns.push_str("```typescript\n");
            patterns.push_str("// Service interface for consistency\n");
            patterns.push_str("interface BaseService<T, CreateDto, UpdateDto> {\n");
            patterns.push_str("  findAll(query?: QueryParams): Promise<T[]>;\n");
            patterns.push_str("  findById(id: string): Promise<T | null>;\n");
            patterns.push_str("  create(data: CreateDto): Promise<T>;\n");
            patterns.push_str("  update(id: string, data: UpdateDto): Promise<T>;\n");
            patterns.push_str("  delete(id: string): Promise<void>;\n");
            patterns.push_str("}\n");
            patterns.push_str("```\n\n");
        }
        
        patterns
    }

    fn generate_framework_conventions(analysis: &CodebaseAnalysis) -> String {
        let mut conventions = String::new();
        
        conventions.push_str("### Naming Conventions\n\n");
        
        // Analyze existing component names to infer conventions
        let component_names: Vec<&String> = analysis.components.iter().map(|c| &c.name).collect();
        
        // Determine naming pattern (PascalCase, camelCase, kebab-case, etc.)
        let pascal_case_count = component_names.iter()
            .filter(|name| name.chars().next().unwrap_or('a').is_uppercase())
            .count();
        
        let camel_case_count = component_names.iter()
            .filter(|name| name.chars().next().unwrap_or('A').is_lowercase() && name.contains(char::is_uppercase))
            .count();
            
        if pascal_case_count > camel_case_count {
            conventions.push_str("**Components**: Use PascalCase (e.g., `UserProfile`, `ProductList`)\n");
        } else {
            conventions.push_str("**Components**: Use camelCase (e.g., `userProfile`, `productList`)\n");
        }
        
        // File naming conventions based on framework
        for framework in &analysis.framework_analysis.detected_frameworks {
            match framework.name.as_str() {
                "React" | "Next.js" => {
                    conventions.push_str("**Files**: Use PascalCase for components (`.tsx`), camelCase for utilities (`.ts`)\n");
                    conventions.push_str("**Directories**: Use kebab-case for feature directories\n");
                },
                "NestJS" => {
                    conventions.push_str("**Files**: Use kebab-case with suffixes (`.controller.ts`, `.service.ts`, `.module.ts`)\n");
                    conventions.push_str("**Classes**: Use PascalCase with descriptive suffixes (`UserController`, `AuthService`)\n");
                },
                _ => {}
            }
        }
        
        conventions.push_str("\n### Code Organization\n\n");
        
        // Infer directory structure from component file paths
        let mut directories = std::collections::HashSet::new();
        for component in &analysis.components {
            if let Some(parent) = std::path::Path::new(&component.file_path).parent() {
                if let Some(dir_name) = parent.file_name() {
                    directories.insert(dir_name.to_string_lossy().to_string());
                }
            }
        }
        
        if directories.contains("components") {
            conventions.push_str("**Components**: Organize in `/components` directory with feature-based subdirectories\n");
        }
        if directories.contains("services") {
            conventions.push_str("**Services**: Place business logic in `/services` directory\n");
        }
        if directories.contains("utils") || directories.contains("lib") {
            conventions.push_str("**Utilities**: Shared utilities in `/utils` or `/lib` directory\n");
        }
        if directories.contains("types") {
            conventions.push_str("**Types**: TypeScript interfaces and types in `/types` directory\n");
        }
        
        conventions.push_str("\n### Error Handling Patterns\n\n");
        conventions.push_str("```typescript\n");
        conventions.push_str("// Consistent error handling\n");
        conventions.push_str("try {\n");
        conventions.push_str("  const result = await operation();\n");
        conventions.push_str("  return { success: true, data: result };\n");
        conventions.push_str("} catch (error) {\n");
        conventions.push_str("  console.error('Operation failed:', error);\n");
        conventions.push_str("  return { success: false, error: error.message };\n");
        conventions.push_str("}\n");
        conventions.push_str("```\n\n");
        
        conventions
    }

    fn generate_integration_guidelines(analysis: &CodebaseAnalysis) -> String {
        let mut guidelines = String::new();
        
        guidelines.push_str("### Safe Extension Principles\n\n");
        
        // Architecture-specific guidelines
        guidelines.push_str(&format!("**Current Architecture**: {}\n\n", 
            analysis.framework_analysis.architecture_pattern));
        
        guidelines.push_str("#### Adding New Features\n\n");
        guidelines.push_str("1. **Follow Existing Patterns**: Maintain consistency with current component structure\n");
        guidelines.push_str("2. **Respect Dependencies**: Check existing component dependencies before adding new ones\n");
        guidelines.push_str("3. **Preserve Interfaces**: Maintain existing API contracts and component interfaces\n");
        guidelines.push_str("4. **Test Integration**: Ensure new features integrate smoothly with existing functionality\n\n");
        
        // Framework-specific guidelines
        for framework in &analysis.framework_analysis.detected_frameworks {
            match framework.name.as_str() {
                "React" => {
                    guidelines.push_str("#### React-Specific Guidelines\n\n");
                    guidelines.push_str("- **State Management**: Use existing state management patterns (hooks, context, or external library)\n");
                    guidelines.push_str("- **Component Composition**: Prefer composition over inheritance\n");
                    guidelines.push_str("- **Props Interface**: Define clear TypeScript interfaces for all props\n");
                    guidelines.push_str("- **Side Effects**: Use `useEffect` appropriately with proper dependency arrays\n\n");
                },
                "NestJS" => {
                    guidelines.push_str("#### NestJS-Specific Guidelines\n\n");
                    guidelines.push_str("- **Dependency Injection**: Use constructor injection for services and repositories\n");
                    guidelines.push_str("- **Module Organization**: Create feature modules for related functionality\n");
                    guidelines.push_str("- **DTOs and Validation**: Use class-validator for request/response validation\n");
                    guidelines.push_str("- **Exception Handling**: Use built-in HTTP exceptions for consistent error responses\n\n");
                },
                "Next.js" => {
                    guidelines.push_str("#### Next.js-Specific Guidelines\n\n");
                    guidelines.push_str("- **Server vs Client**: Clearly distinguish between Server and Client Components\n");
                    guidelines.push_str("- **Data Fetching**: Use appropriate data fetching methods (async components, SWR, etc.)\n");
                    guidelines.push_str("- **Routing**: Follow file-based routing conventions in the `app` directory\n");
                    guidelines.push_str("- **API Routes**: Place API handlers in appropriate route files\n\n");
                },
                _ => {}
            }
        }
        
        guidelines.push_str("#### Integration Safety Checks\n\n");
        guidelines.push_str("```bash\n");
        guidelines.push_str("# Before integrating new code:\n");
        guidelines.push_str("npm run lint          # Check code style consistency\n");
        guidelines.push_str("npm run type-check    # Verify TypeScript compatibility\n");
        guidelines.push_str("npm run test          # Run existing test suite\n");
        guidelines.push_str("npm run build         # Ensure successful compilation\n");
        guidelines.push_str("```\n\n");
        
        // Configuration considerations
        if !analysis.integration_points.configuration_files.is_empty() {
            guidelines.push_str("#### Configuration Management\n\n");
            guidelines.push_str("**Existing Configuration Files**:\n");
            for config in analysis.integration_points.configuration_files.iter().take(5) {
                guidelines.push_str(&format!("- `{}`: {}\n", config.file_path, config.purpose));
            }
            guidelines.push_str("\n**Guidelines**:\n");
            guidelines.push_str("- Maintain configuration consistency across environments\n");
            guidelines.push_str("- Use environment variables for sensitive or environment-specific values\n");
            guidelines.push_str("- Document any new configuration requirements\n\n");
        }
        
        guidelines
    }

    fn generate_quality_standards(analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> String {
        let mut standards = String::new();
        
        standards.push_str("### Code Quality Requirements\n\n");
        
        // Infer quality standards from current codebase
        let avg_complexity = if !analysis.components.is_empty() {
            analysis.components.iter().map(|c| c.complexity_score as f32).sum::<f32>() / analysis.components.len() as f32
        } else {
            50.0
        };
        
        standards.push_str(&format!("**Complexity Threshold**: Keep component complexity below {} (current average: {:.1})\n", 
            (avg_complexity * 1.1) as u8, avg_complexity));
        
        if let Some(intel) = intelligent_analysis {
            standards.push_str(&format!("**Maintainability Target**: Achieve {:.1}% maintainability score (current: {:.1}%)\n", 
                (intel.quality_metrics.maintainability * 1.05) * 100.0,
                intel.quality_metrics.maintainability * 100.0));
                
            if intel.quality_metrics.test_coverage_estimate > 0.0 {
                standards.push_str(&format!("**Test Coverage**: Maintain minimum {}% test coverage (current: {:.1}%)\n",
                    ((intel.quality_metrics.test_coverage_estimate * 100.0) as u8).max(70),
                    intel.quality_metrics.test_coverage_estimate * 100.0));
            } else {
                standards.push_str("**Test Coverage**: Implement minimum 70% test coverage for new features\n");
            }
        } else {
            standards.push_str("**Test Coverage**: Implement comprehensive testing for all new features\n");
        }
        
        standards.push_str("\n### Documentation Standards\n\n");
        
        // Check for existing documentation patterns
        let has_jsdoc = analysis.components.iter()
            .any(|c| c.purpose.len() > 20); // Assuming longer purposes indicate better documentation
            
        if has_jsdoc {
            standards.push_str("**Code Documentation**: Follow existing JSDoc/TSDoc patterns for all public interfaces\n");
        } else {
            standards.push_str("**Code Documentation**: Add comprehensive JSDoc/TSDoc comments for all exported functions and components\n");
        }
        
        standards.push_str("**README Updates**: Update project README for any new features or configuration changes\n");
        standards.push_str("**API Documentation**: Document all API endpoints and data schemas\n");
        
        standards.push_str("\n### Testing Requirements\n\n");
        
        // Framework-specific testing guidance
        for framework in &analysis.framework_analysis.detected_frameworks {
            match framework.name.as_str() {
                "React" => {
                    standards.push_str("**React Testing**: Use React Testing Library for component tests\n");
                    standards.push_str("- Test component rendering and user interactions\n");
                    standards.push_str("- Mock external dependencies and API calls\n");
                    standards.push_str("- Test accessibility and keyboard navigation\n\n");
                },
                "NestJS" => {
                    standards.push_str("**NestJS Testing**: Implement unit and integration tests\n");
                    standards.push_str("- Unit tests for services and controllers\n");
                    standards.push_str("- Integration tests for API endpoints\n");
                    standards.push_str("- Mock database interactions appropriately\n\n");
                },
                _ => {}
            }
        }
        
        standards.push_str("**Test Organization**: Place tests adjacent to source files or in `__tests__` directories\n");
        standards.push_str("**Continuous Integration**: Ensure all tests pass in CI/CD pipeline\n\n");
        
        standards.push_str("### Performance Standards\n\n");
        
        if analysis.framework_analysis.detected_frameworks.iter().any(|f| f.name == "React" || f.name == "Next.js") {
            standards.push_str("**Frontend Performance**:\n");
            standards.push_str("- Bundle size analysis with each build\n");
            standards.push_str("- Lighthouse score targets: Performance > 90, Accessibility > 95\n");
            standards.push_str("- Implement code splitting for large components\n\n");
        }
        
        if analysis.framework_analysis.detected_frameworks.iter().any(|f| f.name == "NestJS" || f.name == "Express.js") {
            standards.push_str("**Backend Performance**:\n");
            standards.push_str("- API response times < 200ms for standard operations\n");
            standards.push_str("- Implement appropriate caching strategies\n");
            standards.push_str("- Database query optimization\n\n");
        }
        
        standards
    }
}

impl DocumentGenerator for ClaudeSpecContextGenerator {
    fn generate(&self, analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> Result<String> {
        let mut content = String::new();
        
        // Header
        content.push_str("# Development Context for Claude Code Spec Workflow\n\n");
        content.push_str("This document provides comprehensive context for extending the existing codebase ");
        content.push_str("while maintaining consistency with established patterns and architecture.\n\n");
        
        // Import metadata for context
        content.push_str("## CODEBASE_CONTEXT\n\n");
        content.push_str(&format!("**Project Type**: {}\n", analysis.business_context.inferred_product_type));
        content.push_str(&format!("**Architecture Pattern**: {}\n", analysis.framework_analysis.architecture_pattern));
        content.push_str(&format!("**Implementation Status**: {:.1}% complete\n", 
            analysis.status_intelligence.overall_completion_percentage));
        content.push_str(&format!("**Total Components**: {}\n", analysis.components.len()));
        
        if !analysis.framework_analysis.detected_frameworks.is_empty() {
            content.push_str("**Active Frameworks**: ");
            let framework_names: Vec<String> = analysis.framework_analysis.detected_frameworks
                .iter()
                .map(|f| format!("{} v{}", f.name, f.version.as_deref().unwrap_or("latest")))
                .collect();
            content.push_str(&framework_names.join(", "));
            content.push_str("\n");
        }
        content.push_str("\n");
        
        // Existing codebase patterns section
        content.push_str("## EXISTING_CODEBASE_PATTERNS\n\n");
        content.push_str(&Self::extract_code_patterns(analysis));
        
        // Framework conventions section  
        content.push_str("## FRAMEWORK_CONVENTIONS\n\n");
        content.push_str(&Self::generate_framework_conventions(analysis));
        
        // Integration guidelines section
        content.push_str("## INTEGRATION_GUIDELINES\n\n");
        content.push_str(&Self::generate_integration_guidelines(analysis));
        
        // Quality standards section
        content.push_str("## QUALITY_STANDARDS\n\n");
        content.push_str(&Self::generate_quality_standards(analysis, intelligent_analysis));
        
        // Current implementation status for context
        content.push_str("## CURRENT_IMPLEMENTATION_STATUS\n\n");
        
        if !analysis.status_intelligence.completed_features.is_empty() {
            content.push_str(&format!("### Implemented Features ({} items)\n\n", 
                analysis.status_intelligence.completed_features.len()));
            for feature in analysis.status_intelligence.completed_features.iter().take(10) {
                content.push_str(&format!("- **{}**: {}\n", feature.name, 
                    feature.description.chars().take(100).collect::<String>()));
            }
            content.push_str("\n");
        }
        
        if !analysis.status_intelligence.in_progress_features.is_empty() {
            content.push_str(&format!("### In Progress Features ({} items)\n\n", 
                analysis.status_intelligence.in_progress_features.len()));
            for feature in analysis.status_intelligence.in_progress_features.iter().take(5) {
                content.push_str(&format!("- **{}**: {}\n", feature.name, 
                    feature.description.chars().take(100).collect::<String>()));
            }
            content.push_str("\n");
        }
        
        if !analysis.status_intelligence.todo_features.is_empty() {
            content.push_str(&format!("### Planned Features ({} items)\n\n", 
                analysis.status_intelligence.todo_features.len()));
            for feature in analysis.status_intelligence.todo_features.iter().take(5) {
                content.push_str(&format!("- **{}**: {}\n", feature.name, 
                    feature.description.chars().take(100).collect::<String>()));
            }
            content.push_str("\n");
        }
        
        // Component relationship context
        if !analysis.implementation_analysis.component_relationships.is_empty() {
            content.push_str("## COMPONENT_RELATIONSHIPS\n\n");
            content.push_str("Understanding existing component dependencies is crucial for safe extension:\n\n");
            
            for relationship in analysis.implementation_analysis.component_relationships.iter().take(10) {
                content.push_str(&format!("- **{}** {} **{}** ({})\n", 
                    relationship.source, 
                    relationship.relationship_type, 
                    relationship.target,
                    relationship.description));
            }
            content.push_str("\n");
        }
        
        // Development workflow guidance
        content.push_str("## DEVELOPMENT_WORKFLOW\n\n");
        content.push_str("### Recommended Development Process\n\n");
        content.push_str("1. **Analysis Phase**:\n");
        content.push_str("   - Review existing patterns and conventions above\n");
        content.push_str("   - Identify integration points with current components\n");
        content.push_str("   - Plan feature implementation to maintain architectural consistency\n\n");
        
        content.push_str("2. **Implementation Phase**:\n");
        content.push_str("   - Follow established naming conventions and file organization\n");
        content.push_str("   - Implement comprehensive tests following existing patterns\n");
        content.push_str("   - Ensure TypeScript compatibility and type safety\n\n");
        
        content.push_str("3. **Integration Phase**:\n");
        content.push_str("   - Run quality checks and tests\n");
        content.push_str("   - Validate integration with existing components\n");
        content.push_str("   - Update documentation and configuration as needed\n\n");
        
        // Environment setup
        if !analysis.integration_points.configuration_files.is_empty() {
            content.push_str("### Environment Setup Requirements\n\n");
            for config in analysis.integration_points.configuration_files.iter().take(3) {
                if config.file_path.contains("package.json") {
                    content.push_str("- **Dependencies**: Run `npm install` or `yarn install`\n");
                } else if config.file_path.contains(".env") {
                    content.push_str(&format!("- **Environment**: Configure {} for local development\n", config.file_path));
                } else if config.file_path.contains("tsconfig") {
                    content.push_str("- **TypeScript**: Project uses TypeScript - ensure type safety\n");
                }
            }
            content.push_str("\n");
        }
        
        // Technical debt awareness
        if !analysis.status_intelligence.technical_debt.is_empty() {
            content.push_str("## TECHNICAL_DEBT_AWARENESS\n\n");
            content.push_str("Be aware of these existing technical considerations:\n\n");
            for debt in analysis.status_intelligence.technical_debt.iter().take(5) {
                content.push_str(&format!("- **{}** ({}): {}\n", 
                    debt.severity, debt.location, debt.description));
                if !debt.recommendation.is_empty() {
                    content.push_str(&format!("  *Recommendation*: {}\n", debt.recommendation));
                }
            }
            content.push_str("\n");
        }
        
        // Footer with usage instructions
        content.push_str("---\n\n");
        content.push_str("**Usage Instructions**: This context document should be provided to Claude Code ");
        content.push_str("when requesting code modifications or extensions. It ensures that new code ");
        content.push_str("maintains consistency with existing patterns and follows established conventions.\n\n");
        
        content.push_str("**Context Refresh**: Re-generate this document after significant architectural ");
        content.push_str("changes or when adding new frameworks to ensure context remains current.\n");
        
        Ok(content)
    }

    fn get_file_extension(&self) -> &str {
        "md"
    }

    fn get_document_type(&self) -> DocumentType {
        DocumentType::ClaudeSpecContext
    }
}