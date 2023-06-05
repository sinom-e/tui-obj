use std::{error::Error, path::Path, io, io::prelude::*, fs, process::Command, thread, time, cmp, sync::Mutex, sync::Arc};
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
    widgets::canvas::{Canvas, Context, Line, Map, MapResolution, Rectangle, Points, Painter},
    widgets::{
        Axis, BarChart, Block, Borders, BorderType, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        ListState, Paragraph, Row, Sparkline, Table, TableState, Tabs, Wrap,
    },
    Frame, Terminal,
};
use tobj;

struct StateList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StateList<T> { //interactive item list
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

enum AppMode { //app states
    VertexList,
    FaceList,
    Help,
}

enum StatusMode { //command states
    Normal,
    Open,
}

struct App<'a> {
    pub mode: AppMode,
    
    models: StateList<tobj::Model>, //list of loaded models (not yet used)

    vertices: StateList<f32>, //list of vertex coordinates

    faces: StateList<u32>, //list of vertex indices forming triangular faces

    input: String, //used for commands
    status: String, //used for user feedback
    status_mode: StatusMode, //current command state
    
    rotation_offset: f64, //rotation tick for rendering
    x_offset: f64, //viewport translation
    y_offset: f64,
    zoom: f64, //scaled bounds of viewport
    top_down: bool, //view model from top

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
            0, 1, 2,
            0, 2, 3,
            4, 5, 6,
            4, 6, 7,
            0, 1, 5,
            0, 4, 5,
            1, 2, 6,
            1, 6, 5,
            2, 3, 7,
            2, 7, 6,
            3, 0, 4,
            3, 7, 4,
        ];
        let mut cube = tobj::Model::new(cube_mesh, "cube".to_string());
        
        App { //default values
            mode: AppMode::Help,

            vertices: StateList::with_items(cube.mesh.positions.clone()),

            faces: StateList::with_items(cube.mesh.indices.clone()),
            
            models: StateList::with_items(vec![cube]),

            input: "".to_string(),
            status: "Welcome to tui_obj!".to_string(),
            status_mode: StatusMode::Normal,

            rotation_offset: 0.0,
            x_offset: 0.0,
            y_offset: 0.0,
            zoom: 10.0,
            top_down: false,
            
            tab_titles: vec!["Vertex", "Face", "Help"],
            tab_index: 2,
        }
    }

    pub fn next_tab(&mut self) { //app control functions
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

    pub fn next_model(&mut self) { //model selection functions; not yet used

    }

    pub fn prev_model(&mut self) {

    }

    pub fn open_file(&mut self, path: &str) { //file read
        let mut new_path = path.to_string();
        if path.contains(".stl") {
            let mut output = Command::new("python3")
                .args(["microservice_helper.py", path])
                .output()
                .expect("failed to execute process");

            output.stdout.pop(); //remove null terminator!!

            new_path = match std::str::from_utf8(&output.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8: {}", e)
            };
        }

        let obj = tobj::load_obj(Path::new(&new_path), &tobj::GPU_LOAD_OPTIONS);
        if !obj.is_ok() {
            self.status = format!("Failed to load file: {}", path);
            return;
        }

        let (models, materials) = obj.expect("Failed to load OBJ file");

        self.vertices = StateList::with_items(models[0].mesh.positions.clone());
        self.faces = StateList::with_items(models[0].mesh.indices.clone());

        self.models = StateList::with_items(models);
        
        self.status = format!("Opened file: {}", path);
    }

    pub fn write_file(&mut self) { //file write; not yet used

    }

    pub fn new_item(&mut self) { //edit functions; not yet used

    }

    pub fn delete_item(&mut self) {

    }

    pub fn translate(&mut self) {

    }

    fn backup(&mut self) { //undo functionality functions; not yet used

    }

    fn restore(&mut self) {

    }
    
    fn zoom_in(&mut self, factor: f64) { //viewport control functions
        self.zoom = self.zoom / 1.1
    }
    
    fn zoom_out(&mut self, factor: f64) {
        self.zoom = self.zoom * 1.1
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
        
        //draw TUI
        terminal.draw(|f| ui(f, &mut app))?;        

        //read input event
        if let Event::Key(key) = event::read()? {
      	    match app.status_mode {
                StatusMode::Normal => if key.kind == KeyEventKind::Press {
                    match key.code {
                        //commands
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('o') => {
                            app.status = "".to_string();
                            app.status_mode = StatusMode::Open;
                        },
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
                            KeyCode::Char('n') => app.new_item(),
                            KeyCode::Char('d') => app.delete_item(),
                            KeyCode::Char('t') => app.translate(),
                            //viewport controls
                            KeyCode::Char('-') => app.zoom_out(1.1),
                            KeyCode::Char('+') => app.zoom_in(1.1),
                            KeyCode::Char('7') => app.rotation_offset += -0.02,
                            KeyCode::Char('9') => app.rotation_offset +=  0.02,
                            KeyCode::Char('8') => app.y_offset +=  0.05 * app.zoom,
                            KeyCode::Char('2') => app.y_offset += -0.05 * app.zoom,
                            KeyCode::Char('6') => app.x_offset +=  0.05 * app.zoom,
                            KeyCode::Char('4') => app.x_offset += -0.05 * app.zoom,
                            KeyCode::Char('5') => app.top_down = !app.top_down,
                            _ => {}
                        }
                    }
                },
                //generic command input mode
                _ => if key.kind == KeyEventKind::Press { 
                    match key.code {
                        KeyCode::Enter => {
                            match app.status_mode {
                                //specific command function handled here
                                StatusMode::Open => {
                                    app.status_mode = StatusMode::Normal;
                                    app.open_file(&app.status.to_string());
                                },
                                _ => unreachable!()
                            }
                        }
                        KeyCode::Char(c) => {
                            app.status.push(c);
                        }
                        KeyCode::Backspace => {
                            app.status.pop();
                        }
                        KeyCode::Esc => {
                            app.status_mode = StatusMode::Normal;
                            app.status = "Operation cancelled".to_string();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) { //TUI handler
    let size = f.size();
    
    //divide interface amongst elements
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

    draw_header(f, app, chunks[0]);
    draw_tab_menu(f, app, chunks[1]);

    //render selected tab
    match app.tab_index {
        0 => draw_vertex_tab(f, app, chunks[2]),
        1 => draw_face_tab(f, app, chunks[2]),
        2 => draw_help(f, app, chunks[2]),
        _ => unreachable!(),
    };

    draw_status(f, app, chunks[3]);
    draw_footer(f, app, chunks[4]);
}

fn draw_header<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //header bit
where
    B: Backend,
{
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
    
    f.render_widget(header, area);
}

fn draw_tab_menu<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //tabs
where
    B: Backend,
{
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
    
    f.render_widget(tabs, area);
}

fn draw_status<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //status message
where
    B: Backend,
{
    let status_bar;
    
    //match formatting to app state
    match app.status_mode {
        StatusMode::Normal => {
            status_bar = Paragraph::new(&*app.status)
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Status Update")
                        .border_type(BorderType::Plain),
                );
        },
        StatusMode::Open => {
            status_bar = Paragraph::new(&*app.status)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Yellow))
                        .title("Open File")
                        .border_type(BorderType::Plain),
                );
        },
        _ => {
            status_bar = Paragraph::new(&*app.status)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Yellow))
                        .title("")
                        .border_type(BorderType::Plain),
                );
        }
    }
    
    f.render_widget(status_bar, area);
}

fn draw_footer<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //footer bit
where
    B: Backend,
{
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
    
    f.render_widget(footer, area);
}

fn draw_vertex_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //vert list & viewport
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ]
            .as_ref(),
        )
        .split(area);
    draw_vertex_list(f, app, chunks[0]);
    draw_viewport(f, app, chunks[1]);
}

fn draw_face_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //face list & viewport
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ]
            .as_ref(),
        )
        .split(area);
    draw_face_list(f, app, chunks[0]);
    draw_viewport(f, app, chunks[1]);
}

fn draw_vertex_list<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //build vertex list
where
    B: Backend,
{
    let mut vertices: Vec<ListItem> = vec![];
    
    //build formatted list from vertex data
    for i in 0..app.vertices.items.len() / 3 {
        let mut lines = Spans::from(vec![
            Span::raw(format!("v{}:", i + 1)),
            Span::raw(format!("    {}", app.vertices.items[3 * i])),
            Span::raw(format!("    {}", app.vertices.items[3 * i + 1])),
            Span::raw(format!("    {}", app.vertices.items[3 * i + 2])),
        ]);
        let mut lines_item: ListItem = ListItem::new(lines);
        vertices.push(lines_item);
    }

    //highlight selected item
    let list_vertex = List::new(vertices)
        .block(Block::default().borders(Borders::ALL).title("Vertices"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list_vertex, area, &mut app.vertices.state);
}

fn draw_face_list<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //build face list
where
    B: Backend,
{
    let mut faces: Vec<ListItem> = vec![];
    
    //build formatted list from vertex data
    for i in 0..app.faces.items.len() / 3 {
        let mut lines = Spans::from(vec![
            Span::raw(format!("f{}:", i + 1)),
            Span::raw(format!("    {}", app.faces.items[3 * i])),
            Span::raw(format!("    {}", app.faces.items[3 * i + 1])),
            Span::raw(format!("    {}", app.faces.items[3 * i + 2])),
        ]);
        let mut lines_item: ListItem = ListItem::new(lines);
        faces.push(lines_item);
    }

    //highlight selected item
    let list_face = List::new(faces)
        .block(Block::default().borders(Borders::ALL).title("Faces"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list_face, area, &mut app.faces.state);
}

fn draw_viewport<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //draw viewport; just redirect to necessary function
where
    B: Backend,
{
    if app.tab_index == 0 {
        dot_render(f, app, area);
    } else {
        line_render(f, app, area);
    };
}

fn dot_render<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //render dot model
where
    B: Backend,
{
    let x_zoom = app.zoom;
    let y_zoom = app.zoom * 2.0 * area.height as f64 / area.width as f64;
    let (y_component, z_component) = match app.top_down { //determine whether model will rotate about y-axis or z-axis
        false => (1, 2),
        true  => (2, 1),
    };
    
    let mut viewport = Canvas::default()
    	.block(Block::default().title("Viewport").borders(Borders::ALL))
    	.x_bounds([x_zoom * -1.0, x_zoom])
    	.y_bounds([y_zoom * -1.0, y_zoom]);
    
    let positions = &app.vertices.items;
    let mut points: Vec<(f64, f64)> = Vec::new();
    
    //convert raw position data into renderable points
    for index in 0..positions.len() / 3 {
        let i = index * 3;
        //basic sine/cosine rotation transform about y-axis
        let x = {
            positions[i] as f64 * 
            app.rotation_offset.sin() + 
            positions[i + z_component] as f64 * 
            app.rotation_offset.cos()
        };
        points.push((x, positions[i + y_component] as f64));
    }
    
    //draw points
    viewport = viewport.paint(|ctx| {
        ctx.draw(&Points {
            coords: &points,
            color: Color::White,
        });
        //highlight selected point
        match app.vertices.state.selected() {
            Some(value) => {
                let i = value * 3;
                let x = {
                    positions[i] as f64 * 
                    app.rotation_offset.sin() + 
                    positions[i + z_component] as f64 * 
                    app.rotation_offset.cos()
                };
                ctx.draw(&Points {
                    coords: &[(x, positions[i + y_component] as f64)],
                    color: Color::Yellow,
                });
            }
            _ => {}
        }
    });

    f.render_widget(viewport, area);
}

fn line_render<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //render wireframe
where
    B: Backend,
{
    let x_zoom = app.zoom;
    let y_zoom = app.zoom * 2.0 * area.height as f64 / area.width as f64;
    let (y_component, z_component) = match app.top_down {
        false => (1, 2),
        true  => (2, 1),
    };
    
    let mut viewport = Canvas::default()
    	.block(Block::default().title("Viewport").borders(Borders::ALL))
    	.x_bounds([x_zoom * -1.0 + app.x_offset, x_zoom + app.x_offset])
    	.y_bounds([y_zoom * -1.0 + app.y_offset, y_zoom + app.y_offset]);
    
    //draw lines between each vertex of each face
    viewport = viewport.paint(|ctx| {
        let positions = &app.vertices.items;
        let indices = &app.faces.items;
        
        for i in 0..indices.len() {
            let j = match i % 3 {
                2 => i - 2,
                _ => i + 1,
            };
            let f1 = indices[i] as usize * 3;
            let f2 = indices[j] as usize * 3;
            let x1 = {
                positions[f1] as f64 * 
                app.rotation_offset.sin() + 
                positions[f1 + z_component] as f64 * 
                app.rotation_offset.cos()
            };
            let x2 = {
                positions[f2] as f64 * 
                app.rotation_offset.sin() + 
                positions[f2 + z_component] as f64 * 
                app.rotation_offset.cos()
            };
            let y1 = positions[f1 + y_component] as f64;
            let y2 = positions[f2 + y_component] as f64;
            
            ctx.draw(&Line {
                x1: x1,
                x2: x2,
                y1: y1,
                y2: y2,
                color: Color::White,
            });
        }
        
        //highlight selected edges
        match app.faces.state.selected() {
            Some(value) => {
                for i in 0..3 {
                    let j = match i % 3 {
                        2 => i - 2,
                        _ => i + 1,
                    };
                    let f1 = indices[value * 3 + i] as usize * 3;
                    let f2 = indices[value * 3 + j] as usize * 3;
                    let x1 = {
                        positions[f1] as f64 * 
                        app.rotation_offset.sin() + 
                        positions[f1 + z_component] as f64 * 
                        app.rotation_offset.cos()
                    };
                    let x2 = {
                        positions[f2] as f64 * 
                        app.rotation_offset.sin() + 
                        positions[f2 + z_component] as f64 * 
                        app.rotation_offset.cos()
                    };
                    let y1 = positions[f1 + y_component] as f64;
                    let y2 = positions[f2 + y_component] as f64;
            
                    ctx.draw(&Line {
                        x1: x1,
                        x2: x2,
                        y1: y1,
                        y2: y2,
                        color: Color::Yellow,
                    });
                }
            }
            _ => {}
        }
    });

    f.render_widget(viewport, area);
}



fn draw_help<B>(f: &mut Frame<B>, app: &mut App, area: Rect) //help menu, stored in compiled program as string literal
where
    B: Backend,
{
    let help = Paragraph::new( //formatted here as it would be displayed
"Main Commands\n
    Q | Quit        - Close the program\n
\n
    H | Help        - Switch to this interface\n
\n
    V | Vertex Mode - Switch to vertex editing interface\n
\n
    F | Face Mode   - Switch to face editing interface\n
\n
    O | Open File   - Open .OBJ or .STL file\n
\n
Vertex Mode\n
\n
Faces Mode"
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Quick Commands")
                .border_type(BorderType::Plain),
        );
    
    f.render_widget(help, area);
}
