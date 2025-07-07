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
const CELL_COLOR_SELECTED: Color = Color::Rgb { r: 0, g: 200, b: 0 };
const CELL_COLOR_AVAILABLE: Color = Color::Blue;

struct Board {
    cells: Vec<Cell>,
    board_status: BoardStatus,
    selected_cell: Option<Cell>,
    available_cells: Vec<Cell>,
    selectable_cells: Vec<Cell>,
    screen: Screen,
    running: bool,
}

struct Screen {
    columns: u16,
    rows: u16,
}

#[derive(Clone, Copy)]
struct Cell {
    column: u16,
    row: u16,
    cell_type: CellType,
}

struct Mouse {
    column: u16,
    row: u16,
}

#[derive(Clone, Copy)]
enum CellType {
    Empty,
    Player,
    Computer,
}

enum BoardStatus {
    Selecting,
    Moving,
    Computer,
}

impl Board {
    fn new() -> std::io::Result<Self> {
        let mut cells = Vec::new();
        for c in 0..3 {
            for r in 0..3 {
                let cell = Cell {
                    column: c,
                    row: r,
                    cell_type: match (c, r) {
                        (_, 0) => CellType::Computer,
                        (_, 2) => CellType::Player,
                        _ => CellType::Empty,
                    },
                };
                cells.push(cell);
            }
        }
        let selectable_cells: Vec<Cell> = cells
            .iter()
            .filter(|cell| matches!(cell.cell_type, CellType::Player))
            .cloned()
            .collect();
        let (columns, rows) = crossterm::terminal::size()?;
        Ok(Board {
            cells,
            board_status: BoardStatus::Selecting,
            selected_cell: None,
            available_cells: Vec::new(),
            selectable_cells,
            screen: Screen { columns, rows },
            running: true,
        })
    }

    fn get_c(&self) -> u16 {
        self.screen.columns / 2 - CELL_WIDTH - CELL_WIDTH / 2 - CELL_WIDTH_GAP
    }

    fn get_r(&self) -> u16 {
        self.screen.rows / 2 - CELL_HIGHT - CELL_HIGHT / 2 - CELL_HIGHT_GAP
    }

    fn update_selected(&mut self, mouse: &Mouse) {
        self.selected_cell = self
            .selectable_cells
            .iter()
            .find(|cell| cell.is_clicked(self, mouse))
            .cloned();
    }

    fn draw(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        for cell in &self.cells {
            cell.draw(stdout, self)?;
        }
        if let Some(cell) = &self.selected_cell {
            cell.draw_selected(stdout, self)?;
        }
        for cell in &self.available_cells {
            cell.draw_available(stdout, self)?;
        }
        Ok(())
    }
}

impl Cell {
    fn get_color(&self) -> Color {
        match self.cell_type {
            CellType::Computer => CELL_COLOR_COMPUTER,
            CellType::Empty => CELL_COLOR_EMPTY,
            CellType::Player => CELL_COLOR_PLAYER,
        }
    }

    fn get_start_c(&self, board_c: u16) -> u16 {
        self.column * (CELL_WIDTH + CELL_WIDTH_GAP) + board_c
    }

    fn get_start_r(&self, board_r: u16) -> u16 {
        self.row * (CELL_HIGHT + CELL_HIGHT_GAP) + board_r
    }

    fn is_clicked(&self, board: &Board, mouse: &Mouse) -> bool {
        self.get_start_c(board.get_c()) <= mouse.column
            && mouse.column <= self.get_start_c(board.get_c()) + CELL_WIDTH
            && self.get_start_r(board.get_r()) <= mouse.row
            && mouse.row <= self.get_start_r(board.get_r()) + CELL_HIGHT
    }

    fn draw(&self, stdout: &mut Stdout, board: &Board) -> std::io::Result<()> {
        for c in 0..CELL_WIDTH {
            for r in 0..CELL_HIGHT {
                queue!(
                    stdout,
                    SetBackgroundColor(self.get_color()),
                    MoveTo(
                        self.get_start_c(board.get_c()) + c,
                        self.get_start_r(board.get_r()) + r,
                    ),
                    Print(" "),
                    ResetColor,
                )?;
            }
        }
        Ok(())
    }

    fn draw_selected(&self, stdout: &mut Stdout, board: &Board) -> std::io::Result<()> {
        for c in 0..CELL_WIDTH {
            for r in 0..CELL_HIGHT {
                queue!(
                    stdout,
                    SetBackgroundColor(CELL_COLOR_SELECTED),
                    MoveTo(
                        self.get_start_c(board.get_c()) + c,
                        self.get_start_r(board.get_r()) + r,
                    ),
                    Print(" "),
                    ResetColor,
                )?;
            }
        }
        Ok(())
    }

    fn draw_available(&self, stdout: &mut Stdout, board: &Board) -> std::io::Result<()> {
        for c in (0..1).chain(CELL_WIDTH - 1..CELL_WIDTH) {
            for r in (0..1).chain(CELL_HIGHT - 1..CELL_HIGHT) {
                queue!(
                    stdout,
                    SetBackgroundColor(CELL_COLOR_AVAILABLE),
                    MoveTo(
                        self.get_start_c(board.get_c()) + c,
                        self.get_start_r(board.get_r()) + r,
                    ),
                    Print(" "),
                    ResetColor,
                )?;
            }
        }
        Ok(())
    }
}

fn when_clicked(board: &mut Board, mouse: Mouse) -> std::io::Result<()> {
    board.update_selected(&mouse);
    draw(&board)?;
    Ok(())
}

fn draw(board: &Board) -> std::io::Result<()> {
    let mut stdout = stdout();
    board.draw(&mut stdout)?;
    stdout.flush()?;
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
    let mut board = Board::new()?;

    while board.running {
        if poll(Duration::from_millis(500))? {
            match read()? {
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    ..
                }) => when_clicked(&mut board, Mouse { column, row }),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => {
                    board.running = false;
                    Ok(())
                }
                _ => Ok(()),
            }?;
        };
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
