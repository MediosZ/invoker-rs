use rustyline::error::ReadlineError;
use rustyline::Editor;
use invoker::Invoker;


fn process(invoker: &mut Invoker, line: String) -> bool{
    // println!("Line: {}", line);
    let mut segs = line.split_whitespace();
    if let Some(command) = segs.next() {
        // println!("command: {}", command);
        if command == "exit" {
            return true;
        }
        if command == "load" {
            if let Some(file) = segs.next(){
                invoker.load(file);
            }
            else {
                println!("load nothing...");
            }
        }
        else if command == "inspect" {
            invoker.inspect();
        }
        else if command == "call" {
            let func = segs.next().unwrap();
            let args = segs.collect::<Vec<&str>>();
            invoker.call(func, args);
        }
        else if command == "unload" {
            invoker.unload();
        }
        else{
            println!("unknown command");
        }
        return false;
    }
    return false;
}

fn main() {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    let mut invoker = Invoker::default();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let should_exit = process(&mut invoker, line);
                if should_exit {
                    break
                }
            },
            Err(ReadlineError::Interrupted) 
            | Err(ReadlineError::Eof) => {
                println!("Exit");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}