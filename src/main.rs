//! Main entry point for the zvar compiler

use std::{fs, process};
use zvar_lang::{
    cli::{Cli, Commands},
    codegen::CodeGenerator,
    error::{ZvarError, ZvarResult},
    parser::Parser,
    symbol_table::SymbolTable,
    vm::VM,
};

fn main() {
    let cli = Cli::parse_args();

    // Validate file extension if applicable
    if let Err(e) = cli.validate_file_extension() {
        eprintln!("Error: {}", e);
        eprintln!("Supported file extensions: {}", Cli::supported_extensions());
        process::exit(1);
    }

    if let Err(e) = run_command(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_command(cli: Cli) -> ZvarResult<()> {
    match cli.command {
        Commands::Run {
            file,
            disasm,
            debug,
        } => run_file(&file, disasm, debug || cli.verbose),
        Commands::Compile {
            file,
            output,
            disasm,
        } => compile_file(&file, output.as_deref(), disasm),
        Commands::Check { file } => check_file(&file),
        Commands::Info { file, docs_only } => show_info(&file, docs_only),
        Commands::Repl { show_bytecode } => run_repl(show_bytecode),
    }
}

fn run_file(file: &std::path::Path, show_disasm: bool, debug: bool) -> ZvarResult<()> {
    if debug {
        println!(
            "Running file: {} (extension: {})",
            file.display(),
            file.extension().and_then(|e| e.to_str()).unwrap_or("none")
        );
    }

    // Read source code
    let source = fs::read_to_string(file).map_err(|e| {
        ZvarError::file_error(format!("Failed to read file {}: {}", file.display(), e))
    })?;

    // Compile to bytecode
    let mut symbol_table = SymbolTable::new();
    let mut parser = Parser::new(&source, &mut symbol_table)?;
    let program = parser.parse_program()?;

    if debug {
        println!("Parsed {} top-level items", program.items.len());
    }

    let mut codegen = CodeGenerator::new();
    let (bytecode, debug_info) = codegen.generate(&program, &symbol_table)?;

    if show_disasm {
        println!("\n{}", bytecode.disassemble());
    }

    if debug {
        println!("Generated {} instructions", bytecode.len());
    }

    // Execute
    let mut vm = VM::new();
    vm.load(bytecode, Some(debug_info));

    if debug {
        println!("Starting execution...\n");
    }

    vm.run()?;

    if debug {
        println!("\nExecution completed successfully");
    }

    Ok(())
}

fn compile_file(
    file: &std::path::Path,
    output: Option<&std::path::Path>,
    show_disasm: bool,
) -> ZvarResult<()> {
    println!("Compiling file: {}", file.display());

    // Read source code
    let source = fs::read_to_string(file).map_err(|e| {
        ZvarError::file_error(format!("Failed to read file {}: {}", file.display(), e))
    })?;

    // Compile to bytecode
    let mut symbol_table = SymbolTable::new();
    let mut parser = Parser::new(&source, &mut symbol_table)?;
    let program = parser.parse_program()?;

    let mut codegen = CodeGenerator::new();
    let (bytecode, _debug_info) = codegen.generate(&program, &symbol_table)?;

    if show_disasm {
        println!("\n{}", bytecode.disassemble());
    }

    // In a real implementation, we'd serialize the bytecode to the output file
    if let Some(output_path) = output {
        println!("Would write bytecode to: {}", output_path.display());
        // TODO: Implement bytecode serialization
    } else {
        println!(
            "Compilation successful - {} instructions generated",
            bytecode.len()
        );
    }

    Ok(())
}

fn check_file(file: &std::path::Path) -> ZvarResult<()> {
    println!("Checking file: {}", file.display());

    // Read source code
    let source = fs::read_to_string(file).map_err(|e| {
        ZvarError::file_error(format!("Failed to read file {}: {}", file.display(), e))
    })?;

    // Parse only (don't generate code)
    let mut symbol_table = SymbolTable::new();
    let mut parser = Parser::new(&source, &mut symbol_table)?;
    let program = parser.parse_program()?;

    println!("✓ Syntax is valid");
    println!("✓ Found {} top-level items", program.items.len());

    // Show basic statistics
    let mut functions = 0;
    let mut main_blocks = 0;

    for item in &program.items {
        match item {
            zvar_lang::parser::ast::Item::Function(_) => functions += 1,
            zvar_lang::parser::ast::Item::MainBlock(_) => main_blocks += 1,
        }
    }

    println!("✓ {} functions, {} main blocks", functions, main_blocks);

    Ok(())
}

fn show_info(file: &std::path::Path, docs_only: bool) -> ZvarResult<()> {
    println!("Analyzing file: {}", file.display());

    // Read source code
    let source = fs::read_to_string(file).map_err(|e| {
        ZvarError::file_error(format!("Failed to read file {}: {}", file.display(), e))
    })?;

    // Parse and analyze
    let mut symbol_table = SymbolTable::new();
    let mut parser = Parser::new(&source, &mut symbol_table)?;
    let _program = parser.parse_program()?;

    println!("\nEntity Information:");
    println!("{:-<50}", "");

    for (name, symbol) in symbol_table.all_symbols() {
        if !docs_only {
            println!(
                "{}: {} (defined at {})",
                name,
                match &symbol.entity_type {
                    zvar_lang::symbol_table::EntityType::Variable { value_type } =>
                        format!("{} variable", value_type),
                    zvar_lang::symbol_table::EntityType::Constant { value_type } =>
                        format!("{} constant", value_type),
                    zvar_lang::symbol_table::EntityType::Function {
                        params,
                        return_type,
                    } => format!("function({} params) -> {}", params.len(), return_type),
                },
                symbol.definition_span
            );
        }

        if let Some(doc) = &symbol.documentation {
            println!("  Documentation: {}", doc);
        }

        if !docs_only {
            println!();
        }
    }

    Ok(())
}

fn run_repl(show_bytecode: bool) -> ZvarResult<()> {
    println!("zvar REPL - Interactive mode");
    println!("Type expressions to evaluate them, or 'exit' to quit");
    println!("{:-<50}", "");

    let mut symbol_table = SymbolTable::new();
    let mut vm = VM::new();

    loop {
        print!("> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                if input == "exit" || input == "quit" {
                    println!("Goodbye!");
                    break;
                }

                // Wrap the input in a main block for parsing
                let wrapped_input = format!("main {{ {} }}", input);

                match evaluate_repl_input(&wrapped_input, &mut symbol_table, &mut vm, show_bytecode)
                {
                    Ok(()) => {}
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn evaluate_repl_input(
    input: &str,
    symbol_table: &mut SymbolTable,
    vm: &mut VM,
    show_bytecode: bool,
) -> ZvarResult<()> {
    // Parse the input
    let mut parser = Parser::new(input, symbol_table)?;
    let program = parser.parse_program()?;

    // Generate bytecode
    let mut codegen = CodeGenerator::new();
    let (bytecode, debug_info) = codegen.generate(&program, symbol_table)?;

    if show_bytecode {
        println!("{}", bytecode.disassemble());
    }

    // Execute
    vm.reset();
    vm.load(bytecode, Some(debug_info));
    vm.run()?;

    Ok(())
}
