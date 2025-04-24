use super::super::quadruple_gen::quadruple::QuadrupleProgram;
use std::collections::{HashMap, HashSet};

pub struct AssemblyGenerator {
    pub instructions: Vec<String>,
    pub data_section: Vec<String>,
    pub bss_section: Vec<String>,
    pub label_map: HashMap<usize, String>,
    pub temp_label_counter: usize,
    pub float_constants: HashMap<u32, String>, // Changed to HashMap to track float constants by value
    pub float_counter: usize,
    pub string_literals: HashMap<String, String>,
    pub string_counter: usize,
    pub defined_variables: HashSet<String>, // Track defined variables
    pub defined_labels: HashSet<String>,    // Track defined labels
    pub current_operation_is_comparison: bool,
    
}

impl AssemblyGenerator {
    pub fn new() -> Self {
        AssemblyGenerator {
            instructions: Vec::new(),
            data_section: Vec::new(),
            bss_section: Vec::new(),
            label_map: HashMap::new(),
            temp_label_counter: 0,
            float_constants: HashMap::new(),
            string_literals: HashMap::new(),
            string_counter: 0,
            defined_variables: HashSet::new(),
            defined_labels: HashSet::new(),
            float_counter: 0,
            current_operation_is_comparison: false,
            
        }
    }

    pub fn generate(&mut self, program: &QuadrupleProgram) {
        // Clear previous state to prevent redefinitions
        *self = AssemblyGenerator::new();

        // 1) Process declarations into data & bss
        self.data_section.insert(0, "section .data".into());
        self.bss_section.insert(0, "section .bss".into());

        self.process_declarations(program);

        // 3) Start text section
        self.instructions.push("section .text".into());
        self.generate_program_start();

        // 5) Main body from quadruples
        self.process_operations(program);

        // 6) Exit syscall epilogue
        self.generate_program_end();

        // 7) Utility routines after main
        self.add_utility_functions();
    }

    pub fn generate_program_start(&mut self) {
        self.instructions.push("global _start".to_string());
        self.instructions.push("_start:".to_string());
        self.instructions.push("    push rbp".to_string());
        self.instructions.push("    mov rbp, rsp".to_string());
        self.instructions.push("    sub rsp, 1024".to_string());
    }

    pub fn generate_program_end(&mut self) {
        // Use 64-bit syscall convention for exit
        self.instructions.push("    mov rax, 60".to_string()); // syscall number for exit
        self.instructions.push("    xor rdi, rdi".to_string()); // exit code 0 
        self.instructions.push("    syscall".to_string()); // use syscall instruction
    }

    pub fn print_instructions(&self) {
        for line in &self.instructions {
            println!("{}", line);
        }
    }
    pub fn print_data_section(&self) {
        for line in &self.data_section {
            println!("{}", line);
        }
    }
    pub fn print_bss_section(&self) {
        for line in &self.bss_section {
            println!("{}", line);
        }
    }
    pub fn print_assambly(&mut self, program: &QuadrupleProgram) {
        self.generate(program);
        self.print_data_section();
        self.print_bss_section();
        self.print_instructions();
    }

    pub fn get_assambly(&mut self, program: &QuadrupleProgram) -> String {
        self.generate(program);
        let mut result = String::new();
        result.push_str(&self.data_section.join("\n"));
        result.push_str("\n");
        result.push_str(&self.bss_section.join("\n"));
        result.push_str("\n");
        result.push_str(&self.instructions.join("\n"));
        result.push_str("\n");
        result.trim().to_string()
    }
}
