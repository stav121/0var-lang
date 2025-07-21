//! The zvar programming language compiler and runtime
//!
//! A bytecode programming language that uses numbered variables and eliminates naming.

pub mod cli;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod symbol_table;
pub mod types;
pub mod vm;

// Re-export commonly used types
pub use error::{ZvarError, ZvarResult};
pub use span::Span;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the zvar compiler
pub fn init() {
    // Any global initialization can go here
}

/// Convenience function to compile and run zvar source code
pub fn run_source(source: &str) -> ZvarResult<()> {
    let mut symbol_table = symbol_table::SymbolTable::new();
    let mut parser = parser::Parser::new(source, &mut symbol_table)?;
    let program = parser.parse_program()?;

    let mut codegen = codegen::CodeGenerator::new();
    let (bytecode, debug_info) = codegen.generate(&program, &symbol_table)?;

    let mut vm = vm::VM::new();
    vm.load(bytecode, Some(debug_info));
    vm.run()?;

    Ok(())
}

/// Convenience function to compile zvar source to bytecode
pub fn compile_source(
    source: &str,
) -> ZvarResult<(
    codegen::instruction::Bytecode,
    codegen::debug_info::DebugInfo,
)> {
    let mut symbol_table = symbol_table::SymbolTable::new();
    let mut parser = parser::Parser::new(source, &mut symbol_table)?;
    let program = parser.parse_program()?;

    let mut codegen = codegen::CodeGenerator::new();
    codegen.generate(&program, &symbol_table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "zvar-lang");
    }

    #[test]
    fn test_simple_program() {
        let source = r#"
        main {
            int v$0 = 42;
            print(v$0);
        }
        "#;

        // This should compile and run without error
        let result = run_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_arithmetic_program() {
        let source = r#"
        main {
            int v$0 = 10;
            int v$1 = 5;
            v$0 = v$0 + v$1;
            print(v$0);
        }
        "#;

        let result = run_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_program() {
        let source = r#"
        fn f$0(v$0 int, v$1 int) -> int {
            ret v$0 + v$1;
        }

        main {
            int v$2 = f$0(5, 3);
            print(v$2);
        }
        "#;

        let result = run_source(source);
        assert!(result.is_ok());
    }
}
