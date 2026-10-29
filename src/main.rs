



fn pipe(&mut self, msg: PipeMessage) -> bool {


    if msg.name != "new-tab-right" || !self.permissions_granted {
        return false;
    }


    if let Ok((current_id, focused_pane)) = get_focused_pane_info()  else {
        return false;
    }

    let Some(current) = get_tab_info(current_id) else {
        return false;
    }







}



fn main() {
    println!("Hello, world!");
}
