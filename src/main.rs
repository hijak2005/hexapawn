use std::{
    io::{Stdout, Write, stdout},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseButton, MouseEvent,
        MouseEventKind, poll, read,
    },
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal::{Clear, disable_raw_mode, enable_raw_mode},
};

const CELL_WIDTH: u16 = 7;
const CELL_HIGHT: u16 = 3;
const CELL_WIDTH_GAP: u16 = 2;
const CELL_HIGHT_GAP: u16 = 1;
const CELL_COLOR_COMPUTER: Color = Color::Yellow;
const CELL_COLOR_PLAYER: Color = Color::White;
const CELL_COLOR_EMPTY: Color = Color::DarkGrey;

struct Terminal {
    columns: u16,
    rows: u16,
}

struct Board {
    cells: [[Cell; 3]; 3],
}

#[derive(Clone, Copy, Default)]
struct Cell {
    // position relative to board
    column: u16,
    row: u16,
    cell_type: CellType,
}

#[derive(Clone, Copy, Default)]
enum CellType {
    #[default]
    Empty,
    Player,
    Computer,
}

impl Terminal {
    fn get_board_position(&self) -> (u16, u16) {
        (
            self.columns / 2 - CELL_WIDTH - CELL_WIDTH / 2 - CELL_WIDTH_GAP,
            self.rows / 2 - CELL_HIGHT - CELL_HIGHT / 2 - CELL_HIGHT_GAP,
        )
    }
}

impl Board {
    fn new() -> Self {
        let mut cells = [[Cell::default(); 3]; 3];
        for c in 0..3 {
            for r in 0..3 {
                cells[c][r].column = (CELL_WIDTH + CELL_WIDTH_GAP) * c as u16;
                cells[c][r].row = (CELL_HIGHT + CELL_HIGHT_GAP) * r as u16;
                cells[c][r].cell_type = match (c, r) {
                    (_, 0) => CellType::Computer,
                    (_, 2) => CellType::Player,
                    _ => CellType::Empty,
                }
            }
        }
        Board { cells }
    }
}

fn when_clicked(c: u16, r: u16) {
    println!("{}, {}", c, r)
}

fn draw(terminal: &Terminal, board: &Board) -> std::io::Result<()> {
    let (board_c, board_r) = terminal.get_board_position();
    let mut stdout = stdout();
    for c in board.cells {
        for r in c {
            draw_cell(
                &mut stdout,
                board_c + r.column,
                board_r + r.row,
                r.cell_type,
            )?;
        }
    }
    stdout.flush()?;
    Ok(())
}

fn draw_cell(
    stdout: &mut Stdout,
    column: u16,
    row: u16,
    cell_type: CellType,
) -> std::io::Result<()> {
    let color = match cell_type {
        CellType::Computer => CELL_COLOR_COMPUTER,
        CellType::Empty => CELL_COLOR_EMPTY,
        CellType::Player => CELL_COLOR_PLAYER,
    };
    for c in 0..CELL_WIDTH {
        for r in 0..CELL_HIGHT {
            queue!(
                stdout,
                SetBackgroundColor(color),
                MoveTo(column + c, row + r),
                Print(" "),
                ResetColor,
            )?;
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    execute!(
        std::io::stdout(),
        EnableMouseCapture,
        Hide,
        Clear(crossterm::terminal::ClearType::All)
    )?;

    // initialization
    let terminal = {
        let (columns, rows) = crossterm::terminal::size()?;
        Terminal { columns, rows }
    };
    let mut board = Board::new();

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

        draw(&terminal, &board)?;
    }

    execute!(
        std::io::stdout(),
        DisableMouseCapture,
        Show,
        Clear(crossterm::terminal::ClearType::All),
        MoveTo(0, 0),
    )?;
    disable_raw_mode()?;
    Ok(())
}
