#[derive(Debug)]
pub struct CommandArgs {
    pub command: String,
    pub args: Vec<String>,
}

impl CommandArgs{
    pub fn parse_command(input: String) -> Option<CommandArgs> {
        let parts: Vec<String> = input.split(':').map(|s|s.to_string()).collect();
        if parts.len() != 2 {
            return None;
        }
        let command = &parts[0];
        if command.len() ==0{
            println!("message has no command");
            return None
        }

        let args_str = &parts[1];
        if !args_str.starts_with('(') || !args_str.ends_with(')') {
            return None;
        }
        let args_clean = &args_str[1..args_str.len() - 1];
        let args: Vec<String> = args_clean.split(',').map(|s|s.to_string()).collect();

        Some(CommandArgs {
            command: command[0..].to_string(),
            args,
        })
    }

}