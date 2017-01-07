
use std::io::Write;
use vm::VirtualMachine;

pub type UnblockEvent = Box<Fn(&mut VirtualMachine)>;

pub struct CommandSystem {
    commands: Vec<Box<Command>>, 
}
impl CommandSystem {
    pub fn new() -> CommandSystem {
        let mut system = CommandSystem {
            commands: Vec::new()
        };

        system.add_command(HelpCommand);
        system.add_command(ClearCommand);
        system.add_command(SourceCommand);
        system.add_command(ListCommand);
        system.add_command(RegistersCommand);
        system.add_command(StepCommand);
        system.add_command(ContinueCommand);
        system.add_command(BreakCommand);
        system.add_command(FlagsCommand);
        system.add_command(MemdmpCommand);
        system.add_command(MemsetCommand);
        system.add_command(MonitorCommand);

        system
    }

    pub fn add_command<C>(&mut self, command: C)
        where C: Command + 'static
    {
        self.commands.push(Box::new(command));
    }

    pub fn execute<S>(&self, command: S, mut vm: &mut VirtualMachine) -> (CommandResult, Option<UnblockEvent>)
        where S: Into<String>
    {
        let command = command.into();
        let parts = command.split_whitespace().map(String::from).collect::<Vec<_>>();

        for command in self.commands.iter() {
            if command.matches_name(parts[0].clone()) {
                let result = command.execute(parts[1..].to_owned(), &self, &mut vm);
                if let CommandResult::SucessBlock = result {
                    return (result, Some(command.on_unblock_event()));
                } else {
                    return (result, None);
                }
            }
        }

        (CommandResult::NotFound, None)
    }
}

pub enum CommandResult {
    NotFound,
    InvalidArgs,
    Sucess,
    SucessBlock,
}

pub trait Command {
    fn execute(&self, args: Vec<String>, system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult;

    fn get_names(&self) -> Vec<&str>;

    fn get_arg_info(&self) -> Option<&str> {
        None
    }

    fn get_help(&self) -> &str {
        "No help text provided!"
    }

    fn matches_name(&self, name: String) -> bool {
        for actual_name in self.get_names() {
            if actual_name == name {
                return true;
            }
        }
        return false
    }

    fn on_unblock_event(&self) -> UnblockEvent {
        Box::new(|_| {})
    }
}


struct HelpCommand;
impl Command for HelpCommand {
    fn execute(&self, _args: Vec<String>, system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        writeln!(vm.console, "Commands:").unwrap();

        // Creates strings containing all names, e.g. "help, h, ?"
        for command in system.commands.iter() {
            let name = command.get_names().join(", ");
            write!(vm.console, "   {}", name).unwrap();
            if let Some(arg_info) = command.get_arg_info() {
                write!(vm.console, ": {}", arg_info).unwrap();
            }
            writeln!(vm.console, "").unwrap();

            let help = command.get_help();
            let help_lines = help.trim().lines().map(|line| line.trim()).collect::<Vec<_>>();
            if !help_lines.is_empty() {
                for help_line in &help_lines {
                    writeln!(vm.console, "      {}", help_line).unwrap();;
                }
            }
        }

        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["help", "h", "?"]
    }

    fn get_help(&self) -> &str {
        "Prints this help message"
    } 
}

struct ClearCommand;
impl Command for ClearCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        vm.console.clear();
        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["clear", "cls"]
    }

    fn get_help(&self) -> &str {
        "Clears the console"
    }
}

struct SourceCommand;
impl Command for SourceCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        vm.dump_disassembly();
        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["source"]
    }

    fn get_help(&self) -> &str {
        "Lists the code currently running in the virtual
         machine. A '>' symbol indicates the current
         program counter."
    }
}

struct ListCommand;
impl Command for ListCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        vm.dump_local_disassembly();
        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["list"]
    }

    fn get_help(&self) -> &str {
        "Lists the code surrounding the current
         program counter."
    }
}

struct RegistersCommand;
impl Command for RegistersCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        vm.dump_registers();
        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["registers", "reg"]
    }

    fn get_help(&self) -> &str {
        "Lists the CPU registers and their current
         values."
    }
}

struct MonitorCommand;
impl Command for MonitorCommand {
    fn execute(&self, args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        if args.len() != 2 {
            writeln!(vm.console, "Expected 2 arguments, found {}", args.len()).unwrap();
            return CommandResult::InvalidArgs;
        }

        let start = usize::from_str_radix(&args[0].replace("0x", "")[..], 16);
        if start.is_err() {
            writeln!(vm.console, "Expected hexadecimal memory address, found {}", args[0]).unwrap();
            return CommandResult::InvalidArgs;
        }
        let start = start.unwrap();
        
        let end = usize::from_str_radix(&args[1].replace("0x", "")[..], 16);
        if end.is_err() {
            writeln!(vm.console, "Expected hexadecimal memory address, found {}", args[1]).unwrap();
            return CommandResult::InvalidArgs;
        }
        let end = end.unwrap();

        vm.enable_memory_monitor(start..end);

        CommandResult::SucessBlock
    }

    fn on_unblock_event(&self) -> UnblockEvent {
        Box::new(|vm| {
            vm.disable_memory_monitor();
            writeln!(vm.console, "Disabled memory monitor").unwrap();
        })
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["monitor", "mon"]
    }
    
    fn get_arg_info(&self) -> Option<&str> {
        Some("start end")
    }

    fn get_help(&self) -> &str {
        "Dumps the memory between <start> and <end>
         (inclusive) every seconds. Press ENTER to
         stop monitoring."
    }
}

struct MemsetCommand;
impl Command for MemsetCommand {
    fn execute(&self, args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        if args.len() < 2 {
            writeln!(vm.console,
                "Expected 2 arguments. E.g.: memset 0x00 0x01 stores 0x01 at address 0x00"
            ).unwrap();
            return CommandResult::InvalidArgs;
        }

        let start = usize::from_str_radix(&args[0].replace("0x", "")[..], 16);
        if start.is_err() {
            writeln!(vm.console, "Expected hexadecimal memory address, found {}", args[0]).unwrap();
            return CommandResult::InvalidArgs;
        }
        let start = start.unwrap();

        let bytes = {
            let mut bytes = Vec::new();
            for index in 1..args.len() {
                let byte = u8::from_str_radix(&args[index].replace("0x", "")[..], 16);
                if byte.is_err() {
                    writeln!(vm.console, "Expected hexadecimal byte, found {}", args[index]).unwrap();
                    return CommandResult::InvalidArgs;
                }
                bytes.push(byte.unwrap());
            }
            bytes
        };

        for (index, byte) in bytes.iter().enumerate() {
            vm.cpu.memory[start + index] = *byte;
        }

        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["memset", "set"]
    }
    
    fn get_arg_info(&self) -> Option<&str> {
        Some("addres value [values, ...]")
    }

    fn get_help(&self) -> &str {
        "Writes the given value to the given address.
         Multiple values will be written to consequent
         addresses."
    }
}

struct MemdmpCommand;
impl Command for MemdmpCommand {
    fn execute(&self, args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        if args.is_empty() || args.len() > 2 {
            writeln!(vm.console, "Expected either 1 or 2 arguments, found {}", args.len()).unwrap();
            return CommandResult::InvalidArgs;
        }

        // Dump a page
        if args.len() == 1 {
            let page = usize::from_str_radix(&args[0].replace("0x", "")[..], 16);
            if page.is_err() {
                writeln!(vm.console, "Expected page index, found {}", args[0]).unwrap();
                return CommandResult::InvalidArgs;
            }
            let page = page.unwrap();

            vm.dump_memory_page(page);

        // Dump a range
        } else if args.len() == 2 { 
            let start = usize::from_str_radix(&args[0].replace("0x", "")[..], 16);
            if start.is_err() {
                writeln!(vm.console, "Expected hexadecimal memory address, found {}", args[0]).unwrap();
                return CommandResult::InvalidArgs;
            }
            let start = start.unwrap();

            let end = usize::from_str_radix(&args[1].replace("0x", "")[..], 16);
            if end.is_err() {
                writeln!(vm.console, "Expected hexadecimal memory address, found {}", args[1]).unwrap();
                return CommandResult::InvalidArgs;
            }
            let end = end.unwrap();

            vm.dump_memory_range(start, end);
        }

        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["memdmp", "dmp"]
    }
    
    fn get_arg_info(&self) -> Option<&str> {
        Some("page OR start end")
    }

    fn get_help(&self) -> &str {
        "Dumps a single memory page, or a specified memory
         range from <start> to <end>."
    }
}

struct FlagsCommand;
impl Command for FlagsCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        vm.dump_flags();
        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["flags"]
    }

    fn get_help(&self) -> &str {
        "Not yet implemented"
    }
}

struct BreakCommand;
impl Command for BreakCommand {
    fn execute(&self, args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        if args.len() > 1 {
            writeln!(vm.console, "Expected 0 or 1 arguments, found {}", args.len()).unwrap();
            return CommandResult::InvalidArgs;
        }

        // Break at the given address
        if args.len() == 1 {
            let address = usize::from_str_radix(&args[0].replace("0x", "")[..], 16);
            if address.is_err() {
                writeln!(vm.console, "Expected hexadecimal memory address, found {}", args[0]).unwrap();
            }
            let address = address.unwrap();

            if address <= u16::max_value() as usize {
                if vm.toggle_breakpoint(address) {
                    writeln!(vm.console, "Added breakpoint at {:04X}", address).unwrap();
                } else {
                    writeln!(vm.console, "Removed breakpoint at {:04X}", address).unwrap();
                }
            } else {
                writeln!(vm.console, "Address outside addressable range.").unwrap();
            }

        // Break at current program counter
        } else {
            vm.break_execution();
            writeln!(vm.console, "Breaking execution at {:04X}", vm.cpu.registers.PC).unwrap();
        }

        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["break", "b"]
    }

    fn get_arg_info(&self) -> Option<&str> {
        Some("[address]")
    }

    fn get_help(&self) -> &str {
        "Toggles a breakpoint at the specified <address>.
         If the program counter hits this address, execution
         stops. If no address is specified, execution will
         be stopped at the current point, without inserting
         a breakpoint."
    }
}

struct ContinueCommand;
impl Command for ContinueCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        vm.continue_execution();
        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["continue", "c"]
    }

    fn get_help(&self) -> &str {
        "Resumes program execution after the program
         has stopped at a breakpoint."
    }
}

struct StepCommand;
impl Command for StepCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) -> CommandResult {
        vm.step_execution();
        CommandResult::Sucess
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["step", "s"]
    }

    fn get_help(&self) -> &str {
        "Executes a single instruction, then stops
         execution"
    }
}

