mod alert;
mod highlight;
mod math;
mod renderer;
pub mod slug;

pub use renderer::render_markdown as md_to_html_gfm_highlight;

#[derive(Default, Clone)]
pub struct MarkdownRenderer;

impl MarkdownRenderer {
    pub fn render(md: &str) -> String {
        md_to_html_gfm_highlight(md)
    }
}
