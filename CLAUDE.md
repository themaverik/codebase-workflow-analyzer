# Codebase Workflow Analyzer

## Project Overview
Open source tool that reverse engineers existing codebases into systematic development workflows, generating PRDs, user stories, and task breakdowns with intelligent status detection.

## Architecture
- **Language:** Rust (core) + TypeScript (web/integrations)  
- **Pattern:** Monolith-first with modular design
- **Target Languages:** Java, TypeScript, Python
- **Framework Detection:** Spring Boot, React, Django/Flask

## Development Priorities
1. TypeScript/React analyzer with component detection
2. Intelligent user story extraction from code patterns
3. PRD generation with business context inference
4. Task status detection (complete/in-progress/todo)
5. CLI interface with file system analysis

## Code Standards
- Security-first design with input validation
- Clear domain boundaries and error handling
- Plugin-ready architecture for future extensions
- Comprehensive testing for analysis accuracy