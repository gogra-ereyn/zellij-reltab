use std::collections::BTreeMap;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    permissioned: bool,
}

register_plugin!(State);

const OAN: &str = env!("CARGO_BIN_NAME");

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        subscribe(&[EventType::PermissionRequestResult]);
        request_permission(&[
            PermissionType::ReadCliPipes,
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunActionsAsUser,
        ]);
    }

    fn pipe(&mut self, mut msg: PipeMessage) -> bool {
        if msg.name != "reltab" || !self.permissioned {
            return false;
        };

        let offset: isize = match msg.payload.as_deref().map(str::trim) {
            None | Some("") => 1,
            Some(s) => match s.parse() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("{OAN} bad offset to open at: {s:?}");
                    return false;
                }
            },
        };

        let Ok((current_id, focused_pane)) = get_focused_pane_info() else {
            return false;
        };

        let Some(current) = get_tab_info(current_id) else {
            return false;
        };

        let cwd = msg
            .args
            .remove("cwd")
            .filter(|p| !p.is_empty())
            .or_else(|| {
                get_pane_cwd(focused_pane)
                    .ok()
                    .map(|p| p.display().to_string())
            });

        let Some(new_id) = new_tab(None, cwd) else {
            return false;
        };

        let Some(new) = get_tab_info(new_id) else {
            return false;
        };

        let last = new.position as isize;
        let cur = current.position as isize;
        let target = if offset > 0 {
            cur + offset
        } else {
            cur + offset + 1
        }
        .clamp(0, last);

        // TODO - for whatever reason MoveTabByTabId
        //  just crashes int he shim every time.
        //  rather than try figure out why tf that is,
        //  just using MoveTab instead.
        for _ in target..last {
            run_action(
                actions::Action::MoveTab {
                    direction: Direction::Left,
                },
                BTreeMap::new(),
            );
            //run_action(
            //    actions::Action::MoveTabByTabId {
            //        id: new_id as u64,
            //        direction: Direction::Left,
            //    },
            //    BTreeMap::new(),
            //);
        }

        if let PipeSource::Cli(id) = &msg.source {
            unblock_cli_pipe_input(id);
        }

        false
    }

    fn update(&mut self, event: Event) -> bool {
        let Event::PermissionRequestResult(status) = event else {
            return false;
        };
        if let PermissionStatus::Granted = status {
            self.permissioned = true;
            hide_self();
        } else {
            self.permissioned = false;
        };
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}
