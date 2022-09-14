/*
extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};
*/

use workflow_terminal::{Cli, Result, Terminal};
use std::sync::{Arc,Mutex};

fn main() ->Result<()>{
    let term = Arc::new(Terminal::new()?);
    let prompt = Arc::new(Mutex::new("$ ".to_string()));
    let cli = Cli::new(term, prompt)?;

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
