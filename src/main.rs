use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};
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
        
    let root = Node::new(path);

    let _result = create_file_leafs(&root);

    select_file(root);
}

fn select_file(root: Rc<RefCell<Node>>) -> Result<(), Error> {
    enable_raw_mode()?;
    stdout().execute(Hide)?;

    display_directory(root)?;

    disable_raw_mode()?;
    Ok(())
}

fn display_directory(mut node: Rc<RefCell<Node>>) -> Result<(), Error> {
    let mut position = 0;
    display_folder(position, &node.borrow().children);
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
                position = (position + 1) % node.borrow().children.len();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {
                position = (position + node.borrow().children.len() - 1) % node.borrow().children.len();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let len = node.borrow().children.len();
                let new_node = node.borrow().children[position].clone();
                create_file_leafs(&new_node)?;
                clear_terminal(0, len as u16)?;
                stdout().execute(MoveTo(0, 0))?;

                node = new_node;
                position = 0;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                kind: KeyEventKind::Press,
                ..
            }) => {
                clear_terminal(0, node.borrow().children.len() as u16)?;
                return Ok(());
            }
            _ => {}
        }

        clear_terminal(0, node.borrow().children.len() as u16)?;
        stdout().execute(MoveTo(0, 0))?;
        display_folder(position, &node.borrow().children);
    }
    Ok(())
}

fn display_folder(position: usize, children: &[Rc<RefCell<Node>>]) {
    for i in 0..children.len() {
        if i == position {
            println!("{}", children[i].borrow().to_string().magenta());
        } else {
            println!("{}", children[i].borrow())
        }
    }
}

#[derive(Debug)]
struct Node {
    path: PathBuf,
    parent: Weak<RefCell<Node>>,
    children: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(path: PathBuf) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            path,
            parent: Weak::new(),
            children: vec![]
        }))
    }
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

fn create_file_leafs(node: &Rc<RefCell<Node>>) -> Result<(), Error> {
    let entries = read_dir(&node.borrow().path)?;
    for entry in entries {
        let path = entry?.path();
        
        let child = Node::new(path);

        child.borrow_mut().parent = Rc::downgrade(node);

        node.borrow_mut().children.push(child);
    }
    Ok(())
}
