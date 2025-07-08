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
const CELL_COLOR_SELECTED: Color = Color::AnsiValue(158);
const CELL_COLOR_MOVE: Color = Color::Blue;

struct Board {
    cells: Vec<Cell>,
    selected_cell: Option<Cell>,
    move_cells: Vec<Cell>,
    running: bool,
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
    Selected,
    Move,
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
        let (columns, rows) = crossterm::terminal::size()?;
        Ok(Board {
            cells,
            selected_cell: None,
            move_cells: Vec::new(),
            running: true,
            columns,
            rows,
        })
    }

    fn get_c(&self) -> u16 {
        self.columns / 2 - CELL_WIDTH - CELL_WIDTH / 2 - CELL_WIDTH_GAP
    }

    fn get_r(&self) -> u16 {
        self.rows / 2 - CELL_HIGHT - CELL_HIGHT / 2 - CELL_HIGHT_GAP
    }

    fn update_selected(&mut self, mouse: &Mouse) {
        self.selected_cell = self
            .cells
            .iter()
            .find(|cell| cell.is_clicked(self, mouse) && matches!(cell.cell_type, CellType::Player))
            .cloned()
            .map(|mut cell| {
                cell.cell_type = CellType::Selected;
                cell
            });
    }

    fn update_move(&mut self) {
        self.move_cells.clear();
        if let Some(cell) = &self.selected_cell {
            let (column, row) = (cell.column, cell.row);
            if let Some(empty_front) = self
                .cells
                .iter()
                .find(|c| c.column == column && c.row + 1 == row)
            {
                if matches!(empty_front.cell_type, CellType::Empty) {
                    self.move_cells.push(Cell {
                        column,
                        row: row - 1,
                        cell_type: CellType::Move,
                    });
                }
            }
            if let Some(enemy_right) = self
                .cells
                .iter()
                .find(|c| c.column == column + 1 && c.row + 1 == row)
            {
                if matches!(enemy_right.cell_type, CellType::Empty) {
                    self.move_cells.push(Cell {
                        column,
                        row: row - 1,
                        cell_type: CellType::Move,
                    });
                }
            }
            if let Some(enemy_left) = self
                .cells
                .iter()
                .find(|c| c.column + 1 == column && c.row + 1 == row)
            {
                if matches!(enemy_left.cell_type, CellType::Empty) {
                    self.move_cells.push(Cell {
                        column,
                        row: row - 1,
                        cell_type: CellType::Move,
                    });
                }
            }
        }
    }

    fn update_cells(){
        
    }

    fn draw(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        for cell in &self.cells {
            cell.draw(stdout, self)?;
        }
        if let Some(cell) = &self.selected_cell {
            cell.draw(stdout, self)?;
        }
        for cell in &self.move_cells {
            cell.draw(stdout, self)?;
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
            CellType::Selected => CELL_COLOR_SELECTED,
            CellType::Move => CELL_COLOR_MOVE,
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
        let (columns, rows) = if matches!(self.cell_type, CellType::Move) {
            (
                (0..1).chain(CELL_WIDTH - 1..CELL_WIDTH),
                (0..1).chain(CELL_HIGHT - 1..CELL_HIGHT),
            )
        } else {
            ((0..CELL_WIDTH).chain(1..0), (0..CELL_HIGHT).chain(1..0))
        };
        for c in columns {
            for r in rows.clone() {
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
}

fn when_clicked(board: &mut Board, mouse: Mouse) -> std::io::Result<()> {
    board.update_selected(&mouse);
    board.update_move();
    draw(&board)?;
    Ok(())
}

fn draw(board: &Board) -> std::io::Result<()> {
    let mut stdout = stdout();
    queue!(stdout, Clear(crossterm::terminal::ClearType::All))?;
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
    when_clicked(&mut board, Mouse { column: 0, row: 0 })?;

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
