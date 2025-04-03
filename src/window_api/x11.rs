//! Contains logic for extracting records through x11. The implementation uses xcb for communication
//! with the server.

use anyhow::{Result, anyhow};
use sysinfo::Pid;
use tracing::{error, instrument};
use xcb::{
    Connection,
    screensaver::{QueryInfo, QueryInfoReply},
    x::{
        self, ATOM_ANY, Atom, Drawable, GetProperty, InternAtom, Window,
    },
};

use super::{ActiveWindowData, WindowManager};

fn get_pid_atom(conn: &Connection) -> Result<Atom> {
    let reply = conn.wait_for_reply(conn.send_request(&InternAtom {
        only_if_exists: false,
        name: b"_NET_WM_PID",
    }))?;
    Ok(reply.atom())
}

fn get_pid(conn: &Connection, window: Window, pid_atom: Atom) -> Result<Option<u32>> {
    let result = conn.wait_for_reply(conn.send_request(&GetProperty {
        delete: false,
        window,
        property: pid_atom,
        r#type: ATOM_ANY,
        long_offset: 0,
        long_length: 1,
    }))?;
    let result_slice = result.value::<u32>();
    if result_slice.is_empty() {
        return Ok(None);
    }
    Ok(Some(result_slice[0]))
}

fn get_process_name(id: u32) -> Result<Option<String>> {
    let system = sysinfo::System::new_all();
    let Some(process) = system.process(Pid::from_u32(id)) else {
        return Ok(None);
    };

    Ok(process
        .exe()
        .and_then(|v| v.to_str())
        .map(|v| v.to_string()))
}

fn get_active_window_atom(conn: &Connection) -> Result<Atom> {
    let active_window_atom = conn.wait_for_reply(conn.send_request(&InternAtom {
        only_if_exists: false,
        name: b"_NET_ACTIVE_WINDOW",
    }))?;
    Ok(active_window_atom.atom())
}

fn get_active_window(conn: &Connection, root: &Window, active_window_atom: Atom) -> Result<Window> {
    let result = conn.wait_for_reply(conn.send_request(&GetProperty {
        delete: false,
        window: *root,
        property: active_window_atom,
        r#type: ATOM_ANY,
        long_offset: 0,
        long_length: 1,
    }))?;
    Ok(result.value::<Window>()[0])
}

fn get_net_wm_name_atom(conn: &Connection) -> Result<Atom> {
    let response = conn.wait_for_reply(conn.send_request(&InternAtom {
        only_if_exists: false,
        name: b"_NET_WM_NAME",
    }))?;
    Ok(response.atom())
}

pub fn get_name(conn: &Connection, window: Window, wm_name_atom: Atom) -> Result<String> {
    let wm_name = conn.wait_for_reply(conn.send_request(&x::GetProperty {
        delete: false,
        window,
        property: wm_name_atom,
        r#type: x::ATOM_ANY,
        long_offset: 0,
        long_length: 1024,
    }))?;
    let title = String::from_utf8(wm_name.value().to_vec())
        .expect("The WM_NAME property is not valid UTF-8");
    Ok(title)
}

struct WindowData {
    connection: Connection,
    preferred_screen: usize,
    active_window_atom: Atom,
    window_name_atom: Atom,
    pid_atom: Atom,
}

impl WindowData {
    #[instrument(skip(self))]
    fn get_active_inner(&self) -> Result<ActiveWindowData> {
        let setup = self.connection.get_setup();

        // Currently the application only supports 1 x11 screen.
        let default_window = setup
            .roots()
            .nth(self.preferred_screen)
            .unwrap()
            .root();

        let active_window =
            get_active_window(&self.connection, &default_window, self.active_window_atom)?;
        let window_name = get_name(&self.connection, active_window, self.window_name_atom)?;
        let process = get_pid(&self.connection, active_window, self.pid_atom)?.ok_or_else(|| anyhow!("Failed to get pid: pid is None"))?;
        let process_name = get_process_name(process)?.ok_or_else(|| anyhow!("Failed to get process name: process name is None"))?;
        Ok(ActiveWindowData {
            window_title: window_name.into(),
            process_name: process_name.into(),
        })
    }
}

pub struct LinuxWindowManager {
    data: Option<WindowData>,
}

impl LinuxWindowManager {
    pub fn new() -> Result<Self> {
        Ok(Self { data: None })
    }

    fn try_reload_manager(&mut self) -> Result<WindowData> {
        let (connection, preferred_screen) = xcb::Connection::connect(None)
            .inspect_err(|e| error!("Failed creating connection {e:?}"))?;
        if preferred_screen < 0 {
            return Err(anyhow!(
                "Preferred screen is less than 0 {preferred_screen}"
            ));
        }
        let preferred_screen = preferred_screen as usize;
        let active_window_atom = get_active_window_atom(&connection)
            .inspect_err(|e| error!("Failed getting active window atom {e:?}"))?;
        let name_atom = get_net_wm_name_atom(&connection)
            .inspect_err(|e| error!("Failed getting wm name atom {e:?}"))?;
        let pid_atom =
            get_pid_atom(&connection).inspect_err(|e| error!("Failed getting pid of an atom {e:?}"))?;
        Ok(WindowData {
            connection,
            preferred_screen,
            active_window_atom,
            window_name_atom: name_atom,
            pid_atom,
        })
    }

    fn try_get_data(&mut self) -> Result<WindowData> {
        match self
            .data
            .take()
            .filter(|v| v.connection.has_error().is_ok())
        {
            Some(data) => Ok(data),
            None => match self.try_reload_manager() {
                Ok(data) => Ok(data),
                Err(e) => {
                    error!("Failed to get xcb connection {e:?}");
                    Err(e)
                }
            },
        }
    }
}

impl WindowManager for LinuxWindowManager {
    #[instrument(skip(self))]
    fn get_active_window_data(&mut self) -> Result<ActiveWindowData> {
        let data = self
            .try_get_data()
            .inspect_err(|e| error!("Failed getting connection {e:?}"))?;
        let result = data.get_active_inner();
        self.data = Some(data);
        result
    }

    #[instrument(skip(self))]
    fn get_idle_time(&mut self) -> Result<u32> {
        let data = self
            .try_get_data()
            .inspect_err(|e| error!("Failed getting connection {e:?}"))?;
        let w = data.connection.get_setup();
        let wnd = w.roots().nth(data.preferred_screen).unwrap().root();
        let idle = data.connection.send_request(&QueryInfo {
            drawable: Drawable::Window(wnd),
        });
        let reply: QueryInfoReply = data
            .connection
            .wait_for_reply(idle)
            .inspect_err(|e| error!("Failed getting idle {e}"))?;
        Ok(reply.ms_since_user_input())
    }
}
