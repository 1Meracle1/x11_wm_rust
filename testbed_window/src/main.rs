use base::Rect;
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::{error, trace};
use x11_bindings::{
    bindings::{
        XCB_ATOM_ATOM, XCB_CW_BACK_PIXEL, XCB_CW_EVENT_MASK, XCB_EVENT_MASK_EXPOSURE,
        XCB_EVENT_MASK_STRUCTURE_NOTIFY, 
    },
    connection::Connection,
};

#[derive(Parser, Debug)]
#[command(subcommand_required = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Location {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Floating {
        name: String,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
    },
    Docked {
        #[arg(value_enum)]
        location: Location,
        name: String,
    },
    Normal {
        name: String,
    },
}

fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let args = Cli::parse();
    trace!("args: {:#?}", args);

    let conn = Connection::new().expect("Unable to establish connection to X11 server.");
    let screen_rect = conn.screen_rect();

    const WINDOW_TYPE_NAME: &str = "_NET_WM_WINDOW_TYPE";
    const WINDOW_TYPE_NORMAL_NAME: &str = "_NET_WM_WINDOW_TYPE_NORMAL";
    const WINDOW_TYPE_DIALOG_NAME: &str = "_NET_WM_WINDOW_TYPE_DIALOG";
    const WINDOW_TYPE_DOCK_NAME: &str = "_NET_WM_WINDOW_TYPE_DOCK";
    const STRUT_PARTIAL_NAME: &str = "_NET_WM_STRUT_PARTIAL";

    let atoms_map = conn.intern_atoms([
        WINDOW_TYPE_NAME,
        WINDOW_TYPE_NORMAL_NAME,
        WINDOW_TYPE_DIALOG_NAME,
        WINDOW_TYPE_DOCK_NAME,
        STRUT_PARTIAL_NAME,
    ]);

    if !atoms_map.contains_key(WINDOW_TYPE_NAME) {
        error!("Failed to retrieve {} atom", WINDOW_TYPE_NAME);
        return;
    }
    let window_type_atom = atoms_map.get(WINDOW_TYPE_NAME).unwrap();

    match args.command {
        Commands::Floating {
            name,
            x,
            y,
            width,
            height,
        } => {
            let x = x.or(Some(0)).unwrap();
            let y = y.or(Some(0)).unwrap();
            let width = width.or(Some(800)).unwrap();
            let height = height.or(Some(600)).unwrap();

            let window = conn.create_window(
                x,
                y,
                width,
                height,
                0,
                XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK,
                [
                    conn.screen().black_pixel,
                    XCB_EVENT_MASK_EXPOSURE | XCB_EVENT_MASK_STRUCTURE_NOTIFY,
                ],
            );

            conn.window_set_instance_class_names(
                window,
                format!("{} instance", name).as_str(),
                format!("{} class", name).as_str(),
            );

            conn.window_set_wm_normal_hints_size(window, width, height);

            if let Some(dialog_atom) = atoms_map.get(WINDOW_TYPE_DIALOG_NAME) {
                conn.window_set_atom(window, window_type_atom, dialog_atom);
            } else {
                error!("Failed to retrieve {} atom", WINDOW_TYPE_DIALOG_NAME);
                return;
            }

            conn.map_window(window);
        }
        Commands::Docked { location, name } => {
            let rect = match location {
                Location::Top | Location::Bottom => Rect {
                    x: 0,
                    y: 0,
                    width: screen_rect.width,
                    height: 25,
                },
                Location::Left | Location::Right => Rect {
                    x: 0,
                    y: 0,
                    width: 25,
                    height: screen_rect.height,
                },
            };
            let strut: [u32; 12] = match location {
                Location::Bottom => [
                    0,           // left
                    0,           // right
                    0,           // top
                    rect.height, // bottom
                    0,           // left_start_y
                    0,           // left_end_y
                    0,           // right_start_y
                    0,           // right_end_y
                    0,           // top_start_x
                    0,           // top_end_x (spanning the entire width)
                    0,           // bottom_start_x
                    rect.width,  // bottom_end_x
                ],
                Location::Top => [
                    0,           // left
                    0,           // right
                    rect.height, // top
                    0,           // bottom
                    0,           // left_start_y
                    0,           // left_end_y
                    0,           // right_start_y
                    0,           // right_end_y
                    0,           // top_start_x
                    rect.width,  // top_end_x (spanning the entire width)
                    0,           // bottom_start_x
                    0,           // bottom_end_x
                ],
                Location::Left => [
                    rect.width,  // left
                    0,           // right
                    0,           // top
                    0,           // bottom
                    0,           // left_start_y
                    rect.height, // left_end_y
                    0,           // right_start_y
                    0,           // right_end_y
                    0,           // top_start_x
                    0,           // top_end_x (spanning the entire width)
                    0,           // bottom_start_x
                    0,           // bottom_end_x
                ],
                Location::Right => [
                    0,           // left
                    rect.width,  // right
                    0,           // top
                    0,           // bottom
                    0,           // left_start_y
                    0,           // left_end_y
                    0,           // right_start_y
                    rect.height, // right_end_y
                    0,           // top_start_x
                    0,           // top_end_x (spanning the entire width)
                    0,           // bottom_start_x
                    0,           // bottom_end_x
                ],
            };
            let window = conn.create_window(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                0,
                XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK,
                [
                    conn.screen().black_pixel,
                    XCB_EVENT_MASK_EXPOSURE | XCB_EVENT_MASK_STRUCTURE_NOTIFY,
                ],
            );
            if let Some(dock_atom) = atoms_map.get(WINDOW_TYPE_DOCK_NAME) {
                conn.window_set_atom(window, window_type_atom, dock_atom);
            } else {
                error!("Failed to retrieve {} atom", WINDOW_TYPE_DOCK_NAME);
                return;
            }
            if let Some(strut_atom) = atoms_map.get(STRUT_PARTIAL_NAME) {
                conn.change_property(
                    window,
                    *strut_atom,
                    XCB_ATOM_ATOM,
                    32,
                    12,
                    strut.as_ptr() as *const ::std::os::raw::c_void,
                );
            } else {
                error!("Failed to retrieve {} atom", STRUT_PARTIAL_NAME);
                return;
            }
            conn.window_set_instance_class_names(
                window,
                format!("{} instance", name).as_str(),
                format!("{} class", name).as_str(),
            );

            conn.window_set_wm_normal_hints_size(window, rect.width, rect.height);

            conn.map_window(window);
        }
        Commands::Normal { name } => {
            let width = 800 as u32;
            let height = 600 as u32;

            let window = conn.create_window(
                0,
                0,
                width,
                height,
                0,
                XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK,
                [
                    conn.screen().black_pixel,
                    XCB_EVENT_MASK_EXPOSURE | XCB_EVENT_MASK_STRUCTURE_NOTIFY,
                ],
            );

            conn.window_set_instance_class_names(
                window,
                format!("{} instance", name).as_str(),
                format!("{} class", name).as_str(),
            );

            conn.window_set_wm_normal_hints_size(window, width, height);

            if let Some(normal_atom) = atoms_map.get(WINDOW_TYPE_NORMAL_NAME) {
                conn.window_set_atom(window, window_type_atom, normal_atom);
            } else {
                error!("Failed to retrieve {} atom", WINDOW_TYPE_NORMAL_NAME);
                return;
            }

            conn.map_window(window);
        }
    }
    conn.flush();

    loop {
        let _ = conn.poll_for_event();
    }
}
