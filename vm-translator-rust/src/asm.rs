use crate::parser::VmCommand;

static LOGICAL_OR_MATH_TEMPLATE: &str = r#"// MATH or LOGICAL {operation}
@SP
AM=M-1
D=M
A=A-1
M={operation}
"#;

static NEG: &str = r#"// neg
@SP
A=M-1
M=-M
"#;

static NOT: &str = r#"// not
@SP
A=M-1
M=!M
"#;

static GT_LT_TEMPLATE: &str = r#"// OPERATION IS LOGICAL {jump_condition}
@SP
AM=M-1
D=M
A=A-1
D=M-D
@{jump_condition}_TRUE_{postfix}
D;{jump_condition}
@SP
A=M-1
M=0
@{jump_condition}_END_{postfix}
0;JMP
({jump_condition}_TRUE_{postfix})
@SP
A=M-1
M=-1
({jump_condition}_END_{postfix})
"#;

static PUSH_REF_TEMPLATE: &str = r#"// PUSH {segment} {index}
@{index}
D=A
@{segment}
A=D+M
D=M
@SP
A=M
M=D
@SP
M=M+1
"#;

static PUSH_DIRECT_TEMPLATE: &str = r#"// PUSH {address}
@{address}
{target}
@SP
A=M
M=D
@SP
M=M+1
"#;

static POP_REF_TEMPLATE: &str = r#"// POP {segment} {index}
@{index}
D=A
@{segment}
D=D+M
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
"#;

static POP_DIRECT_TEMPLATE: &str = r#"// POP {address}
@SP
AM=M-1
D=M
@{address}
M=D
"#;

pub struct AsmTranslator {
    condition_counter: u16,
}

impl AsmTranslator {
    pub fn new() -> Self {
        Self {
            condition_counter: 0,
        }
    }

    pub fn to_asm(&mut self, command: VmCommand, postfix: &str) -> Result<String, String> {
        match command {
            VmCommand::Add => {
                Ok(self.custom_str_replace(LOGICAL_OR_MATH_TEMPLATE, &[("{operation}", "D+M")]))
            }
            VmCommand::Sub => {
                Ok(self.custom_str_replace(LOGICAL_OR_MATH_TEMPLATE, &[("{operation}", "M-D")]))
            }
            VmCommand::Neg => Ok(NEG.to_string()),
            VmCommand::Gt => {
                let unique_postfix = format!("{}_{}", postfix, self.condition_counter);
                self.condition_counter += 1;
                Ok(self.custom_str_replace(
                    GT_LT_TEMPLATE,
                    &[("{postfix}", &unique_postfix), ("{jump_condition}", "JGT")],
                ))
            }
            VmCommand::Lt => {
                let unique_postfix = format!("{}_{}", postfix, self.condition_counter);
                self.condition_counter += 1;
                Ok(self.custom_str_replace(
                    GT_LT_TEMPLATE,
                    &[("{postfix}", &unique_postfix), ("{jump_condition}", "JLT")],
                ))
            }
            VmCommand::Eq => {
                let unique_postfix = format!("{}_{}", postfix, self.condition_counter);
                self.condition_counter += 1;
                Ok(self.custom_str_replace(
                    GT_LT_TEMPLATE,
                    &[("{postfix}", &unique_postfix), ("{jump_condition}", "JEQ")],
                ))
            }

            VmCommand::And => {
                Ok(self.custom_str_replace(LOGICAL_OR_MATH_TEMPLATE, &[("{operation}", "D&M")]))
            }

            VmCommand::Or => {
                Ok(self.custom_str_replace(LOGICAL_OR_MATH_TEMPLATE, &[("{operation}", "D|M")]))
            }

            VmCommand::Not => Ok(NOT.to_string()),

            VmCommand::Push { segment, index } => self.assemble_push(segment, index, postfix),

            VmCommand::Pop { segment, index } => self.assemble_pop(segment, index, postfix),

            _ => Err("Not implemented".to_string()),
        }
    }

    fn assemble_push(&self, segment: &str, index: u16, postfix: &str) -> Result<String, String> {
        match segment {
            "constant" => Ok(self.custom_str_replace(
                PUSH_DIRECT_TEMPLATE,
                &[("{address}", &index.to_string()), ("{target}", "D=A")],
            )),

            "static" => Ok(self.custom_str_replace(
                PUSH_DIRECT_TEMPLATE,
                &[
                    ("{address}", &format!("{}.{}", postfix, index)),
                    ("{target}", "D=M"),
                ],
            )),

            "pointer" if index == 0 => Ok(self.custom_str_replace(
                PUSH_DIRECT_TEMPLATE,
                &[("{address}", "THIS"), ("{target}", "D=M")],
            )),

            "pointer" if index == 1 => Ok(self.custom_str_replace(
                PUSH_DIRECT_TEMPLATE,
                &[("{address}", "THAT"), ("{target}", "D=M")],
            )),

            "temp" => Ok(self.custom_str_replace(
                PUSH_DIRECT_TEMPLATE,
                &[
                    ("{address}", &format!("{}", 5 + index)),
                    ("{target}", "D=M"),
                ],
            )),

            _ => {
                let reference = self.resolve_reference(segment)?;
                Ok(self.custom_str_replace(
                    PUSH_REF_TEMPLATE,
                    &[("{segment}", &reference), ("{index}", &index.to_string())],
                ))
            }
        }
    }

    fn assemble_pop(&self, segment: &str, index: u16, postfix: &str) -> Result<String, String> {
        match segment {
            "static" => Ok(self.custom_str_replace(
                POP_DIRECT_TEMPLATE,
                &[("{address}", &format!("{}.{}", postfix, index))],
            )),

            "pointer" if index == 0 => {
                Ok(self.custom_str_replace(POP_DIRECT_TEMPLATE, &[("{address}", "THIS")]))
            }

            "pointer" if index == 1 => {
                Ok(self.custom_str_replace(POP_DIRECT_TEMPLATE, &[("{address}", "THAT")]))
            }

            "temp" => Ok(self.custom_str_replace(
                POP_DIRECT_TEMPLATE,
                &[("{address}", &format!("R{}", 5 + index))],
            )),

            _ => {
                let reference = self.resolve_reference(segment)?;
                Ok(self.custom_str_replace(
                    POP_REF_TEMPLATE,
                    &[("{segment}", &reference), ("{index}", &index.to_string())],
                ))
            }
        }
    }

    fn resolve_reference(&self, segment: &str) -> Result<String, String> {
        match segment {
            "local" => Ok("LCL".to_string()),
            "argument" => Ok("ARG".to_string()),
            "this" => Ok("THIS".to_string()),
            "that" => Ok("THAT".to_string()),
            _ => Err("Invalid segment".to_string()),
        }
    }

    fn custom_str_replace(&self, template: &str, replacements: &[(&str, &str)]) -> String {
        let mut result = String::with_capacity(template.len());
        let mut idx = 0;
        while idx < template.len() {
            let mut found = false;
            for (find, replace) in replacements {
                if template[idx..].starts_with(find) {
                    result.push_str(replace);
                    idx += find.len();
                    found = true;
                    break;
                }
            }
            if !found {
                result.push(template.chars().nth(idx).unwrap());
                idx += 1;
            }
        }
        result
    }
}

#[cfg(test)]
mod unit {

    #[test]
    fn test_vm_command_to_asm() {
        use super::*;
        let command = VmCommand::Push {
            segment: "constant",
            index: 32000,
        };
        let mut translator = AsmTranslator::new();
        println!("{:?}", translator.to_asm(command, "test"));
    }
}
