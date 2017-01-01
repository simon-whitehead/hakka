
// TODO:
// Implement repeat command
// Add some way for help messages to display parameters
// Implement missing commands (Need flags, which need to be shown in help)
//  - monitor | mon (Note: Monitor command needs some way to cancel monitoring)
//  - memset | set
//  - memdmp | dmp
//  - flags
//  - break | b
//  - continue | c
//  - step | s

use std::io::Write;
use vm::VirtualMachine;

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

        system
    }

    pub fn add_command<C>(&mut self, command: C)
        where C: Command + 'static
    {
        self.commands.push(Box::new(command));
    }

    pub fn execute<S>(&self, command: S, mut vm: &mut VirtualMachine) -> bool
        where S: Into<String>
    {
        let command = command.into();
        let parts = command.split_whitespace().map(String::from).collect::<Vec<_>>();

        for command in &self.commands {
            if command.matches_name(parts[0].clone()) {
                command.execute(parts[1..].to_owned(), &self, &mut vm);
                return true;
            }
        }

        false
    }
}

pub trait Command {
    fn execute(&self, args: Vec<String>, system: &CommandSystem, vm: &mut VirtualMachine);
    fn get_names(&self) -> Vec<&str>;
    fn get_help(&self) -> String {
        String::from("No help text provided")
    }

    fn matches_name(&self, name: String) -> bool {
        for actual_name in self.get_names() {
            if actual_name == name {
                return true;
            }
        }
        return false
    }
}

struct HelpCommand;
impl Command for HelpCommand {
    fn execute(&self, _args: Vec<String>, system: &CommandSystem, vm: &mut VirtualMachine) {
        writeln!(vm.console, "Commands:").unwrap();

        // Creates strings containing all names, e.g. "help, h, ?"
        for command in system.commands.iter() {
            let name = command.get_names().join(", ");
            writeln!(vm.console, "   {}", name).unwrap();

            let help = command.get_help();
            let help_lines = help.trim().lines().map(|line| line.trim()).collect::<Vec<_>>();
            if !help_lines.is_empty() {
                for help_line in &help_lines {
                    writeln!(vm.console, "      {}", help_line).unwrap();;
                }
            }
        }
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["help", "h", "?"]
    }

    fn get_help(&self) -> String {
        String::from("Prints this help message")
    } 
}

struct ClearCommand;
impl Command for ClearCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) {
        vm.console.clear();
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["clear", "cls"]
    }

    fn get_help(&self) -> String {
        String::from("
            Clears the console
        ")
    }
}

struct SourceCommand;
impl Command for SourceCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) {
        vm.dump_disassembly();
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["source"]
    }

    fn get_help(&self) -> String {
        String::from("
            Lists the code currently running in the virtual
            machine. A '>' symbol indicates the current
            program counter.
        ")
    }
}

struct ListCommand;
impl Command for ListCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) {
        vm.dump_local_disassembly();
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["list"]
    }

    fn get_help(&self) -> String {
        String::from("
            Lists the code surrounding the current
            program counter.
        ")
    }
}

struct RegistersCommand;
impl Command for RegistersCommand {
    fn execute(&self, _args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) {
        vm.dump_registers();
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["registers", "reg"]
    }

    fn get_help(&self) -> String {
        String::from("
            Lists the CPU registers and their current
            values.
        ")
    }
}

