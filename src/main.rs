use std::collections::BTreeMap;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    permissioned: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        eprintln!("[openat::load]");
        subscribe(&[EventType::PermissionRequestResult]);
        request_permission(&[
            PermissionType::ReadCliPipes,
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunActionsAsUser,
        ]);
    }

    fn pipe(&mut self, msg: PipeMessage) -> bool {
        eprintln!("[openat::pipe]");
        if msg.name != "tab-openat" || !self.permissioned {
            return false;
        };

        let offset: isize = match msg.payload.as_deref().map(str::trim) {
            None | Some("") => 1,
            Some(s) => match s.parse() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("openat bad offset to open at: {s:?}");
                    return false;
                }
            },
        };

        eprintln!("[openat::pipe] got offset={offset:?}");

        let Ok((current_id, focused_pane)) = get_focused_pane_info() else {
            return false;
        };

        eprintln!("[openat::pipe] got current_id={current_id:?}");
        let Some(current) = get_tab_info(current_id) else {
            return false;
        };
        eprintln!("[openat::pipe] got current {current:?}");

        // TODO which is better?
        let cwd_two = msg
            .args
            .get("cwd")
            .filter(|p| !p.is_empty())
            .cloned()
            .or_else(|| {
                get_pane_cwd(focused_pane)
                    .ok()
                    .map(|p| p.display().to_string())
            });

        eprintln!("cwd_two={cwd_two:?}");

        let cwd = msg.payload.filter(|p| !p.is_empty()).or_else(|| {
            get_pane_cwd(focused_pane)
                .ok()
                .map(|p| p.display().to_string())
        });
        dbg!(&cwd);

        let Some(new_id) = new_tab(None, cwd) else {
            return false;
        };

        dbg!(new_id);

        let Some(new) = get_tab_info(new_id) else {
            return false;
        };
        dbg!(&new);

        let last = new.position as isize;
        let cur = current.position as isize;
        let target = if offset > 0 {
            cur + offset
        } else {
            cur + offset + 1
        }
        .clamp(0, last);

        dbg!(last, cur, target);
        for i in target..last {
            dbg!(i);
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

        dbg!("before unblock cli");
        if let PipeSource::Cli(id) = &msg.source {
            unblock_cli_pipe_input(id);
        }

        dbg!("after unblock cli");
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
