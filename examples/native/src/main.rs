/*
extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};
*/

use workflow_terminal::{Result, cli::Terminal as TerminalTrait};
use std::sync::Arc;
use async_trait::async_trait;
use workflow_terminal::Terminal;
use workflow_terminal::CliHandler;
use workflow_terminal::Options;
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



struct TestCli {
    term : Arc<Terminal>
}

impl TestCli {
    fn new(term:Arc<Terminal>)->Result<Arc<Self>>{
        let handler = Arc::new(Self{term});
        handler.term.register_handler(handler.clone())?;
        Ok(handler)
    }
}
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

#[async_trait]
impl CliHandler for TestCli {
    async fn digest(&self, cmd: String) -> Result<()> {
        //println!("cmd:: {}", cmd);
        let argv = cmd.split(' ').collect::<Vec<&str>>();
        //println!("argv[0]:: {}", argv[0]);
        match argv[0] {
            "hello" => {
                self.term.write_str("hello back to you!")?;
            },
            _ => {
                return Err("Unknown command".into())
            }
        }

        Ok(())
    }

    async fn complete(&self, substring : String) -> Result<Vec<String>> {
        if substring.starts_with('a') {
            Ok(vec!["alpha".to_string(), "aloha".to_string(), "albatross".to_string()])
        } else {
            Ok(vec![])
        }
    }
}

fn main() ->Result<()>{

    //^
    //^ TODO perhaps (to simplify) we don't want to create Terminal here
    //^ we want to make Cli create the term automatically
    //^ but pass target Element in TerminalOptions passed to Cli
    //^ i.e. more like:  
    //^     let cli = Cli::new(Options { target_element : Some(el), prompt });
    //^

    /*
    let term = Arc::new(Terminal::new()?);
    let prompt = Arc::new(Mutex::new("$ ".to_string()));
    let cli = Cli::new(term.clone(), prompt)?;
    */

    let term = Terminal::new(Options{
        prompt:"$ ".to_string()
    })?;
    let handler = TestCli::new(term)?;

    handler.term.write_str("Example of Native Terminal:")?;
    handler.term.prompt()?;
    handler.term.start()?;
    



    Ok(())
}
