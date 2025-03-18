use anyhow::Result;
use sysinfo::Pid;
use tracing::instrument;
use xcb::{
    screensaver::{QueryInfo, QueryInfoReply},
    x::{
        self, Atom, Drawable, GetProperty, GrabServer, InternAtom, UngrabServer, Window, ATOM_ANY,
    },
    Connection,
};

use super::{ActiveWindowData, WindowManager};

fn get_pid_atom(conn: &Connection) -> Result<Atom> {
    let reply = conn.wait_for_reply(conn.send_request(&InternAtom {
        only_if_exists: false,
        name: b"_NET_WM_PID",
    }))?;
    Ok(reply.atom())
}

fn get_pid(conn: &Connection, window: Window) -> Result<Option<u32>> {
    let result = conn.wait_for_reply(conn.send_request(&GetProperty {
        delete: false,
        window,
        property: get_pid_atom(conn)?,
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

fn get_active_window(conn: &Connection, root: &Window) -> Result<Window> {
    let result = conn.wait_for_reply(conn.send_request(&GetProperty {
        delete: false,
        window: *root,
        property: get_active_window_atom(conn)?,
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

pub fn get_name(conn: &Connection, window: Window) -> Result<String> {
    let wm_name = conn.wait_for_reply(conn.send_request(&x::GetProperty {
        delete: false,
        window,
        property: get_net_wm_name_atom(conn)?,
        r#type: x::ATOM_ANY,
        long_offset: 0,
        long_length: 1024,
    }))?;
    let title = String::from_utf8(wm_name.value().to_vec())
        .expect("The WM_NAME property is not valid UTF-8");
    Ok(title)
}

pub struct LinuxWindowManager {
    connection: Connection,
    preferred_screen: i32,
}

impl LinuxWindowManager {
    pub fn new() -> Result<Self> {
        let (connection, preferred_screen) = xcb::Connection::connect(None)?;
        Ok(Self {
            connection,
            preferred_screen,
        })
    }

    #[instrument(skip(conn))]
    pub fn get_active_internal(
        conn: &Connection,
        preferred_screen: u32,
    ) -> Result<ActiveWindowData> {
        let setup = conn.get_setup();

        // Currently the application only supports 1 x11 screen.
        let default_window = setup.roots().nth(preferred_screen as usize).unwrap().root();

        let active_window = get_active_window(conn, &default_window)?;
        let window_name = get_name(conn, active_window)?;
        let process = get_pid(conn, active_window)?.unwrap();
        let process_name = get_process_name(process)?.unwrap();
        Ok(ActiveWindowData {
            window_title: window_name.into(),
            process_name: process_name.into(),
        })
    }
}

impl WindowManager for LinuxWindowManager {
    #[instrument(skip(self))]
    fn get_active_window_data(&mut self) -> Result<ActiveWindowData> {
        assert!(self.preferred_screen >= 0);
        let screen_num: u32 = self.preferred_screen.try_into().unwrap();

        let _ = self.connection.send_request(&GrabServer {});

        let result = Self::get_active_internal(&self.connection, screen_num);
        let _ = self.connection.send_request(&UngrabServer {});
        result
    }

    #[instrument(skip(self))]
    fn get_idle_time(&mut self) -> Result<u32> {
        let w = self.connection.get_setup();
        let wnd = w.roots().nth(self.preferred_screen as usize).unwrap().root();
        let idle = self.connection.send_request(&QueryInfo {
            drawable: Drawable::Window(wnd),
        });
        let reply: QueryInfoReply = self.connection.wait_for_reply(idle)?;
        Ok(reply.ms_since_user_input())
    }
}
