use super::quadruple_gen::generator::QuadrupleGenerator;

pub struct CodeGenerator {
    pub quadrupl: QuadrupleGenerator,
    pub other_field: String,
}

impl CodeGenerator {
    
    pub fn new(quadrupl: QuadrupleGenerator, other_field: String) -> Self {
        CodeGenerator { quadrupl, other_field }
    }


    pub fn generate_code(&self) {

    }
}