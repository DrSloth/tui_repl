use tui_repl::Repl;

fn main() -> std::io::Result<()> {
    Repl::new_run_fullscreen(|cmd: String, out: &mut String| {
        out.push('\n');
        out.push_str(&cmd);
        out.push('\n');
        Ok(())
    })
}
