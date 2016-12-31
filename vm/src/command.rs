
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
        let mut command_names = system.commands.iter().map(|c| c.get_names().join(", ")).collect::<Vec<_>>();
        let longest_name_length = command_names.iter().max_by_key(|name| name.len()).map(|name| name.len()).unwrap_or(0);
        let name_padding = longest_name_length + 2;

        for (index, command) in system.commands.iter().enumerate() {
            let ref name = command_names[index];
            writeln!(vm.console, "    {0:1$}{2}", name, name_padding, command.get_help()).unwrap();
        }
    }

    fn get_names(&self) -> Vec<&str> {
        vec!["help", "h", "?"]
    }

    fn get_help(&self) -> String {
        String::from("Prints this help message")
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

    fn get_names(&self) -> Vec<&str> {
        vec!["greet", "g"]
    }

    fn get_help(&self) -> String {
        String::from("Greets the given person")
    }
}

