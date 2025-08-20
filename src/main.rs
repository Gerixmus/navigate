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
        
    let root = Node::new(path, None);

    let _result = create_file_leafs(&root);

    select_file(root);
}

fn select_file(root: Rc<RefCell<Node>>) -> Result<(), Error> {
    let _keep_root_alive = root.clone();
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
                let len = node.borrow().children.len();
                position = (position + 1) % len;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let len = node.borrow().children.len();
                position = (position + len - 1) % len;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let selected = node.borrow().children[position].clone();

                if !selected.borrow().path.is_dir() {
                    continue;
                }

                if selected.borrow().children.is_empty() {
                    create_file_leafs(&selected)?;
                }

                let len = node.borrow().children.len();
                clear_terminal(0, len as u16)?;
                stdout().execute(MoveTo(0, 0))?;

                node = selected;
                position = 0;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                kind: KeyEventKind::Press,
                ..
            }) => {
                clear_terminal(0, node.borrow().children.len() as u16)?;
                    if let Some(parent) = node.clone().borrow().parent.upgrade() {
                        node = parent;
                        // TODO: recalculate position
                        position = 0;
                    } else {
                    }
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
    fn new(path: PathBuf, parent: Option<&Rc<RefCell<Node>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            path,
            parent: parent.map(Rc::downgrade).unwrap_or_else(Weak::new),
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
        
        let child = Node::new(path, Some(node));

        node.borrow_mut().children.push(child);
    }
    Ok(())
}
