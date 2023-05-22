use std::path::Path;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, BorderType, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        ListState, Paragraph, Row, Sparkline, Table, TableState, Tabs, Wrap,
    },
    Frame, Terminal,
};
use tobj;
use std::{error::Error, io, io::prelude::*, fs, process::Command};

struct StateList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StateList<T> {
    fn with_items(items: Vec<T>) -> StateList<T> {
        StateList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() / 3 - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() / 3 - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

enum AppMode {
    VertexList,
    FaceList,
    Help,
    Input,
}

struct App<'a> {
    pub mode: AppMode,
    
    models: StateList<tobj::Model>,

    vertices: StateList<f32>,

    faces: StateList<u32>,

    input: String,
    status: String,

    pub tab_titles: Vec<&'a str>,
    pub tab_index: usize,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        //initialize with hard-coded cube 
        let mut cube_mesh = tobj::Mesh {
            positions: Vec::new(),
            normals: Vec::new(),
            texcoords: Vec::new(),
            indices: Vec::new(),
            normal_indices: Vec::new(),
            texcoord_indices: Vec::new(),
            face_arities: Vec::new(),
            vertex_color: Vec::new(),
            material_id: None,
        };
        cube_mesh.positions = vec![
            1.0, 1.0, 1.0, 
            -1.0, 1.0, 1.0, 
            -1.0, 1.0, -1.0, 
            1.0, 1.0, -1.0, 
            1.0, -1.0, 1.0, 
            -1.0, -1.0, 1.0, 
            -1.0, -1.0, -1.0, 
            1.0, -1.0, -1.0,
        ];
        cube_mesh.indices = vec![
            1, 2, 3,
            1, 3, 4,
            5, 6, 7,
            5, 7, 8,
            1, 2, 6,
            1, 5, 6,
            2, 3, 7,
            2, 7, 6,
            3, 4, 8,
            3, 8, 7,
            4, 1, 5,
            4, 8, 5,
        ];
        let mut cube = tobj::Model::new(cube_mesh, "cube".to_string());
        
        App {
            mode: AppMode::Help,

            vertices: StateList::with_items(cube.mesh.positions.clone()),

            faces: StateList::with_items(cube.mesh.indices.clone()),

            models: StateList::with_items(vec![cube]),

            input: "".to_string(),
            status: "Welcome to tui_obj!".to_string(),

            tab_titles: vec!["Vertex", "Face", "Help"],
            tab_index: 2,
        }
    }

    fn obj_from_path(path: &Path) {
        //let obj = tobj::load_obj(path);
        //assert!(obj.is_ok());

        //let (models, materials) = obj.expect("Failed to load OBJ file");

        //for (i, m) in models.iter().enumerate() {
        //    let mesh = &m.mesh();


        //}


    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tab_titles.len();
    }

    pub fn prev_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = self.tab_titles.len() - 1;
        }
    }

    pub fn set_tab(&mut self, tab: usize) {
        self.tab_index = tab;
        
        self.status = format!("Switched to {} Mode", self.tab_titles[tab]);
    }

    pub fn next_item(&mut self) {
        match self.tab_index {
            0 => self.vertices.next(),
            1 => self.faces.next(),
            _ => {}
        }
    }

    pub fn prev_item(&mut self) {
        match self.tab_index {
            0 => self.vertices.previous(),
            1 => self.faces.previous(),
            _ => {}
        }
    }

    pub fn next_model(&mut self) {

    }

    pub fn prev_model(&mut self) {

    }

    pub fn open_file(&mut self) {
        let path = self.get_input("File Location: ");
        
        let output = Command::new("python3")
            .args(["microservice_helper.py", path])
            .output()
            .expect("failed to execute process");

        //let obj = tobj::load_obj(Path::new(path), &tobj::GPU_LOAD_OPTIONS);
        //assert!(obj.is_ok());

        //let (models, materials) = obj.expect("Failed to load OBJ file");

        //self.vertices = StateList::with_items(models[0].mesh.positions.clone());
        //self.faces = StateList::with_items(models[0].mesh.indices.clone());

        //self.models = StateList::with_items(models);
    }

    pub fn write_file(&mut self) {

    }

    pub fn new_item(&mut self) {

    }

    pub fn delete_item(&mut self) {

    }

    pub fn translate(&mut self) {

    }

    fn backup(&mut self) {

    }

    fn restore(&mut self) {

    }

    fn get_input(&mut self, prompt: &str) -> &str {
        "./cube.stl"
    }
}

fn main() -> io::Result<()> {
    //init terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    //run terminal app
    let app = App::new();
    let exit_res = run(&mut terminal, app);    

    //reset terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;        

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    //commands
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('o') => app.open_file(),
                    KeyCode::Char('w') => app.write_file(),
                    KeyCode::Char('u') => app.restore(),
                    KeyCode::Right => app.next_model(),
                    KeyCode::Left => app.prev_model(),
                    //tabs
                    KeyCode::Char('v') => app.set_tab(0),
                    KeyCode::Char('f') => app.set_tab(1),
                    KeyCode::Char('h') => app.set_tab(2),
                    _ => {}
                }
                if app.tab_index < 2 {
                    match key.code {
                        //list controls
                        KeyCode::Down => app.next_item(),
                        KeyCode::Up => app.prev_item(),
                        KeyCode::Char('d') => app.new_item(),
                        KeyCode::Char('d') => app.delete_item(),
                        KeyCode::Char('t') => app.translate(),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(size);

    let header = Paragraph::new("tui_OBJ 2023 - copyright Simon Eagar - all rights reserved")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("tui_OBJ")
                .border_type(BorderType::Plain),
        );
            
    let tab_menu = app
        .tab_titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
           ])
        })
        .collect();
            
    let tabs = Tabs::new(tab_menu)
        .block(Block::default().title("Modes").borders(Borders::ALL))
        .select(app.tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"));

    let mut vertices: Vec<ListItem> = vec![];
    
    for i in 0..app.vertices.items.len() / 3 {
        let mut lines = Spans::from(vec![
            Span::raw(format!("Vertex {}:", i + 1)),
            Span::raw(format!("    {}", app.vertices.items[3 * i])),
            Span::raw(format!("    {}", app.vertices.items[3 * i + 1])),
            Span::raw(format!("    {}", app.vertices.items[3 * i + 2])),
        ]);
        let mut lines_item: ListItem = ListItem::new(lines);
        vertices.push(lines_item);
    }

    let list_vertex = List::new(vertices)
        .block(Block::default().borders(Borders::ALL).title("Vertices"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut faces: Vec<ListItem> = vec![];
    
    for i in 0..app.faces.items.len() / 3 {
        let mut lines = Spans::from(vec![
            Span::raw(format!("Face {}:", i + 1)),
            Span::raw(format!("    {}", app.faces.items[3 * i])),
            Span::raw(format!("    {}", app.faces.items[3 * i + 1])),
            Span::raw(format!("    {}", app.faces.items[3 * i + 2])),
        ]);
        let mut lines_item: ListItem = ListItem::new(lines);
        faces.push(lines_item);
    }

    let list_face = List::new(faces)
        .block(Block::default().borders(Borders::ALL).title("Faces"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let help = Paragraph::new("Main Commands\n    Q | Quit        - Close the program\n\n    H | Help        - Switch to this interface\n\n    V | Vertex Mode - Switch to vertex editing interface\n\n    F | Face Mode   - Switch to face editing interface\n\nVertex Mode\n\nFaces Mode")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Quick Commands")
                .border_type(BorderType::Plain),
        );

    let status_bar = Paragraph::new(&*app.status)
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Status Update")
                .border_type(BorderType::Plain),
        );

    let footer = Paragraph::new("Q | Quit   Up/Down | Select")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Help")
                .border_type(BorderType::Plain),
        );

    f.render_widget(header, chunks[0]);
    f.render_widget(tabs, chunks[1]);

    match app.tab_index {
        0 => f.render_stateful_widget(list_vertex, chunks[2], &mut app.vertices.state),
        1 => f.render_stateful_widget(list_face, chunks[2], &mut app.faces.state),
        2 => f.render_widget(help, chunks[2]),
        _ => unreachable!(),
    };

    f.render_widget(status_bar, chunks[3]);
    f.render_widget(footer, chunks[4]);
}
