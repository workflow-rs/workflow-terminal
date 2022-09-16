/*
extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};
*/

// use workflow_terminal::{Result, cli::Terminal as TerminalTrait};
// use std::sync::Arc;
// use async_trait::async_trait;
// use workflow_terminal::Terminal;
// use workflow_terminal::Cli;
// use workflow_terminal::Options;
// use workflow_terminal::parse;

use workflow_terminal::Result;
use cli::example_terminal;

//use workflow_log::*;

/*
#[derive(Clone)]
pub struct LogSink{
    logs:Arc<Mutex<Vec<String>>>
}

impl workflow_log::Sink for LogSink {
    fn write(&self, _level:Level, args : &std::fmt::Arguments<'_>) -> bool {
        if let Some(logs) = self.logs.lock().unwrap().as_mut() {
            logs.push(args.to_string());
        }
        false
    }
}
*/  



// struct TestCli {
//     // term : Arc<Terminal>
// }

// impl TestCli {
//     pub fn new() -> TestCli {
//         TestCli {
//         }
//     }

//     // fn new(term:Arc<Terminal>)->Result<Arc<Self>>{
//     //     let handler = Arc::new(Self{term});
//     //     handler.term.register_handler(handler.clone())?;
//     //     Ok(handler)
//     // }
// }
/*
// optional: for binding to logs only!
impl workflow_log::Sink for TestCli {
    fn write(&self, _level:Level, args : &std::fmt::Arguments<'_>) -> bool {
        
        self.term.write(args.to_string())?;
        // return: 
        // - false for default log output handling (print to stdout or web console)
        // - true, to disable further processing (no further output is made)
        true
    }
}
*/

// #[async_trait]
// impl Cli for TestCli {
//     async fn digest(&self, term : &Arc<Terminal>, cmd: String) -> Result<()> {
//         //println!("cmd:: {}", cmd);
//         // let argv = cmd.split(' ').collect::<Vec<&str>>();
//         let argv = parse(&cmd);
//         //println!("argv[0]:: {}", argv[0]);
//         match argv[0].as_str() {
//             "hello" => {
//                 term.writeln("hello back to you!");
//             },
//             "exit" => {
//                 term.writeln("bye!");
//                 term.exit();
//             },
//             _ => {
//                 return Err(format!("command not found: {}", cmd).into())
//             }
//         }

//         Ok(())
//     }

//     async fn complete(&self, term : &Arc<Terminal>, substring : String) -> Result<Vec<String>> {
//         if substring.starts_with('a') {
//             Ok(vec!["alpha".to_string(), "aloha".to_string(), "albatross".to_string()])
//         } else {
//             Ok(vec![])
//         }
//     }
// }

#[async_std::main]
async fn main() -> Result<()>{

    example_terminal().await?;

    // let cli = Arc::new(TestCli::new());
    // let term = Arc::new(Terminal::new(cli,"$ "));
    // term.init().await?;
    // term.writeln("Terminal example");
    // term.run().await?;

    Ok(())
}
