use nov_viz::{agent_response_sample, show};

fn main() -> anyhow::Result<()> {
    show(agent_response_sample(), "Nov Viz Demo — agent response fixture")
}
