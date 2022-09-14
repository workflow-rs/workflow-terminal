/*
extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};
*/

use workflow_terminal::{Cli, Result, Terminal};
use workflow_log::*;
use std::sync::{Arc,Mutex};

// ^=================================================================
#[async_trait]
trait CliHandler {
    async fn digest(&self, cmd: String) -> Result<()>;
    async fn complete(&self, substring : String) -> Result<Vec<String>>;
}
// ^=================================================================

#[derive(Clone)]
pub struct LogSink;

impl workflow_log::Sink for LogSink {
    fn write(&self, _level:Level, args : &std::fmt::Arguments<'_>) -> bool {
        if let Some(logs) = self.logs.lock().unwrap().as_mut() {
            logs.push(args.to_string());
        }
        false
    }
}    



struct TestCli {
    term : Arc<dyn Terminal>
}

impl TestCli {
    pub fn new(term : &Arc<dyn Terminal>) -> TestCli {
        TestCli {
            term : term.clone()
        }
    }
}

// optional: for binding to logs only!
impl workflow_log::Sink for TestCli {
    fn write(&self, _level:Level, args : &std::fmt::Arguments<'_>) -> bool {
        
        self.term.write(args.to_string());
        // return: 
        // - false for default log output handling (print to stdout or web console)
        // - true, to disable further processing (no further output is made)
        true
    }
}

#[async_trait]
impl CliHandler for TestCli {
    async fn digest(&self, cmd: String) -> Result<()> {

        let argv = cmd.split(' ').collect::<Vec<&str>>();

        match argv[0] {
            "hello" => {
                log_trace!("hello back to you!");
            },
            _ => {
                return Err("Unknown command")
            }
        }

        Ok(())
    }

    async fn complete(&self, substring : String) -> Result<Vec<String>> {
        if substring.starts_with('a') {
            vec!["alpha", "aloha", "albatross"]
        } else {
            vec![]
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

    let term = Arc::new(Terminal::new()?);
    let prompt = Arc::new(Mutex::new("$ ".to_string()));
    let cli = Cli::new(term.clone(), prompt)?;

    let handler = TestCli::new(&term);

    cli.start();

    /*
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout,
           "{}{}q to exit. Type stuff, use alt, and so on.{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Hide)
            .unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys() {
        
        write!(stdout,
               "{}{}",
               termion::cursor::Goto(1, 1),
               termion::clear::CurrentLine)
                .unwrap();
        

        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char(c) => {
                if c == '\n' || c == '\r'{
                    print!("enter: {}", c);
                }else{
                    print!("{}", c);
                }
            },
            Key::Alt(c) => print!("^{}", c),
            Key::Ctrl(c) => print!("*{}", c),
            Key::Esc => print!("ESC"),
            Key::Left => print!("←"),
            Key::Right => print!("→"),
            Key::Up => print!("↑"),
            Key::Down => print!("↓"),
            Key::Backspace => print!("×"),
            //Key::Insert => println!("\r\n"),
            _ => {}
        }
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
    */

    Ok(())
}
