use std::fmt;
use std::{
    env::current_dir,
    fs::read_dir,
    io::{stdout, Error},
    path::PathBuf,
};

use crossterm::cursor::MoveTo;
use crossterm::event::{
    self, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
};
use crossterm::style::{self, style, Color, Stylize};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{
    cursor::Hide,
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};

fn main() {
    let path = current_dir().unwrap();
    let mut root = Node {
        path: path,
        children: vec![],
    };
    let _result = create_file_leafs(&mut root);

    select_file(&mut root);
}

fn select_file(root: &mut Node) -> Result<(), Error> {
    enable_raw_mode()?;
    stdout().execute(Hide)?;

    display_directory(root);

    disable_raw_mode()?;
    Ok(())
}

fn display_directory(mut node: &mut Node) -> Result<(), Error> {
    let mut position = 0;
    display_folder(position, &node.children);
    loop {
        let event = read()?;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                ..
            }) => {
                position = (position + 1) % node.children.len();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {
                position = (position + node.children.len() - 1) % node.children.len();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let len = node.children.len();
                let new_node = &mut node.children[position];
                create_file_leafs(new_node);
                clear_terminal(0, len as u16)?;
                stdout().execute(MoveTo(0, 0))?;
                display_directory(new_node);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                kind: KeyEventKind::Press,
                ..
            }) => {
                clear_terminal(0, node.children.len() as u16)?;
                return Ok(());
            }
            _ => {}
        }

        clear_terminal(0, node.children.len() as u16)?;
        stdout().execute(MoveTo(0, 0))?;
        display_folder(position, &node.children);
    }
    Ok(())
}

fn display_folder(position: usize, children: &[Node]) {
    for i in 0..children.len() {
        if i == position {
            println!("{}", children[i].to_string().magenta());
        } else {
            println!("{}", children[i])
        }
    }
}

#[derive(Debug)]
struct Node {
    path: PathBuf,
    children: Vec<Node>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let file_name = self
            .path
            .file_name()
            .expect("no file name")
            .to_str()
            .expect("invalid UTF-8 in file name");
        write!(f, "{}", file_name)
    }
}

fn clear_terminal(start: u16, end: u16) -> std::io::Result<()> {
    for i in start..end {
        stdout().execute(MoveTo(0, i))?;
        stdout().execute(Clear(ClearType::CurrentLine))?;
    }
    Ok(())
}

fn create_file_leafs(node: &mut Node) -> Result<(), Error> {
    for entry in read_dir(&node.path)? {
        let path = entry?.path();
        node.children.push(Node {
            path: path,
            children: vec![],
        });
    }
    Ok(())
}
