use clap::Arg;
use logos::source;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::parser::ast::Program;

use super::assambly_gen::generator::AssemblyGenerator;
use super::quadruple_gen::generator::QuadrupleGenerator;
pub struct CodeGenerator {
    pub quadrupl_gen: QuadrupleGenerator,
    pub assembly_gen: AssemblyGenerator,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            quadrupl_gen: QuadrupleGenerator::new(),
            assembly_gen: AssemblyGenerator::new(),
        }
    }

    pub fn generate_code(
        &mut self,
        program: &Program,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {

        // Generate code for the program
        self.quadrupl_gen.generate_quadruples(program);
        // Generate assembly code from the quadruple program
        self.assembly_gen.get_assambly(&self.quadrupl_gen.program);

        let asm_path = output_path.with_extension("asm");
        let obj_path = output_path.with_extension("o");

        let asm_file_path = output_path.with_extension("asm");
        match fs::write(
            &asm_file_path,
            self.assembly_gen.get_assambly(&self.quadrupl_gen.program),
        ) {
            Ok(_) => println!("Assembly written to {}", asm_file_path.display()),
            Err(e) => {
                println!("Failed to write assembly to file: {}", e);
                return Err(Box::new(e)); // Updated to return the error as a Box<dyn std::error::Error>
            }
        }

        // Assemble and link
        self.assemble_and_link(&asm_path, &obj_path, output_path)?;

        println!("Code generation completed successfully.");

        Ok(())
    }
    
    fn assemble_and_link(
        &self,
        asm_path: &Path,
        obj_path: &Path,
        exe_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let nasm_status = Command::new("nasm")
            .arg("-f")
            .arg("elf64")
            .arg(asm_path.to_str().unwrap())
            .arg("-o")
            .arg(obj_path.to_str().unwrap())
            .status()?;
        println!("NASM Status: {:?}", nasm_status);

        if !nasm_status.success() {
            return Err("NASM assembly failed".into());
        }

        // Use ld directly since we're not using C library
        let ld_status = Command::new("ld")
            .arg("-o")
            .arg(exe_path)
            .arg(obj_path)
            .status()?;

        println!("LD Status: {:?}", ld_status);
        if !ld_status.success() {
            return Err("Linking failed".into());
        }

        Ok(())
    }
}
