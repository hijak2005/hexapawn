#![allow(dead_code)]

use std::time::Duration;

use crossterm::{
    cursor::{Hide, Show},
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseButton, MouseEvent,
        MouseEventKind, poll, read,
    },
    execute,
    terminal::{Clear, disable_raw_mode, enable_raw_mode},
};

fn main() -> std::io::Result<()> {
    let mut sc = std::io::stdout();
    let (max_c, max_r) = crossterm::terminal::size()?;
    enable_raw_mode()?;
    execute!(
        std::io::stdout(),
        EnableMouseCapture,
        Hide,
        Clear(crossterm::terminal::ClearType::All)
    )?;

    loop {
        if poll(Duration::from_millis(500))? {
            match read()? {
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    ..
                }) => when_clicked(column, row),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => break,
                _ => (),
            }
        }
    }

    execute!(
        std::io::stdout(),
        DisableMouseCapture,
        Show,
        Clear(crossterm::terminal::ClearType::All)
    )?;
    disable_raw_mode()?;
    Ok(())
}

struct Board {
    cells: [[Cell; 3]; 3],
}

impl Board {
    fn new() -> Board {
        let mut cells = [[Cell::default(); 3]; 3];
        for c in 0..3 {
            for r in 0..3 {
                cells[c][r] = Cell::new(c as u16, r as u16, CellType::Empty)
            }
        }
        Board { cells }
    }
}

#[derive(Clone, Copy, Default)]
struct Cell {
    column: u16,
    row: u16,
    cell_type: CellType,
}

impl Cell {
    fn new(column: u16, row: u16, cell_type: CellType) -> Cell {
        Cell {
            column,
            row,
            cell_type,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum CellType {
    #[default]
    Empty,
    Player,
    Computer,
}

fn when_clicked(c: u16, r: u16) {
    println!("{}, {}", c, r)
}

// fn draw()
