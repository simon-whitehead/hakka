
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
        system.add_command(GreetCommand);
        // TODO: Implement 'repeat' as default command

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
    fn matches_name(&self, name: String) -> bool;
    fn get_help(&self) -> String {
        String::from("No help text provided")
    }
}

struct HelpCommand;
impl Command for HelpCommand {
    fn execute(&self, _args: Vec<String>, system: &CommandSystem, vm: &mut VirtualMachine) {
        writeln!(vm.console, "Commands:").unwrap();
        for command in &system.commands {
            writeln!(vm.console, "    {}", command.get_help()).unwrap();
        }
    }

    fn matches_name(&self, name: String) -> bool {
        name == "help" || name == "h" || name == "?"
    }

    fn get_help(&self) -> String {
        String::from("help, h, ?      Prints this help message")
    } 
}

struct GreetCommand;
impl Command for GreetCommand {
    fn execute(&self, args: Vec<String>, _system: &CommandSystem, vm: &mut VirtualMachine) {
        if args.is_empty() {
            writeln!(vm.console, "Hello world!").unwrap();
        } else {
            for name in args {
                writeln!(vm.console, "Hello {}!", name).unwrap();
            }
        }
    }

    fn matches_name(&self, name: String) -> bool {
        name == "greet" || name == "g"
    }

    fn get_help(&self) -> String {
        String::from("greet, g        Greets the given person")
    } 
}

