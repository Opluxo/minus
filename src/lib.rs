pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;

use error::Result;
use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;
use codegen::CodeGenerator;

pub fn compile(source: &str) -> Result<String> {
    // Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)?;

    // Code generation
    let mut codegen = CodeGenerator::new();
    let llvm_ir = codegen.generate(&program)?;

    Ok(llvm_ir)
}

pub fn compile_to_file(source: &str, output_path: &str) -> Result<()> {
    let llvm_ir = compile(source)?;
    std::fs::write(output_path, llvm_ir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let source = r#"
            void main() {
                Print("Hello, World!");
            }
        "#;

        let result = compile(source);
        if let Err(ref e) = result {
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_variable_and_arithmetic() {
        let source = r#"
            void main() {
                int x = 10;
                int y = 20;
                int z = x + y;
                Print(z);
            }
        "#;

        let result = compile(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_definition() {
        let source = r#"
            int add(int a, int b) {
                return a + b;
            }
            
            void main() {
                int result = add(5, 3);
                Print(result);
            }
        "#;

        let result = compile(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_else() {
        let source = r#"
            void main() {
                int x = 10;
                if (x > 5) {
                    Print("greater");
                } else {
                    Print("less or equal");
                }
            }
        "#;

        let result = compile(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop() {
        let source = r#"
            void main() {
                for (int i = 0; i < 10; i = i + 1) {
                    Print(i);
                }
            }
        "#;

        let result = compile(source);
        if let Err(ref e) = result {
            eprintln!("For loop error: {}", e);
        }
        assert!(result.is_ok());
    }
}
