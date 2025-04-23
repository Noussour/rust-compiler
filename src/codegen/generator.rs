use crate::parser::ast::Program;

use super::quadruple_gen::{generator::QuadrupleGenerator, quadruple::QuadrupleProgram};

pub struct CodeGenerator {
    pub quadrupl: QuadrupleGenerator,
    pub other_field: String,
}

impl CodeGenerator {
    
    pub fn new() -> Self {
        CodeGenerator { quadrupl: QuadrupleGenerator::new(), other_field: String::new() }
    }


    pub fn generate_code(&mut self, program: &Program) -> Option<QuadrupleProgram> {
        // Generate code for the program
        let quadruple_program = self.quadrupl.generate_code(program)?;
        Some(quadruple_program)
    }

    pub fn additional_method(&self) {

    }
}