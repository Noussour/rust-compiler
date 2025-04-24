use super::super::super::parser::ast::Type;
use super::super::quadruple_gen::quadruple::{Operand, Operation, Quadruple, QuadrupleProgram};
use super::generator::AssemblyGenerator;

impl AssemblyGenerator {
    pub fn process_declarations(&mut self, program: &QuadrupleProgram) {
        for quad in &program.quadruples {
            match &quad.operation {
                Operation::DeclareVariable(typ) => {
                    if let Operand::Variable(name) = &quad.result {
                        if !self.defined_variables.contains(name) {
                            let directive = self.get_type_directive(typ);
                            if Operand::Empty != quad.operand1 {
                                let init = match &quad.operand1 {
                                    Operand::IntLiteral(v) => self.operand_to_asm(&quad.operand1),
                                    Operand::FloatLiteral(v) => self.operand_to_asm(&quad.operand1),
                                    _ => "".into(),
                                };
                                self.data_section
                                    .push(format!("{}: {} {}", name, directive, init));
                            } else {
                                // Reserve space in BSS if no initial value
                                let size = self.get_type_size(typ);
                                let directive = self.get_reserve_directive(typ);
                                self.bss_section
                                    .push(format!("{} {} {}", name, directive, size));
                            }
                            self.defined_variables.insert(name.clone());
                        }
                    }
                }

                Operation::DeclareArray(typ, size) => {
                    if let Operand::ArrayVariable(name, _) = &quad.result {
                        if !self.defined_variables.contains(name) {
                            // Reserve space in BSS if no initial value
                            let size = self.get_type_size(typ) * size;
                            let directive = self.get_reserve_directive(typ);
                            self.bss_section
                                .push(format!("{} {} {}", name, directive, size));
                        }
                        self.defined_variables.insert(name.clone());
                    }
                }
                _ => {}
            }
        }
    }

    pub fn process_operations(&mut self, program: &QuadrupleProgram) {
        // Build label map
        for quad in &program.quadruples {
            if let Operation::Label(id) = quad.operation {
                self.label_map.insert(id, format!("L{}", id));
            }
        }
        // Generate instructions
        for quad in &program.quadruples {
            self.quad_to_instructions(quad);
        }
    }

    fn quad_to_instructions(&mut self, quad: &Quadruple) {
        match &quad.operation {
            Operation::Label(id) => {
                if let Some(lbl) = self.label_map.get(id) {
                    self.instructions.push(format!("{}:", lbl));
                }
            }

            // Integer and Float operations unified
            Operation::Assign => {
                let src = self.operand_to_asm(&quad.operand1);

                let dst = self.operand_to_asm(&quad.result);
                self.instructions.push(format!("mov rax, {}", src));
                self.instructions.push(format!("mov {}, rax", dst));
            }

            Operation::Add | Operation::Subtract | Operation::Multiply | Operation::Divide => {
                let is_float = matches!(&quad.operand1, Operand::FloatLiteral(_))
                    || matches!(&quad.operand2, Operand::FloatLiteral(_));
                let a1 = self.operand_to_asm(&quad.operand1);
                let a2 = self.operand_to_asm(&quad.operand2);
                let res = self.operand_to_asm(&quad.result);
                if is_float {
                    // use x87 FPU for float
                    self.instructions.push(format!("fld dword {}", a1));
                    match &quad.operation {
                        Operation::Add => self.instructions.push(format!("fadd dword {}", a2)),
                        Operation::Subtract => self.instructions.push(format!("fsub dword {}", a2)),
                        Operation::Multiply => self.instructions.push(format!("fmul dword {}", a2)),
                        Operation::Divide => self.instructions.push(format!("fdiv dword {}", a2)),
                        _ => {}
                    }
                    self.instructions.push(format!("fstp dword {}", res));
                } else {
                    self.instructions.push(format!("mov rax, {}", a1));
                    match &quad.operation {
                        Operation::Add => self.instructions.push(format!("add rax, {}", a2)),
                        Operation::Subtract => self.instructions.push(format!("sub rax, {}", a2)),
                        Operation::Multiply => self.instructions.push(format!("imul rax, {}", a2)),
                        Operation::Divide => {
                            self.instructions.push(format!("cqo")); // 64-bit version of cdq
                            self.instructions.push(format!("mov rbx, {}", a2));
                            self.instructions.push(format!("idiv rbx"));
                        }
                        _ => {}
                    }
                    self.instructions.push(format!("mov {}, rax", res));
                }
            }

            // Delegate other ops to helper
            _ => self.generate_other(quad),
        }
    }

    /// Handles all operations except primitive arithmetic and labels
    fn generate_other(&mut self, quad: &Quadruple) {
        match &quad.operation {
            Operation::Jump(id) => {
                if let Some(lbl) = self.label_map.get(id) {
                    self.instructions.push(format!("jmp {}", lbl));
                }
            }
            Operation::JumpIfTrue(id) => {
                let cond = self.operand_to_asm(&quad.operand1);
                self.instructions.push(format!("mov rax, {}", cond));
                self.instructions.push("cmp rax, 0".into());
                if let Some(lbl) = self.label_map.get(id) {
                    self.instructions.push(format!("jnz {}", lbl));
                }
            }
            Operation::JumpIfFalse(id) => {
                let cond = self.operand_to_asm(&quad.operand1);
                self.instructions.push(format!("mov rax, {}", cond));
                self.instructions.push("cmp rax, 0".into());
                if let Some(lbl) = self.label_map.get(id) {
                    self.instructions.push(format!("jz {}", lbl));
                }
            }
            Operation::Equal
            | Operation::NotEqual
            | Operation::LessThan
            | Operation::GreaterThan
            | Operation::LessEqual
            | Operation::GreaterEqual => {
                self.gen_comparison(quad);
            }
            Operation::ArrayStore => {
                if let Operand::Variable(arr) = &quad.result {
                    let val = self.operand_to_asm(&quad.operand1);
                    let idx = self.operand_to_asm(&quad.operand2);
                    self.instructions.extend([
                        "push rax".into(),
                        "push rbx".into(),
                        "push rcx".into(),
                        format!("mov rax, {}", idx),
                        "mov rbx, 8".into(), // 8 bytes for 64-bit values
                        "imul rax, rbx".into(),
                        format!("mov rbx, {}", arr),
                        format!("mov rcx, {}", val),
                        format!("mov [rbx+rax], rcx"),
                        "pop rcx".into(),
                        "pop rbx".into(),
                        "pop rax".into(),
                    ]);
                }
            }
            Operation::ArrayLoad => {
                if let Operand::Variable(arr) = &quad.operand1 {
                    let idx = self.operand_to_asm(&quad.operand2);
                    let dst = self.operand_to_asm(&quad.result);
                    self.instructions.extend([
                        "push rax".into(),
                        "push rbx".into(),
                        "push rdx".into(),
                        format!("mov rax, {}", idx),       // rax = index
                        "shl rax, 3".into(),               // rax *= 8 (64-bit word size)
                        format!("lea rbx, [rel {}]", arr), // rbx = address of array
                        "mov rdx, [rbx + rax]".into(),     // rdx = array[index]
                        format!("mov {}, rdx", dst),       // store into destination
                        "pop rdx".into(),
                        "pop rbx".into(),
                        "pop rax".into(),
                    ]);
                }
            }

            Operation::Output => {
                match &quad.operand1 {
                    Operand::StringLiteral(_) => {
                        // Handle string literals
                        let v = self.operand_to_asm(&quad.operand1);
                        self.instructions.extend([
                            format!("lea rax, [{}]", v),
                            format!("push rax"),
                            "call print_string".into(),
                            "pop rax".into(),
                        ]);
                    }
                    Operand::FloatLiteral(_) => {
                        // Handle float literals
                        let v = self.operand_to_asm(&quad.operand1);
                        self.instructions.extend([
                            format!("mov rax, {}", v),
                            format!("push rax"),
                            "call print_int".into(), // Temporary, should be print_float when implemented
                            "pop rax".into(),
                        ]);
                    }
                    _ => {
                        // Handle variables and other values
                        let v = self.operand_to_asm(&quad.operand1);
                        self.instructions.extend([
                            format!("mov rax, {}", v), // Load the value, not the address
                            format!("push rax"),
                            "call print_int".into(),
                            "pop rax".into(),
                        ]);
                    }
                }
            }

            Operation::Input => {
                let dst = self.operand_to_asm(&quad.result);
                self.instructions
                    .extend(["call read_int".into(), format!("mov {}, rax", dst)]);
            }
            Operation::Call(n) => self.instructions.push(format!("call {}", n)),
            Operation::Return => self.instructions.push("ret".into()),
            Operation::DeclareVariable(_) | Operation::DeclareArray(_, _) => {}
            Operation::And => {
                let l = self.operand_to_asm(&quad.operand1);
                let r = self.operand_to_asm(&quad.operand2);
                let d = self.operand_to_asm(&quad.result);
                self.instructions.extend([
                    format!("mov rax, {}", l),
                    format!("and rax, {}", r),
                    format!("mov {}, rax", d),
                ]);
            }
            Operation::Or => {
                let l = self.operand_to_asm(&quad.operand1);
                let r = self.operand_to_asm(&quad.operand2);
                let d = self.operand_to_asm(&quad.result);
                self.instructions.extend([
                    format!("mov rax, {}", l),
                    format!("or rax, {}", r),
                    format!("mov {}, rax", d),
                ]);
            }
            Operation::Not => {
                let o = self.operand_to_asm(&quad.operand1);
                let d = self.operand_to_asm(&quad.result);
                self.instructions.extend([
                    format!("mov rax, {}", o),
                    "not rax".into(),
                    format!("mov {}, rax", d),
                ]);
            }
            _ => {
                // Handle other operations if needed
                println!("Unhandled operation: {:?}", quad.operation);
            }
        }
    }

    fn gen_comparison(&mut self, quad: &Quadruple) {
        self.current_operation_is_comparison = true; // Set the context to comparison
        let left = self.operand_to_asm(&quad.operand1);
        let right = self.operand_to_asm(&quad.operand2);
        let result = self.operand_to_asm(&quad.result);

        let is_float = matches!(&quad.operand1, Operand::FloatLiteral(_))
            || matches!(&quad.operand2, Operand::FloatLiteral(_));

        let label_id = self.get_next_temp_label();
        let true_label = format!("L{}_true", label_id);
        let end_label = format!("L{}_end", label_id);

        if is_float {
            // 1) load and compare on x87
            self.instructions.push(format!("    fld   dword {}", left));
            self.instructions.push(format!("    fld  dword {}", right));
            self.instructions.push("    fcomip st0, st1".into());
            self.instructions.push("     fstp st0".into());
            self.instructions
                .push(format!("    mov   qword {}, 0", result));
            // 2) set result=0, then conditionally set to 1
            match &quad.operation {
                Operation::Equal => {
                    self.instructions.push(format!("    je    {}", true_label));
                }
                Operation::NotEqual => {
                    self.instructions.push(format!("    jne   {}", true_label));
                }
                Operation::LessThan => {
                    self.instructions.push(format!("    jb    {}", true_label)); // CF=1 if unordered or less
                }
                Operation::GreaterThan => {
                    self.instructions.push(format!("    ja    {}", true_label));
                }
                Operation::LessEqual => {
                    self.instructions.push(format!("    jae   {}", true_label));
                }
                Operation::GreaterEqual => {
                    self.instructions.push(format!("    jbe   {}", true_label));
                }
                _ => {}
            }
            self.instructions.push(format!("    jmp   {}", end_label));
            self.instructions.push(format!("{}:", true_label));
            self.instructions
                .push(format!("    mov   qword {}, 1", result));
            self.instructions.push(format!("{}:", end_label));
        } else {
            // fall back to integer compare
            self.instructions.push(format!("    mov   rax, {}", left));
            self.instructions.push(format!("    cmp   rax, {}", right));
            self.instructions
                .push(format!("    mov   qword {}, 0", result));
            match &quad.operation {
                Operation::Equal => {
                    self.instructions.push(format!("    je    {}", true_label));
                }
                Operation::NotEqual => {
                    self.instructions.push(format!("    jne   {}", true_label));
                }
                Operation::LessThan => {
                    self.instructions.push(format!("    jl    {}", true_label));
                }
                Operation::GreaterThan => {
                    self.instructions.push(format!("    jg    {}", true_label));
                }
                Operation::LessEqual => {
                    self.instructions.push(format!("    jle   {}", true_label));
                }
                Operation::GreaterEqual => {
                    self.instructions.push(format!("    jge   {}", true_label));
                }
                _ => {}
            }
            self.instructions.push(format!("    jmp   {}", end_label));
            self.instructions.push(format!("{}:", true_label));
            self.instructions
                .push(format!("    mov   qword {}, 1", result));
            self.instructions.push(format!("{}:", end_label));
        }

        self.current_operation_is_comparison = false; // Reset the context
    }

    // Add this helper method to get unique label IDs
    fn get_next_temp_label(&mut self) -> usize {
        let label_id = self.temp_label_counter;
        self.temp_label_counter += 1;
        label_id
    }

    fn operand_to_asm(&mut self, op: &Operand) -> String {
        match op {
            Operand::IntLiteral(val) => val.to_string(),
            Operand::FloatLiteral(val) => {
                // Conversion en représentation IEEE 754
                let val32 = *val as f32;
                let ieee754_bits = val32.to_bits();

                // Pour les constantes flottantes non affectées à une variable,
                // on les enregistre dans la section data
                if self.is_direct_usage_context() {
                    // Cette fonction déterminerait le contexte d'utilisation
                    // Vérifier si cette constante existe déjà
                    if !self.float_constants.contains_key(&ieee754_bits) {
                        // Créer un nouveau label pour cette constante
                        let label = format!("float_{}", self.float_counter);
                        self.float_counter += 1;

                        // Ajouter à la section data
                        // Enregistrer la constante pour la section data
                        self.data_section
                            .push(format!("{}: dd 0x{:08x}", label, ieee754_bits));

                        // Enregistrer pour une utilisation future
                        self.float_constants.insert(ieee754_bits, label.clone());

                        // Retourner le label
                        format!("[{}]", label)
                    } else {
                        // Retourner le label existant
                        format!("[{}]", self.float_constants.get(&ieee754_bits).unwrap())
                    }
                } else {
                    // Pour les assignations directes à des variables au début du programme,
                    // garder le format hexadécimal comme avant
                    format!("0x{:08x}", ieee754_bits)
                }
            }
            // Rest remains the same
            Operand::Variable(name) => format!("[{}]", name),
            Operand::TempVariable(name) => {
                format!("[rbp-{}]", 8 * name[1..].parse::<i32>().unwrap_or(1)) // 8 bytes per variable in 64-bit
            }
            Operand::ArrayVariable(name, _) => format!("{}", name),
            Operand::ArrayElement(name, idx) => {
                format!("[{}+{}*8]", name, self.operand_to_asm(idx)) // 8 bytes for 64-bit values
            }
            Operand::Empty => "_".to_string(),
            Operand::StringLiteral(s) => {
                // Handle string literals
                let id = self.get_next_string_id();
                let label = format!("str_{}", id);

                // Escaped string content for assembly
                let escaped_content = s.replace("\n", "10,").replace("\"", "\\\"");

                // Add to data section
                self.data_section
                    .push(format!("{}: db \"{}\", 0", label, escaped_content));

                // Return just the label, not quoted
                label
            }
        }
    }

    fn is_direct_usage_context(&self) -> bool {
        // Logique pour déterminer si l'opération en cours est une comparaison
        // ou une autre opération où la constante est utilisée directement
        // Par exemple, si on est dans gen_comparison(), cette fonction retournerait true
        // Pour les assignations simples, elle retournerait false

        // Implémentation simplifiée (à adapter selon votre logique)
        self.current_operation_is_comparison
    }

    fn get_type_directive(&self, typ: &Type) -> &'static str {
        match typ {
            Type::Int => "dq",   // 8-byte integer
            Type::Float => "dq", // 8-byte float
            _ => "db",           // Default
        }
    }

    fn get_reserve_directive(&self, typ: &Type) -> &'static str {
        match typ {
            Type::Int => "resq",   // reserve 8-byte integer
            Type::Float => "resq", // reserve 8-byte float
            _ => "resb",           // Default
        }
    }

    // Update the existing get_type_size function to handle all types
    fn get_type_size(&self, typ: &Type) -> usize {
        match typ {
            Type::Int => 8,   // 8 bytes for 64-bit integers
            Type::Float => 8, // 8 bytes for 64-bit floats
            _ => 8,           // Default size for 64-bit
        }
    }

    fn get_next_string_id(&mut self) -> usize {
        let id = self.string_counter;
        self.string_counter += 1;
        id
    }

    pub fn add_utility_functions(&mut self) {
        // Add necessary data buffers for our utility functions
        self.data_section.push("buffer: times 32 db 0".into()); // Buffer for integer/float conversions
        self.data_section.push("newline: db 10, 0".into()); // Newline character
        self.data_section
            .push("input_buffer: times 256 db 0".into()); // Buffer for reading input
        self.data_section.push("float_format: db \"%f\", 0".into()); // Format string for float printing

        // Add print_int implementation
        self.instructions.push(String::new());
        self.instructions
            .push("; Function to print integers".to_string());
        self.instructions.push("print_int:".to_string());
        self.instructions.push("    push rbp".to_string());
        self.instructions.push("    mov rbp, rsp".to_string());
        self.instructions.push("    push rbx".to_string());
        self.instructions.push("    push r12".to_string());
        self.instructions.push("    push r13".to_string());
        self.instructions.push("    mov rax, [rsp+40]".to_string()); // Get the parameter (64-bit calling convention)

        // Convert integer to string
        self.instructions.push("    mov rcx, 10".to_string());
        self.instructions.push("    mov rbx, buffer+31".to_string()); // Point to end of buffer
        self.instructions.push("    mov byte [rbx], 0".to_string()); // Null terminate
        self.instructions.push("    dec rbx".to_string());

        // Handle negative numbers
        self.instructions.push("    mov r12, 0".to_string()); // Sign flag
        self.instructions.push("    cmp rax, 0".to_string());
        self.instructions.push("    jge .positive".to_string());
        self.instructions.push("    mov r12, 1".to_string()); // Set sign flag
        self.instructions.push("    neg rax".to_string()); // Make positive

        self.instructions.push(".positive:".to_string());
        self.instructions.push(".loop:".to_string());
        self.instructions.push("    xor rdx, rdx".to_string()); // Clear rdx for division
        self.instructions.push("    div rcx".to_string()); // rax / 10, remainder in rdx
        self.instructions.push("    add dl, '0'".to_string()); // Convert to ASCII
        self.instructions.push("    mov [rbx], dl".to_string()); // Store digit
        self.instructions.push("    dec rbx".to_string()); // Move buffer pointer
        self.instructions.push("    test rax, rax".to_string()); // Check if done
        self.instructions.push("    jnz .loop".to_string()); // Continue if not zero

        self.instructions.push("    cmp r12, 1".to_string()); // Check sign flag
        self.instructions.push("    jne .print".to_string());
        self.instructions
            .push("    mov byte [rbx], '-'".to_string()); // Add minus sign
        self.instructions.push("    dec rbx".to_string());

        self.instructions.push(".print:".to_string());
        self.instructions.push("    inc rbx".to_string()); // Point to first character

        // Print the string using write syscall
        self.instructions.push("    mov rax, 1".to_string()); // syscall: write
        self.instructions.push("    mov rdi, 1".to_string()); // file: stdout
        self.instructions.push("    mov rsi, rbx".to_string()); // buffer
        self.instructions.push("    mov rdx, buffer+31".to_string());
        self.instructions.push("    sub rdx, rbx".to_string()); // length
        self.instructions.push("    syscall".to_string());

        // Print newline
        self.instructions.push("    mov rax, 1".to_string()); // syscall: write
        self.instructions.push("    mov rdi, 1".to_string()); // file: stdout
        self.instructions.push("    mov rsi, newline".to_string()); // buffer
        self.instructions.push("    mov rdx, 1".to_string()); // length
        self.instructions.push("    syscall".to_string());

        self.instructions.push("    pop r13".to_string());
        self.instructions.push("    pop r12".to_string());
        self.instructions.push("    pop rbx".to_string());
        self.instructions.push("    pop rbp".to_string());
        self.instructions.push("    ret".to_string());

        // Add read_int implementation
        self.instructions.push(String::new());
        self.instructions
            .push("; Function to read integers".to_string());
        self.instructions.push("read_int:".to_string());
        self.instructions.push("    push rbp".to_string());
        self.instructions.push("    mov rbp, rsp".to_string());
        self.instructions.push("    push rbx".to_string());
        self.instructions.push("    push r12".to_string());

        // Read input using read syscall
        self.instructions.push("    mov rax, 0".to_string()); // syscall: read
        self.instructions.push("    mov rdi, 0".to_string()); // file: stdin
        self.instructions
            .push("    mov rsi, input_buffer".to_string()); // buffer
        self.instructions.push("    mov rdx, 255".to_string()); // max length
        self.instructions.push("    syscall".to_string());

        // Parse integer
        self.instructions.push("    mov rcx, 0".to_string()); // value accumulator
        self.instructions
            .push("    mov rbx, input_buffer".to_string());
        self.instructions.push("    mov r12, 0".to_string()); // sign flag

        // Check for leading minus sign
        self.instructions
            .push("    cmp byte [rbx], '-'".to_string());
        self.instructions.push("    jne .parse_loop".to_string());
        self.instructions.push("    mov r12, 1".to_string()); // Set sign flag
        self.instructions.push("    inc rbx".to_string()); // Skip the minus

        self.instructions.push(".parse_loop:".to_string());
        self.instructions
            .push("    movzx rax, byte [rbx]".to_string()); // Get character
        self.instructions.push("    cmp al, 10".to_string()); // Check for newline
        self.instructions.push("    je .parse_done".to_string());
        self.instructions.push("    cmp al, 0".to_string()); // Check for null
        self.instructions.push("    je .parse_done".to_string());

        self.instructions.push("    sub al, '0'".to_string()); // Convert to digit
        self.instructions.push("    imul rcx, 10".to_string()); // Multiply accumulator by 10
        self.instructions.push("    add rcx, rax".to_string()); // Add digit
        self.instructions.push("    inc rbx".to_string()); // Next character
        self.instructions.push("    jmp .parse_loop".to_string());

        self.instructions.push(".parse_done:".to_string());
        self.instructions.push("    cmp r12, 1".to_string()); // Check sign flag
        self.instructions.push("    jne .return".to_string());
        self.instructions.push("    neg rcx".to_string()); // Negate if needed

        self.instructions.push(".return:".to_string());
        self.instructions.push("    mov rax, rcx".to_string()); // Return value
        self.instructions.push("    pop r12".to_string());
        self.instructions.push("    pop rbx".to_string());
        self.instructions.push("    pop rbp".to_string());
        self.instructions.push("    ret".to_string());

        // Add print_float implementation
        self.instructions.push(String::new());
        self.instructions
            .push("; Function to print floats".to_string());
        self.instructions.push("print_float:".to_string());
        self.instructions.push("    push rbp".to_string());
        self.instructions.push("    mov rbp, rsp".to_string());

        // Assume the float is in ST0
        self.instructions.push("    fstp qword [rsp-8]".to_string()); // Store float from ST0 to stack
        self.instructions.push("    sub rsp, 8".to_string()); // Adjust stack

        // Use a simple algorithm for float to string conversion
        // For simplicity here, we convert integer part, then fraction
        self.instructions.push("    fld qword [rsp]".to_string()); // Load float back to FPU
        self.instructions.push("    lea rbx, [buffer]".to_string()); // Buffer for output

        // Extract integer part
        self.instructions.push("    fld st0".to_string()); // Duplicate float
        self.instructions.push("    frndint".to_string()); // Round to integer (in FPU)
        self.instructions.push("    fistp qword [rbx]".to_string()); // Store integer part
        self.instructions.push("    mov rax, [rbx]".to_string()); // Load integer part

        // Print integer part using our existing print_int
        self.instructions.push("    push rax".to_string());
        self.instructions
            .push("    call print_float_helper".to_string());
        self.instructions.push("    add rsp, 8".to_string());

        self.instructions.push("    add rsp, 8".to_string()); // Restore stack
        self.instructions.push("    pop rbp".to_string());
        self.instructions.push("    ret".to_string());

        // Helper function for print_float
        self.instructions.push("print_float_helper:".to_string());
        self.instructions.push("    push rbp".to_string());
        self.instructions.push("    mov rbp, rsp".to_string());

        // Simplified algorithm to print a float
        self.instructions.push("    mov rax, [rsp+16]".to_string()); // Get the float value

        // Just convert to int and print for now (simplified)
        self.instructions.push("    push rax".to_string());
        self.instructions.push("    call print_int".to_string());
        self.instructions.push("    add rsp, 8".to_string());

        self.instructions.push("    pop rbp".to_string());
        self.instructions.push("    ret".to_string());

        // Add read_float implementation (simplistic version)
        self.instructions.push(String::new());
        self.instructions
            .push("; Function to read floats".to_string());
        self.instructions.push("read_float:".to_string());
        self.instructions.push("    push rbp".to_string());
        self.instructions.push("    mov rbp, rsp".to_string());

        // For simplicity, we just read an integer and convert to float
        self.instructions.push("    call read_int".to_string());
        self.instructions.push("    cvtsi2sd xmm0, rax".to_string()); // Convert int to float

        self.instructions.push("    pop rbp".to_string());
        self.instructions.push("    ret".to_string());

        // Add print_string implementation
        self.instructions.push(String::new());
        self.instructions
            .push("; Function to print strings".to_string());
        self.instructions.push("print_string:".to_string());
        self.instructions.push("    push rbp".to_string());
        self.instructions.push("    mov rbp, rsp".to_string());
        self.instructions.push("    push rbx".to_string());

        // The parameter is the address of the string
        self.instructions.push("    mov rbx, [rsp+24]".to_string());

        // Calculate string length
        self.instructions.push("    mov rdx, 0".to_string()); // Length counter
        self.instructions.push(".strlen_loop:".to_string());
        self.instructions
            .push("    cmp byte [rbx+rdx], 0".to_string());
        self.instructions.push("    je .print_it".to_string());
        self.instructions.push("    inc rdx".to_string());
        self.instructions.push("    jmp .strlen_loop".to_string());

        // Print the string using write syscall
        self.instructions.push(".print_it:".to_string());
        self.instructions.push("    mov rax, 1".to_string()); // syscall: write
        self.instructions.push("    mov rdi, 1".to_string()); // file: stdout
        self.instructions.push("    mov rsi, rbx".to_string()); // buffer
        self.instructions.push("    syscall".to_string());

        self.instructions.push("    pop rbx".to_string());
        self.instructions.push("    pop rbp".to_string());
        self.instructions.push("    ret".to_string());

        // Add read_string implementation
        self.instructions.push(String::new());
        self.instructions
            .push("; Function to read strings".to_string());
        self.instructions.push("read_string:".to_string());
        self.instructions.push("    push rbp".to_string());
        self.instructions.push("    mov rbp, rsp".to_string());

        // The parameter is the buffer address and max size
        self.instructions.push("    mov rsi, [rsp+16]".to_string()); // Buffer address
        self.instructions.push("    mov rdx, [rsp+24]".to_string()); // Max size

        // Read input using read syscall
        self.instructions.push("    mov rax, 0".to_string()); // syscall: read
        self.instructions.push("    mov rdi, 0".to_string()); // file: stdin
        self.instructions.push("    syscall".to_string());

        // Replace newline with null terminator
        self.instructions.push("    mov rbx, rsi".to_string());
        self.instructions.push("    add rbx, rax".to_string()); // Point to the end
        self.instructions.push("    dec rbx".to_string());
        self.instructions.push("    cmp byte [rbx], 10".to_string()); // Check for newline
        self.instructions.push("    jne .done".to_string());
        self.instructions.push("    mov byte [rbx], 0".to_string()); // Replace with null

        self.instructions.push(".done:".to_string());
        self.instructions.push("    pop rbp".to_string());
        self.instructions.push("    ret".to_string());
    }
}
