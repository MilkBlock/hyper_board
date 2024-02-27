#[macro_export] 
macro_rules! match_command{
    ($(command $command_name:ident with args ($($arg_n:ident:$arg_n_type:ident),*) )+ in $command_args:ident with sock $sock:ident) => {
        match($command_args){
            $(Some(CommandArgs{command ,args}) if &command == stringify!($command_name) =>{
                let mut count = 0;
                match ($({count+=1;&args[count-1].parse::<$arg_n_type>()}),*){
                    ($(Ok($arg_n)),*) =>{
                        $command_name(&mut $sock);
                        println!("successfullly run \x1B[32m {} \x1B[0m with args \x1B[32m{:?}\x1B[0m",command,args)
                    },
                    _ => {println!("conversion failed \x1B[31m{}\x1B[0m with args \x1B[31m{:?}\x1B[0m ",stringify!($command_name), args)}
                }
            }),*
            // ? 找不到命令
            _=>{
                println!("command {:?} not recognized! " , $command_args)
            }
        }
    } ;
}