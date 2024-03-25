#[derive(Debug)]
pub struct CommandArgs {
    pub command: String,
    pub args: Vec<String>,
    pub dialogue_id: Option<u64>,
}

impl CommandArgs{
    pub fn parse_command(input: String) -> Option<CommandArgs> {
        let input = input.trim().to_string();
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

        let dialogue_id:Option<u64>= if input.ends_with("]"){
            let rev_chs = input.chars().rev();
            // 这是个左闭右开区间
            let event_id_start_index = input.len()-1;
            let event_id_end_index = rev_chs.enumerate().find(|(index,ch)| *ch=='[').unwrap().0 +1;
            Some(input[event_id_start_index..event_id_end_index].parse::<u64>().unwrap())
        }else{
            None
        };
        Some(CommandArgs {
            command: command[0..].to_string(),
            args,
            dialogue_id,
        })
    }
}

// fn main() {
//     let input = "a:(1,2)";
//     match parse_command(input) {
//         Some(cmd_args) => {
//             println!("command: {}", cmd_args.command);
//             println!("args: {:?}", cmd_args.args);
//         }
//         None => println!("Invalid format"),
//     }
// }
