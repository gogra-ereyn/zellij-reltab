use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    permissions_granted: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunActionsAsUser,
        ]);
    }

    fn pipe(&mut self, msg: PipeMessage) -> bool {

        if msg.name != "new-tab-right" || !self.permissions_granted {
            return false;
        };

        let Ok((current_id, focused_pane)) = get_focused_pane_info() else {
            return false;
        };

        let Some(current) = get_tab_info(current_id) else {
            return false;
        };


        let cwd=msg.payload.filter(|p| !p.is_empty())
            .or_else(||get_pane_cwd(focused_pane).ok().map(|p|p.display().to_string()));

        let Some(new_id)=new_tab(None, cwd) else {
            return false;
        };

        let Some(new) = get_tab_info(new_id) else {
            return false;
        };

        for _ in current.position + 1..new.position {
            run_action(actions::Action::MoveTabByTabId{id:new_id as u64, direction: Direction::Left}, BTreeMap::new(),
            );
        }
        false

    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(status) => {
                match status {
                    PermissionStatus::Granted => {
                        self.permissions_granted = true;
                        hide_self();
                        eprintln!("got perms brudda");
                    }
                    PermissionStatus::Denied => {
                        self.permissions_granted = false;
                        eprintln!("NO perms brudda");
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
    }
}

